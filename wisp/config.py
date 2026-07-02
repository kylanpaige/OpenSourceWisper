"""Configuration for Wisp OS.

Defaults live here; overrides come from `wisp.config.json` at the repo root and
from environment variables (highest precedence). Stdlib only.
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass, field
from pathlib import Path

from . import REPO_ROOT


@dataclass
class Config:
    # Identity
    assistant_name: str = "Wisp"

    # Server
    host: str = "127.0.0.1"
    port: int = 8737  # "VSPR" on a phone keypad, more or less

    # Routing
    router_engines: list[str] = field(default_factory=lambda: ["claude-cli", "anthropic-api", "ollama"])
    router_model: str = "claude-haiku-4-5-20251001"
    ollama_url: str = "http://127.0.0.1:11434"
    ollama_model: str = "llama3.2"
    route_confidence_floor: float = 0.25

    # Headless Claude Code
    claude_bin: str = "claude"
    claude_timeout_s: int = 600
    claude_permission_mode: str = "acceptEdits"
    # When True, never invoke the claude CLI even if present (CI-friendly).
    simulate: bool = False

    # Voice
    stt_engines: list[str] = field(default_factory=lambda: ["faster-whisper", "stdin"])
    tts_engines: list[str] = field(default_factory=lambda: ["kokoro", "say", "espeak", "print"])

    # Paths
    vault_dir: Path = REPO_ROOT / "vault"
    skills_dir: Path = REPO_ROOT / ".claude" / "skills"

    @property
    def base_url(self) -> str:
        return f"http://{self.host}:{self.port}"


_ENV_MAP = {
    "WISP_HOST": ("host", str),
    "WISP_PORT": ("port", int),
    "WISP_SIMULATE": ("simulate", lambda v: v.lower() in ("1", "true", "yes")),
    "WISP_CLAUDE_BIN": ("claude_bin", str),
    "WISP_ROUTER_MODEL": ("router_model", str),
    "WISP_VAULT_DIR": ("vault_dir", Path),
    "WISP_SKILLS_DIR": ("skills_dir", Path),
}


def load_config(config_path: Path | None = None) -> Config:
    cfg = Config()
    path = config_path or REPO_ROOT / "wisp.config.json"
    if path.exists():
        try:
            data = json.loads(path.read_text())
        except (OSError, json.JSONDecodeError):
            data = {}
        for key, value in data.items():
            if hasattr(cfg, key):
                current = getattr(cfg, key)
                if isinstance(current, Path):
                    value = Path(value)
                setattr(cfg, key, value)
    for env, (attr, cast) in _ENV_MAP.items():
        if env in os.environ:
            setattr(cfg, attr, cast(os.environ[env]))
    return cfg
