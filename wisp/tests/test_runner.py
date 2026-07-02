import json
import unittest

from wisp.runner import Dispatcher, _parse_report_json
from wisp.skills import SkillRegistry
from wisp.vault import Vault

from .helpers import make_workspace


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.tmp, self.config = make_workspace()
        self.registry = SkillRegistry(self.config.skills_dir)
        self.vault = Vault(self.config.vault_dir)
        self.vault.ensure_layout()
        self.dispatcher = Dispatcher(self.config, self.registry, self.vault)

    def tearDown(self):
        self.tmp.cleanup()

    def test_simulate_mode_disables_claude(self):
        self.assertFalse(self.dispatcher.claude.available())

    def test_builtin_run_writes_report_and_journal(self):
        outcome = self.dispatcher.run_skill("inbox-brief")
        self.assertTrue(outcome.ok)
        self.assertEqual(outcome.engine, "builtin")
        self.assertIn("1 threads", outcome.summary)
        self.assertTrue(outcome.report.path.exists())
        journal = (self.vault.root / "log" / "runs.jsonl").read_text().splitlines()
        self.assertEqual(json.loads(journal[-1])["skill"], "inbox-brief")

    def test_claude_skill_falls_back_to_stub_in_simulate(self):
        # 'research' declares runner: claude and has no builtin — in simulate
        # mode it must degrade to a stub report, never crash.
        outcome = self.dispatcher.run_skill("research", "what is VAD?")
        self.assertEqual(outcome.engine, "stub")
        self.assertFalse(outcome.ok)
        self.assertTrue(outcome.report.path.exists())

    def test_unknown_skill(self):
        outcome = self.dispatcher.run_skill("does-not-exist")
        self.assertFalse(outcome.ok)
        self.assertIsNone(outcome.report)

    def test_morning_report_uses_vault_directives(self):
        outcome = self.dispatcher.run_skill("morning-report")
        self.assertTrue(outcome.ok)
        _, body = self.vault.read_report(outcome.report.name)
        self.assertIn("Ship the HUD", body)

    def test_chat_fallback_without_engine(self):
        reply = self.dispatcher.chat("hello there")
        self.assertIn("morning-report", reply)

    def test_parse_report_json_variants(self):
        title, summary, body = _parse_report_json(
            'preamble {"title": "T", "summary": "S", "body": "B"} epilogue', "x")
        self.assertEqual((title, summary, body), ("T", "S", "B"))
        title, summary, body = _parse_report_json("plain text answer", "x")
        self.assertTrue(title.startswith("x "))
        self.assertEqual(summary, "plain text answer")


if __name__ == "__main__":
    unittest.main()
