import unittest

from wisp.skills import SkillRegistry

from .helpers import make_workspace


class SkillRegistryTests(unittest.TestCase):
    def setUp(self):
        self.tmp, self.config = make_workspace()
        self.registry = SkillRegistry(self.config.skills_dir)

    def tearDown(self):
        self.tmp.cleanup()

    def test_loads_all_manifests(self):
        names = {s.name for s in self.registry.all()}
        self.assertEqual(names, {"morning-report", "inbox-brief", "research"})

    def test_manifest_fields(self):
        skill = self.registry.get("morning-report")
        self.assertEqual(skill.runner, "builtin")
        self.assertEqual(skill.builtin, "morning_report")
        self.assertEqual(skill.hud_group, "briefing")
        self.assertTrue(skill.voice_reply)
        self.assertIn("rundown", skill.triggers)
        self.assertIn("morning report", skill.triggers)
        self.assertIn("Build the morning report", skill.body)

    def test_trigger_matching_prefers_longest_match(self):
        skill = self.registry.match_triggers("please give me the morning report rundown")
        self.assertEqual(skill.name, "morning-report")

    def test_missing_dir_is_empty_registry(self):
        registry = SkillRegistry(self.config.skills_dir / "nope")
        self.assertEqual(registry.all(), [])

    def test_to_json_shape(self):
        data = self.registry.get("research").to_json()
        self.assertEqual(data["runner"], "claude")
        self.assertEqual(data["group"], "knowledge")


if __name__ == "__main__":
    unittest.main()
