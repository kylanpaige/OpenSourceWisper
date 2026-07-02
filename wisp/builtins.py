"""Builtin skill handlers — deterministic, dependency-free implementations.

These make the OS fully functional with zero credentials. Skills that declare
`runner: claude` use headless Claude Code instead when it's available; these
builtins are also its fallback so a click on the HUD never dead-ends.

Each handler takes a `SkillContext` and returns a `SkillResult`; the dispatcher
persists the result to the vault and (optionally) speaks the summary.
"""

from __future__ import annotations

import datetime as _dt
import shutil
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

from .config import Config
from .vault import Vault, parse_frontmatter


@dataclass
class SkillContext:
    config: Config
    vault: Vault
    repo_root: Path
    text: str = ""  # the utterance that triggered the skill, if any


@dataclass
class SkillResult:
    title: str
    summary: str
    body: str
    tags: tuple[str, ...] = ()
    directives: list[str] = field(default_factory=list)  # if set, update vault directives


def _run(cmd: list[str], cwd: Path, timeout: int = 120) -> tuple[int, str]:
    if not shutil.which(cmd[0]):
        return 127, f"{cmd[0]}: not installed"
    try:
        proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return proc.returncode, (proc.stdout + proc.stderr).strip()
    except subprocess.TimeoutExpired:
        return 124, "timed out"
    except OSError as exc:
        return 1, str(exc)


def _git(repo: Path, *args: str) -> str:
    code, out = _run(["git", *args], cwd=repo)
    return out if code == 0 else ""


# ---------------------------------------------------------------------------
# briefing
# ---------------------------------------------------------------------------

def morning_report(ctx: SkillContext) -> SkillResult:
    today = _dt.date.today().isoformat()
    repo = ctx.repo_root
    branch = _git(repo, "rev-parse", "--abbrev-ref", "HEAD") or "unknown"
    last_commits = _git(repo, "log", "--oneline", "-5")
    status = _git(repo, "status", "--porcelain")
    dirty = len([l for l in status.splitlines() if l.strip()])
    stats = ctx.vault.stats()
    directives = ctx.vault.directives()
    schedule = ctx.vault.schedule()
    inbox = ctx.vault.inbox_items()

    lines = [
        f"Generated {today}.",
        "",
        "## Repo",
        f"- Branch **{branch}**, {dirty} uncommitted change(s).",
        "```",
        last_commits or "(no commits)",
        "```",
        "",
        "## Vault",
        f"- {stats['notes']} notes, {stats['reports']} reports, {stats['inbox']} inbox items.",
        "",
        "## Directives",
    ]
    lines += [f"{i}. {d}" for i, d in enumerate(directives, 1)] or ["(none set — run daily-review)"]
    if schedule:
        lines += ["", "## Schedule"] + [f"- {s['time']} — {s['item']}" for s in schedule]
    if inbox:
        lines += ["", "## Inbox (pending triage)"] + [
            f"- **{i.get('subject')}** from {i.get('from', '?')} ({i.get('kind', 'item')})" for i in inbox
        ]
        lines += ["", "Run [[reports|inbox-brief]] for a full triage."]

    top = directives[0] if directives else "no directive set"
    summary = (
        f"Morning report ready. Branch {branch} with {dirty} uncommitted changes, "
        f"{stats['inbox']} inbox items pending. Top of the board: {top}."
    )
    return SkillResult(title=f"Morning Report {today}", summary=summary, body="\n".join(lines), tags=("briefing",))


