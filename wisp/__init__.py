"""Wisp OS — the OpenSourceWisper agentic OS layered on top of Claude Code.

Blueprint (from docs/video-notes-PW0sgog3kXY.md):

    voice/text ─► router (regex triggers → LLM router → keyword fallback)
              ─► skill dispatch (builtin handler or headless Claude Code)
              ─► vault (Obsidian-compatible markdown memory)
              ─► summary ─► TTS + HUD pop-up

Everything degrades gracefully: no API key, no `claude` CLI, no microphone and no
TTS engine are all fine — the OS still routes, runs builtin skills, writes the
vault and serves the HUD.
"""

from pathlib import Path

__version__ = "0.1.0"

# Repo root = parent of the package directory.
REPO_ROOT = Path(__file__).resolve().parent.parent
VAULT_DIR = REPO_ROOT / "vault"
SKILLS_DIR = REPO_ROOT / ".claude" / "skills"
