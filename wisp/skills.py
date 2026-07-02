"""Skill registry: one skill system shared by Claude Code and the HUD.

Each skill is a directory under `.claude/skills/<name>/` with a `SKILL.md`:

    ---
    name: morning-report
    description: Build the morning report from the vault and repo state.
    triggers: rundown, morning report, what's happening
    runner: builtin            # builtin | claude
    builtin: morning_report    # function in wisp.builtins (runner: builtin)
    hud-icon: ☀
    hud-group: briefing
    voice-reply: true
    ---
    <markdown body — the prompt/instructions used for headless Claude runs,
     and the skill instructions Claude Code itself sees>

Claude Code discovers these as normal skills; Wisp OS reads the same files to
render buttons, compile trigger regexes and dispatch runs.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

from .vault import parse_frontmatter


@dataclass
class Skill:
    name: str
    description: str = ""
    triggers: list[str] = field(default_factory=list)
    runner: str = "builtin"
    builtin: str = ""
    hud_icon: str = "◆"
    hud_group: str = "general"
    voice_reply: bool = False
    body: str = ""
    path: Path | None = None

    def trigger_patterns(self) -> list[re.Pattern]:
        pats = []
        for phrase in self.triggers:
            words = [re.escape(w) for w in phrase.split()]
            if not words:
                continue
            pats.append(re.compile(r"\b" + r"\s+".join(words) + r"\b", re.IGNORECASE))
        return pats

    def to_json(self) -> dict:
        return {
            "name": self.name,
            "description": self.description,
            "triggers": self.triggers,
            "runner": self.runner,
            "icon": self.hud_icon,
            "group": self.hud_group,
        }


def _parse_skill(path: Path) -> Skill | None:
    try:
        meta, body = parse_frontmatter(path.read_text())
    except OSError:
        return None
    name = meta.get("name", path.parent.name)
    triggers = [t.strip() for t in meta.get("triggers", "").split(",") if t.strip()]
    return Skill(
        name=name,
        description=meta.get("description", ""),
        triggers=triggers,
        runner=meta.get("runner", "builtin"),
        builtin=meta.get("builtin", ""),
        hud_icon=meta.get("hud-icon", "◆"),
        hud_group=meta.get("hud-group", "general"),
        voice_reply=meta.get("voice-reply", "false").lower() in ("1", "true", "yes"),
        body=body.strip(),
        path=path,
    )


class SkillRegistry:
    def __init__(self, skills_dir: Path):
        self.skills_dir = Path(skills_dir)
        self._skills: dict[str, Skill] = {}
        self.reload()

    def reload(self) -> None:
        self._skills = {}
        if not self.skills_dir.exists():
            return
        for manifest in sorted(self.skills_dir.glob("*/SKILL.md")):
            skill = _parse_skill(manifest)
            if skill:
                self._skills[skill.name] = skill

    def all(self) -> list[Skill]:
        return list(self._skills.values())

    def get(self, name: str) -> Skill | None:
        return self._skills.get(name)

    def match_triggers(self, text: str) -> Skill | None:
        """First routing stage from the video: plain regex trigger words."""
        best: tuple[int, Skill] | None = None
        for skill in self._skills.values():
            for pat in skill.trigger_patterns():
                m = pat.search(text)
                if m:
                    length = len(m.group(0))
                    if best is None or length > best[0]:
                        best = (length, skill)
        return best[1] if best else None