def inbox_brief(ctx: SkillContext) -> SkillResult:
    items = ctx.vault.inbox_items()
    groups: dict[str, list[dict]] = {}
    for item in items:
        groups.setdefault(item.get("kind", "other"), []).append(item)

    lines = [f"{len(items)} item(s) in the inbox.", ""]
    order = {"urgent": 0, "lead": 1, "sponsor": 2, "other": 3}
    for kind in sorted(groups, key=lambda k: order.get(k, 9)):
        lines.append(f"## {kind.title()} ({len(groups[kind])})")
        for item in groups[kind]:
            lines.append(f"- **{item.get('subject')}** — from {item.get('from', '?')}")
            if item.get("body"):
                first = item["body"].splitlines()[0][:120]
                lines.append(f"  > {first}")
        lines.append("")

    counts = ", ".join(f"{len(v)} {k}" for k, v in sorted(groups.items(), key=lambda kv: order.get(kv[0], 9)))
    summary = f"Inbox brief done. {len(items)} threads: {counts}." if items else "Inbox brief done. Inbox is empty."
    return SkillResult(title=f"Inbox Brief {_dt.date.today().isoformat()}", summary=summary,
                       body="\n".join(lines), tags=("briefing", "inbox"))


def daily_review(ctx: SkillContext) -> SkillResult:
    repo = ctx.repo_root
    today = _dt.date.today().isoformat()
    commits_today = _git(repo, "log", "--oneline", "--since=midnight")
    reports = ctx.vault.reports(limit=6)
    inbox = ctx.vault.inbox_items()

    new_directives: list[str] = []
    if inbox:
        new_directives.append(f"Clear the inbox ({len(inbox)} items pending triage)")
    status = _git(repo, "status", "--porcelain")
    if status.strip():
        new_directives.append("Commit or clean the working tree")
    new_directives.append("Advance the wisper audio pipeline (RESEARCH.md §5 phase plan)")
    new_directives = new_directives[:3]

    lines = [
        f"Daily review for {today}.",
        "",
        "## Shipped today",
        "```",
        commits_today or "(no commits today)",
        "```",
        "",
        "## Recent reports",
    ]
    lines += [f"- [[reports/{r.name}|{r.title}]] ({r.skill})" for r in reports] or ["(none)"]
    lines += ["", "## Directives for tomorrow"] + [f"{i}. {d}" for i, d in enumerate(new_directives, 1)]

    summary = (
        f"Daily review done. {len(commits_today.splitlines()) if commits_today else 0} commit(s) today; "
        f"set {len(new_directives)} directives for tomorrow."
    )
    return SkillResult(title=f"Daily Review {today}", summary=summary, body="\n".join(lines),
                       tags=("review",), directives=new_directives)


# ---------------------------------------------------------------------------
# vault
# ---------------------------------------------------------------------------

def vault_sync(ctx: SkillContext) -> SkillResult:
    ctx.vault.ensure_layout()
    stats = ctx.vault.stats()
    orphans = []
    for path in (ctx.vault.root / "reports").glob("*.md"):
        meta, _ = parse_frontmatter(path.read_text())
        if not meta.get("title"):
            orphans.append(path.name)
    ctx.vault._refresh_home()
    lines = [
        f"Vault re-indexed: {stats['notes']} notes, {stats['reports']} reports, {stats['inbox']} inbox items.",
        "",
        "Home.md recent-reports section refreshed.",
    ]
    if orphans:
        lines += ["", "## Reports missing frontmatter"] + [f"- {o}" for o in orphans]
    summary = (
        f"Vault synced: {stats['notes']} notes indexed, {len(orphans)} report(s) missing frontmatter."
        if orphans else f"Vault synced: {stats['notes']} notes indexed, Home.md refreshed."
    )
    return SkillResult(title=f"Vault Sync {_dt.date.today().isoformat()}", summary=summary,
                       body="\n".join(lines), tags=("vault",))


# ---------------------------------------------------------------------------
# repo
# ---------------------------------------------------------------------------

