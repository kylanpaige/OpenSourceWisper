import json
import time
import unittest
import urllib.error
import urllib.request

from wisp.server import serve

from .helpers import make_workspace


def _get(url: str):
    with urllib.request.urlopen(url, timeout=10) as resp:
        return json.load(resp)


def _post(url: str, payload: dict):
    req = urllib.request.Request(url, data=json.dumps(payload).encode(),
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=10) as resp:
        return json.load(resp)


class ServerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp, cls.config = make_workspace()
        cls.config.port = 0  # pick a free port
        cls.server = serve(cls.config, block=False)
        cls.base = f"http://127.0.0.1:{cls.server.server_address[1]}"

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.tmp.cleanup()

    def _wait_done(self, job_id: int, timeout=15):
        app = self.server.wisp_app
        deadline = time.time() + timeout
        while time.time() < deadline:
            job = app.jobs.get(job_id)
            if job and job.status in ("done", "error"):
                return job
            time.sleep(0.1)
        self.fail("job did not finish")

    def test_hud_served(self):
        with urllib.request.urlopen(self.base + "/", timeout=10) as resp:
            html = resp.read().decode()
        self.assertIn("Wisp", html)
        self.assertIn("text/html", resp.headers["Content-Type"])

    def test_state_shape(self):
        state = _get(self.base + "/api/state")
        for key in ("assistant", "vitals", "directives", "schedule", "skills", "jobs", "reports", "trail"):
            self.assertIn(key, state)
        self.assertGreaterEqual(state["vitals"]["skills"], 3)

    def test_run_skill_endpoint_and_report_fetch(self):
        res = _post(self.base + "/api/skills/inbox-brief/run", {})
        job = self._wait_done(res["job"]["id"])
        self.assertEqual(job.status, "done")
        report = _get(f"{self.base}/api/reports/{job.report}")
        self.assertIn("Sponsorship", report["body"])

    def test_run_unknown_skill_404(self):
        with self.assertRaises(urllib.error.HTTPError) as ctx:
            _post(self.base + "/api/skills/nope/run", {})
        self.assertEqual(ctx.exception.code, 404)

    def test_say_routes_to_skill(self):
        res = _post(self.base + "/api/say", {"text": "give me the rundown"})
        self.assertEqual(res["decision"]["skill"], "morning-report")
        job = self._wait_done(res["job"]["id"])
        self.assertEqual(job.status, "done")

    def test_say_chat_fallback(self):
        res = _post(self.base + "/api/say", {"text": "what's the weather like on Mars"})
        self.assertIn("reply", res)

    def test_say_requires_text(self):
        with self.assertRaises(urllib.error.HTTPError) as ctx:
            _post(self.base + "/api/say", {})
        self.assertEqual(ctx.exception.code, 400)


if __name__ == "__main__":
    unittest.main()
