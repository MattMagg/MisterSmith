import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Optional


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "find_codex_session.py"


class FindCodexSessionScriptTests(unittest.TestCase):
    def run_script(
        self,
        session_root: Path,
        cwd: str,
        thread_id: Optional[str] = None,
    ) -> subprocess.CompletedProcess[str]:
        command = [
            "python3",
            str(SCRIPT_PATH),
            "--session-root",
            str(session_root),
            "--cwd",
            cwd,
        ]
        if thread_id is not None:
            command.extend(["--thread-id", thread_id])

        return subprocess.run(
            command,
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

    def write_session_meta(
        self,
        root: Path,
        relative_path: str,
        session_id: str,
        cwd: str,
    ) -> None:
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        payload = {"type": "session_meta", "payload": {"id": session_id, "cwd": cwd}}
        path.write_text(json.dumps(payload) + "\n", encoding="utf-8")

    def test_prefers_exact_thread_id_match_with_same_repo(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_cwd = "/repo/ms-38"
            self.write_session_meta(root, "2026/03/11/a.jsonl", "old-thread", repo_cwd)
            self.write_session_meta(root, "2026/03/11/b.jsonl", "wanted-thread", repo_cwd)

            result = self.run_script(root, repo_cwd, thread_id="wanted-thread")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str(root / "2026/03/11/b.jsonl"))

    def test_falls_back_to_newest_repo_match_when_thread_id_points_elsewhere(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_cwd = "/repo/ms-38"
            other_cwd = "/repo/other"
            self.write_session_meta(root, "2026/03/10/old.jsonl", "wanted-thread", other_cwd)
            self.write_session_meta(root, "2026/03/11/new.jsonl", "current-thread", repo_cwd)

            result = self.run_script(root, repo_cwd, thread_id="wanted-thread")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str(root / "2026/03/11/new.jsonl"))

    def test_returns_error_when_no_session_matches(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            self.write_session_meta(root, "2026/03/11/a.jsonl", "thread-a", "/repo/other")

            result = self.run_script(root, "/repo/ms-38", thread_id="missing-thread")

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("No Codex session file found", result.stderr)


if __name__ == "__main__":
    unittest.main()