def repo_pulse(ctx: SkillContext) -> SkillResult:
    repo = ctx.repo_root
    branch = _git(repo, "rev-parse", "--abbrev-ref", "HEAD")
    log = _git(repo, "log", "--oneline", "-8")
    status = _git(repo, "status", "--short")
    code, check = _run(["cargo", "check", "--quiet", "--message-format=short"], cwd=repo, timeout=300)
    check_line = "cargo check: ✅ clean" if code == 0 else f"cargo check: ❌ ({'not installed' if code == 127 else 'errors'})"

    lines = [
        f"## Branch\n`{branch}`",
        "",
        f"## Build\n{check_line}",
        "",
        "## Working tree",
        "```",
        status or "(clean)",
        "```",
        "",
        "## Recent commits",
        "```",
        log or "(none)",
        "```",
    ]
    if code not in (0, 127) and check:
        lines += ["", "## cargo check output", "```", check[-1500:], "```"]
    summary = f"Repo pulse: branch {branch}, {check_line.split(': ')[1]}."
    return SkillResult(title=f"Repo Pulse {_dt.date.today().isoformat()}", summary=summary,
                       body="\n".join(lines), tags=("repo",))


def ship_check(ctx: SkillContext) -> SkillResult:
    repo = ctx.repo_root
    steps = [
        ("cargo fmt --check", ["cargo", "fmt", "--check"]),
        ("cargo check", ["cargo", "check", "--quiet"]),
        ("cargo test", ["cargo", "test", "--quiet"]),
        ("wisp tests", ["python3", "-m", "unittest", "discover", "-s", "wisp/tests", "-t", ".", "-q"]),
    ]
    results, failed = [], []
    for label, cmd in steps:
        code, out = _run(cmd, cwd=repo, timeout=600)
        ok = code == 0
        skipped = code == 127
        mark = "⏭ skipped" if skipped else ("✅ pass" if ok else "❌ FAIL")
        results.append((label, mark, out))
        if not ok and not skipped:
            failed.append(label)

    lines = ["| Step | Result |", "|---|---|"]
    lines += [f"| `{label}` | {mark} |" for label, mark, _ in results]
    for label, mark, out in results:
        if "FAIL" in mark and out:
            lines += ["", f"## {label} output", "```", out[-2000:], "```"]

    summary = "Ship check: all green — clear to commit." if not failed else \
              f"Ship check: {len(failed)} step(s) failing ({', '.join(failed)})."
    return SkillResult(title=f"Ship Check {_dt.date.today().isoformat()}", summary=summary,
                       body="\n".join(lines), tags=("repo", "ci"))


# ---------------------------------------------------------------------------
# voice / dictation (integration point with the wisper-core crate)
# ---------------------------------------------------------------------------

def transcribe(ctx: SkillContext) -> SkillResult:
    engines = []
    try:
        import faster_whisper  # noqa: F401
        engines.append("faster-whisper (installed)")
    except ImportError:
        engines.append("faster-whisper (not installed — `pip install faster-whisper`)")
    if shutil.which("whisper-cli") or shutil.which("main"):
        engines.append("whisper.cpp CLI (installed)")
    engines.append("wisper-core (this repo — audio pipeline lands with the Phase 1 MVP)")

    body = (
        f"Requested: {ctx.text or '(no audio provided)'}\n\n"
        "## Available speech-to-text engines\n" + "\n".join(f"- {e}" for e in engines) +
        "\n\nOnce the wisper Phase 1 MVP ships, this skill pipes microphone audio through "
        "the local `wisper` engine and drops clean text at the cursor — the same pipeline "
        "the OS voice loop uses (see `wisp/voice.py`)."
    )
    summary = f"Transcription stack surveyed: {sum('installed)' in e for e in engines)} engine(s) ready."
    return SkillResult(title=f"Transcribe {_dt.date.today().isoformat()}", summary=summary,
                       body=body, tags=("voice",))


HANDLERS = {
    "morning_report": morning_report,
    "inbox_brief": inbox_brief,
    "daily_review": daily_review,
    "vault_sync": vault_sync,
    "repo_pulse": repo_pulse,
    "ship_check": ship_check,
    "transcribe": transcribe,
}
