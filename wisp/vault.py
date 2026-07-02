"""Obsidian-compatible markdown vault: the memory layer of Wisp OS.

Layout:
    vault/
      Home.md               # index; "Recent reports" section is machine-maintained
      schedule.md           # today's agenda ("- 09:00 Deep work")
      directives/current.md # top priorities ("1. …")
      inbox/*.md            # triage items with frontmatter (from, subject, kind)
      reports/*.md          # skill output, frontmatter + wikilinks
      log/trail.jsonl       # document trail (most recently touched notes)

Reports are plain markdown with YAML-ish frontmatter so Obsidian, Claude Code
and the HUD all read the same files.
"""

from __future__ import annotations

import datetime as _dt
import json
import re
from dataclasses import dataclass
from pathlib import Path

_RECENT_START = "<!-- wisp:recent-reports -->"
_RECENT_END = "<!-- wisp:end-recent-reports -->"


def slugify(text: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")
    return slug or "note"


def parse_frontmatter(text: str) -> tuple[dict, str]:
    """Parse a minimal `key: value` frontmatter block. Returns (meta, body)."""
    if not text.startswith("---"):
        return {}, text
    lines = text.splitlines()
    meta: dict = {}
    for i, line in enumerate(lines[1:], start=1):
        if line.strip() == "---":
            return meta, "\n".join(lines[i + 1:]).lstrip("\n")
        if ":" in line:
            key, _, value = line.partition(":")
            meta[key.strip()] = value.strip()
    return {}, text


def render_frontmatter(meta: dict) -> str:
    lines = ["---"]
    for key, value in meta.items():
        lines.append(f"{key}: {value}")
    lines.append("---")
    return "\n".join(lines)


@dataclass
class Report:
    path: Path
    title: str
    date: str
    skill: str
    engine: str
    summary: str

    @property
    def name(self) -> str:
        return self.path.stem


class Vault:
    def __init__(self, root: Path):
        self.root = Path(root)

    # -- layout ----------------------------------------------------------
    def ensure_layout(self) -> None:
        for sub in ("reports", "inbox", "directives", "log"):
            (self.root / sub).mkdir(parents=True, exist_ok=True)
        home = self.root / "Home.md"
        if not home.exists():
            home.write_text(
                "# Home\n\nWelcome to the Wisp OS vault.\n\n"
                f"## Recent reports\n\n{_RECENT_START}\n{_RECENT_END}\n"
            )

    # -- reports ---------------------------------------------------------
    def write_report(
        self,
        skill: str,
        title: str,
        body: str,
        summary: str,
        engine: str = "builtin",
        tags: tuple[str, ...] = (),
    ) -> Report:
        self.ensure_layout()
        now = _dt.datetime.now()
        date = now.strftime("%Y-%m-%d")
        # "Morning Report 2026-07-02" → "2026-07-02-morning-report", not date twice
        slug = slugify(title).removesuffix(f"-{date}").removeprefix(f"{date}-") or "note"
        stem = f"{date}-{slug}"
        path = self.root / "reports" / f"{stem}.md"
        n = 2
        while path.exists():
            path = self.root / "reports" / f"{stem}-{n}.md"
            n += 1
        meta = {
            "title": title,
            "date": now.strftime("%Y-%m-%d %H:%M"),
            "skill": skill,
            "engine": engine,
            "summary": summary.replace("\n", " ").strip(),
        }
        if tags:
            meta["tags"] = ", ".join(tags)
        path.write_text(f"{render_frontmatter(meta)}\n\n# {title}\n\n{body.rstrip()}\n")
        report = Report(path=path, title=title, date=meta["date"], skill=skill, engine=engine, summary=meta["summary"])
        self._touch_trail(path, action=f"report:{skill}")
        self._refresh_home()
        return report

    def reports(self, limit: int = 20) -> list[Report]:
        folder = self.root / "reports"
        if not folder.exists():
            return []
        out = []
        for path in sorted(folder.glob("*.md"), key=lambda p: p.stat().st_mtime, reverse=True)[:limit]:
            meta, _ = parse_frontmatter(path.read_text())
            out.append(
                Report(
                    path=path,
                    title=meta.get("title", path.stem),
                    date=meta.get("date", ""),
                    skill=meta.get("skill", ""),
                    engine=meta.get("engine", ""),
                    summary=meta.get("summary", ""),
                )
            )
        return out

    def read_report(self, name: str) -> tuple[dict, str] | None:
        path = self.root / "reports" / f"{name}.md"
        if not path.exists() or not path.resolve().is_relative_to((self.root / "reports").resolve()):
            return None
        meta, body = parse_frontmatter(path.read_text())
        self._touch_trail(path, action="read")
        return meta, body

    # -- directives / schedule / inbox ------------------------------------
    def directives(self) -> list[str]:
        path = self.root / "directives" / "current.md"
        if not path.exists():
            return []
        items = []
        for line in path.read_text().splitlines():
            m = re.match(r"\s*(?:\d+\.|[-*])\s+(.*)", line)
            if m and m.group(1).strip():
                items.append(m.group(1).strip())
        return items

    def set_directives(self, items: list[str]) -> None:
        self.ensure_layout()
        lines = [f"{i}. {item}" for i, item in enumerate(items, start=1)]
        path = self.root / "directives" / "current.md"
        path.write_text("# Directives\n\n" + "\n".join(lines) + "\n")
        self._touch_trail(path, action="directives")

    def schedule(self) -> list[dict]:
        path = self.root / "schedule.md"
        if not path.exists():
            return []
        out = []
        for line in path.read_text().splitlines():
            m = re.match(r"\s*[-*]\s+(\d{1,2}:\d{2})\s+(.*)", line)
            if m:
                out.append({"time": m.group(1), "item": m.group(2).strip()})
        return out

    def inbox_items(self) -> list[dict]:
        folder = self.root / "inbox"
        if not folder.exists():
            return []
        items = []
        for path in sorted(folder.glob("*.md")):
            meta, body = parse_frontmatter(path.read_text())
            meta.setdefault("subject", path.stem)
            meta["file"] = path.name
            meta["body"] = body.strip()
            items.append(meta)
        return items

    # -- document trail ----------------------------------------------------
    def _touch_trail(self, path: Path, action: str) -> None:
        log = self.root / "log"
        log.mkdir(parents=True, exist_ok=True)
        entry = {
            "ts": _dt.datetime.now().isoformat(timespec="seconds"),
            "note": str(path.relative_to(self.root)),
            "action": action,
        }
        with (log / "trail.jsonl").open("a") as fh:
            fh.write(json.dumps(entry) + "\n")

    def trail(self, limit: int = 12) -> list[dict]:
        path = self.root / "log" / "trail.jsonl"
        if not path.exists():
            return []
        lines = path.read_text().splitlines()[-limit:]
        out = []
        for line in reversed(lines):
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                continue
        return out

    # -- home index --------------------------------------------------------
    def _refresh_home(self) -> None:
        home = self.root / "Home.md"
        if not home.exists():
            self.ensure_layout()
        text = home.read_text()
        if _RECENT_START not in text or _RECENT_END not in text:
            text += f"\n\n## Recent reports\n\n{_RECENT_START}\n{_RECENT_END}\n"
        links = "\n".join(
            f"- [[reports/{r.name}|{r.title}]] — {r.summary[:80]}" for r in self.reports(limit=8)
        )
        head, _, rest = text.partition(_RECENT_START)
        _, _, tail = rest.partition(_RECENT_END)
        home.write_text(f"{head}{_RECENT_START}\n{links}\n{_RECENT_END}{tail}")

    # -- stats for the HUD ---------------------------------------------------
    def stats(self) -> dict:
        return {
            "reports": len(list((self.root / "reports").glob("*.md"))) if (self.root / "reports").exists() else 0,
            "inbox": len(list((self.root / "inbox").glob("*.md"))) if (self.root / "inbox").exists() else 0,
            "notes": len(list(self.root.rglob("*.md"))) if self.root.exists() else 0,
        }
