"""Background job queue: queued → running → done, with SSE fan-out.

The HUD's skill buttons enqueue here; a single worker thread executes runs so
skill executions never block the HTTP server, and every state change is pushed
to connected HUD clients ("it mentions it's queued right away … 'Running' …
then we see a new pop-up").
"""

from __future__ import annotations

import datetime as _dt
import itertools
import json
import queue
import threading
from dataclasses import dataclass, field


@dataclass
class Job:
    id: int
    skill: str
    text: str = ""
    status: str = "queued"  # queued | running | done | error
    engine: str = ""
    summary: str = ""
    report: str | None = None
    created: str = field(default_factory=lambda: _dt.datetime.now().isoformat(timespec="seconds"))
    finished: str | None = None

    def to_json(self) -> dict:
        return {
            "id": self.id, "skill": self.skill, "status": self.status, "engine": self.engine,
            "summary": self.summary, "report": self.report, "created": self.created,
            "finished": self.finished,
        }


class JobQueue:
    def __init__(self, dispatcher):
        self.dispatcher = dispatcher
        self._jobs: dict[int, Job] = {}
        self._ids = itertools.count(1)
        self._pending: queue.Queue[int] = queue.Queue()
        self._listeners: list[queue.Queue] = []
        self._lock = threading.Lock()
        self._worker = threading.Thread(target=self._loop, daemon=True, name="wisp-jobs")
        self._worker.start()

    # -- API ----------------------------------------------------------------
    def enqueue(self, skill: str, text: str = "") -> Job:
        job = Job(id=next(self._ids), skill=skill, text=text)
        with self._lock:
            self._jobs[job.id] = job
        self._pending.put(job.id)
        self._emit("job", job.to_json())
        return job

    def jobs(self, limit: int = 20) -> list[Job]:
        with self._lock:
            return sorted(self._jobs.values(), key=lambda j: j.id, reverse=True)[:limit]

    def get(self, job_id: int) -> Job | None:
        with self._lock:
            return self._jobs.get(job_id)

    # -- SSE ------------------------------------------------------------------
    def subscribe(self) -> queue.Queue:
        q: queue.Queue = queue.Queue(maxsize=100)
        with self._lock:
            self._listeners.append(q)
        return q

    def unsubscribe(self, q: queue.Queue) -> None:
        with self._lock:
            if q in self._listeners:
                self._listeners.remove(q)

    def _emit(self, kind: str, payload: dict) -> None:
        msg = json.dumps({"kind": kind, **payload})
        with self._lock:
            listeners = list(self._listeners)
        for q in listeners:
            try:
                q.put_nowait(msg)
            except queue.Full:
                pass

    # -- worker ----------------------------------------------------------------
    def _loop(self) -> None:
        while True:
            job_id = self._pending.get()
            job = self.get(job_id)
            if not job:
                continue
            job.status = "running"
            self._emit("job", job.to_json())
            try:
                outcome = self.dispatcher.run_skill(job.skill, job.text)
                job.engine = outcome.engine
                job.summary = outcome.summary
                job.report = outcome.report.name if outcome.report else None
                job.status = "done" if outcome.ok else "error"
            except Exception as exc:  # noqa: BLE001 — worker must never die
                job.status = "error"
                job.summary = f"{type(exc).__name__}: {exc}"
            job.finished = _dt.datetime.now().isoformat(timespec="seconds")
            self._emit("job", job.to_json())
