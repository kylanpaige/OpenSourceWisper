import unittest

from wisp.router import CHAT, Router, _extract_skill_name
from wisp.skills import SkillRegistry

from .helpers import make_workspace


class RouterTests(unittest.TestCase):
    def setUp(self):
        self.tmp, self.config = make_workspace()
        self.registry = SkillRegistry(self.config.skills_dir)
        self.router = Router(self.config, self.registry)

    def tearDown(self):
        self.tmp.cleanup()

    def test_regex_trigger_wins_instantly(self):
        decision = self.router.route("Hey Wisp, give me the rundown for today")
        self.assertEqual(decision.skill, "morning-report")
        self.assertEqual(decision.engine, "regex")
        self.assertEqual(decision.confidence, 1.0)

    def test_multiword_trigger(self):
        decision = self.router.route("could you triage my inbox please")
        self.assertEqual(decision.skill, "inbox-brief")
        self.assertEqual(decision.engine, "regex")

    def test_keyword_fallback(self):
        # No trigger phrase, but token overlap with the skill description.
        decision = self.router.route("summarize the inbox for me")
        self.assertEqual(decision.skill, "inbox-brief")
        self.assertEqual(decision.engine, "keywords")

    def test_chat_when_nothing_matches(self):
        decision = self.router.route("what's the weather like on Mars")
        self.assertEqual(decision.skill, CHAT)

    def test_empty_input(self):
        decision = self.router.route("   ")
        self.assertEqual(decision.skill, CHAT)
        self.assertEqual(decision.engine, "empty")

    def test_trigger_is_case_insensitive_and_word_bounded(self):
        self.assertEqual(self.router.route("RUNDOWN please").skill, "morning-report")
        # "groundwork" contains no word-bounded "rundown"
        self.assertNotEqual(self.router.route("lay the groundwork").skill, "morning-report")

    def test_extract_skill_name_from_json(self):
        valid = {"morning-report", "inbox-brief"}
        self.assertEqual(_extract_skill_name('{"skill": "inbox-brief"}', valid), "inbox-brief")
        self.assertEqual(_extract_skill_name('noise {"skill": "morning-report"} noise', valid), "morning-report")
        self.assertEqual(_extract_skill_name("I'd pick inbox-brief here", valid), "inbox-brief")
        self.assertIsNone(_extract_skill_name("no idea", valid))


if __name__ == "__main__":
    unittest.main()
