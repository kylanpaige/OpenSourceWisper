import unittest

from wisp.vault import Vault, parse_frontmatter, slugify

from .helpers import make_workspace


class VaultTests(unittest.TestCase):
    def setUp(self):
        self.tmp, self.config = make_workspace()
        self.vault = Vault(self.config.vault_dir)
        self.vault.ensure_layout()

    def tearDown(self):
        self.tmp.cleanup()

    def test_write_and_list_reports(self):
        report = self.vault.write_report("morning-report", "Morning Report", "body text", "all good")
        self.assertTrue(report.path.exists())
        reports = self.vault.reports()
        self.assertEqual(reports[0].title, "Morning Report")
        self.assertEqual(reports[0].skill, "morning-report")

    def test_duplicate_titles_get_unique_paths(self):
        r1 = self.vault.write_report("s", "Same Title", "a", "a")
        r2 = self.vault.write_report("s", "Same Title", "b", "b")
        self.assertNotEqual(r1.path, r2.path)

    def test_read_report_and_traversal_guard(self):
        report = self.vault.write_report("s", "Readable", "content here", "sum")
        found = self.vault.read_report(report.name)
        self.assertIsNotNone(found)
        meta, body = found
        self.assertEqual(meta["title"], "Readable")
        self.assertIn("content here", body)
        self.assertIsNone(self.vault.read_report("../Home"))
        self.assertIsNone(self.vault.read_report("nope"))

    def test_home_recent_reports_refreshed(self):
        self.vault.write_report("s", "Fresh Report", "b", "s")
        home = (self.vault.root / "Home.md").read_text()
        self.assertIn("Fresh Report", home)

    def test_directives_roundtrip(self):
        self.vault.set_directives(["One", "Two", "Three"])
        self.assertEqual(self.vault.directives(), ["One", "Two", "Three"])

    def test_schedule_parsing(self):
        schedule = self.vault.schedule()
        self.assertEqual(schedule[0], {"time": "09:00", "item": "Deep work"})

    def test_inbox_items(self):
        items = self.vault.inbox_items()
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0]["kind"], "sponsor")

    def test_trail_records_writes(self):
        self.vault.write_report("s", "Trail Check", "b", "s")
        trail = self.vault.trail()
        self.assertTrue(any("trail-check" in t["note"] for t in trail))

    def test_frontmatter_parser(self):
        meta, body = parse_frontmatter("---\ntitle: X\nskill: y\n---\n\nbody")
        self.assertEqual(meta, {"title": "X", "skill": "y"})
        self.assertEqual(body, "body")
        meta, body = parse_frontmatter("no frontmatter")
        self.assertEqual(meta, {})

    def test_slugify(self):
        self.assertEqual(slugify("Morning Report 2026!"), "morning-report-2026")
        self.assertEqual(slugify("***"), "note")


if __name__ == "__main__":
    unittest.main()
