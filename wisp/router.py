"""Routing layer — decides which skill handles an utterance.

Stage 1: regex trigger words from skill manifests (free, instant).
Stage 2: an LLM router, tried in order of configured engines:
           claude-cli    → `claude -p` with a cheap model (Haiku)
           anthropic-api → direct Messages API call (needs ANTHROPIC_API_KEY)
           ollama        → local model via the Ollama HTTP API
Stage 3: deterministic keyword-overlap scoring (always available).

The LLM stages are *only* routing — they pick a skill name from a closed list,
exactly as in the video ("we're simply routing here").
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import urllib.error
import urllib.request
from dataclasses import dataclass

from .config import Config
from .skills import Skill, SkillRegistry

CHAT = "chat"  # pseudo-skill: no skill matched; handle conversationally


@dataclass
class RouteDecision:
    skill: str
    engine: str
    confidence: float
    text: str

    def to_json(self) -> dict:
        return {"skill": self.skill, "engine": self.engine, "confidence": round(self.confidence, 3), "text": self.text}


def _router_prompt(text: str, skills: list[Skill]) -> str:
    menu = "\n".join(f"- {s.name}: {s.description} (triggers: {', '.join(s.triggers)})" for s in skills)
    return (
        "You are a router for a voice assistant. Pick the single best skill for the "
        "user's request from this list, or \"chat\" if none applies.\n\n"
        f"Skills:\n{menu}\n\nUser request: {text!r}\n\n"
        'Reply with ONLY a JSON object like {"skill": "<name>"} and nothing else.'
    )


def _extract_skill_name(raw: str, valid: set[str]) -> str | None:
    m = re.search(r"\{[^{}]*\}", raw, re.DOTALL)
    if m:
        try:
            name = json.loads(m.group(0)).get("skill", "")
            if name in valid or name == CHAT:
                return name
        except json.JSONDecodeError:
            pass
    for name in valid:
        if re.search(rf"\b{re.escape(name)}\b", raw):
            return name
    return None


class Router:
    def __init__(self, config: Config, registry: SkillRegistry):
        self.config = config
        self.registry = registry

    # -- stage 2 engines --------------------------------------------------
    def _route_claude_cli(self, prompt: str) -> str | None:
        if self.config.simulate or not shutil.which(self.config.claude_bin):
            return None
        try:
            proc = subprocess.run(
                [self.config.claude_bin, "-p", prompt, "--model", self.config.router_model,
                 "--output-format", "text", "--max-turns", "1"],
                capture_output=True, text=True, timeout=60,
            )
        except (subprocess.TimeoutExpired, OSError):
            return None
        return proc.stdout if proc.returncode == 0 else None

    def _route_anthropic_api(self, prompt: str) -> str | None:
        key = os.environ.get("ANTHROPIC_API_KEY")
        if not key:
            return None
        body = json.dumps({
            "model": self.config.router_model,
            "max_tokens": 64,
            "messages": [{"role": "user", "content": prompt}],
        }).encode()
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={"content-type": "application/json", "x-api-key": key, "anthropic-version": "2023-06-01"},
        )
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                data = json.load(resp)
            return "".join(block.get("text", "") for block in data.get("content", []))
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError, OSError):
            return None

    def _route_ollama(self, prompt: str) -> str | None:
        body = json.dumps({"model": self.config.ollama_model, "prompt": prompt, "stream": False}).encode()
        req = urllib.request.Request(
            f"{self.config.ollama_url}/api/generate",
            data=body, headers={"content-type": "application/json"},
        )
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                return json.load(resp).get("response", "")
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError, OSError):
            return None

    _ENGINES = {
        "claude-cli": _route_claude_cli,
        "anthropic-api": _route_anthropic_api,
        "ollama": _route_ollama,
    }

    # -- stage 3: deterministic fallback ----------------------------------
    def _keyword_score(self, text: str) -> tuple[Skill | None, float]:
        tokens = set(re.findall(r"[a-z0-9']+", text.lower())) - _STOPWORDS
        if not tokens:
            return None, 0.0
        best, best_score = None, 0.0
        for skill in self.registry.all():
            hay = " ".join([skill.name.replace("-", " "), skill.description, " ".join(skill.triggers)])
            hay_tokens = set(re.findall(r"[a-z0-9']+", hay.lower())) - _STOPWORDS
            if not hay_tokens:
                continue
            overlap = tokens & hay_tokens
            score = len(overlap) / max(2, len(tokens))
            if score > best_score:
                best, best_score = skill, score
        return best, best_score

    # -- public API --------------------------------------------------------
    def route(self, text: str) -> RouteDecision:
        text = text.strip()
        if not text:
            return RouteDecision(CHAT, "empty", 0.0, text)

        skill = self.registry.match_triggers(text)
        if skill:
            return RouteDecision(skill.name, "regex", 1.0, text)

        valid = {s.name for s in self.registry.all()}
        prompt = _router_prompt(text, self.registry.all())
        for engine in self.config.router_engines:
            fn = self._ENGINES.get(engine)
            if not fn:
                continue
            raw = fn(self, prompt)
            if raw:
                name = _extract_skill_name(raw, valid)
                if name:
                    return RouteDecision(name, engine, 0.9, text)

        skill, score = self._keyword_score(text)
        if skill and score >= self.config.route_confidence_floor:
            return RouteDecision(skill.name, "keywords", score, text)
        return RouteDecision(CHAT, "keywords", score, text)


_STOPWORDS = {
    "a", "an", "and", "are", "can", "do", "for", "get", "give", "hey", "how", "i", "in",
    "is", "it", "me", "my", "of", "on", "please", "run", "show", "that", "the", "this",
    "to", "today", "todays", "up", "us", "what", "whats", "with", "you", "your",
}
