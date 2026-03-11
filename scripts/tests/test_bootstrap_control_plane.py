import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "bootstrap_control_plane.py"
CANONICAL_SKILLS = [
    "mister-smith-control-plane-router",
    "mister-smith-control-plane-bootstrap",
    "symphony-linear-mister-smith",
    "stage-mister-smith-phase",
    "symphony-mister-smith-review-dispatch",
    "mister-smith-frontier-mandate",
]


class BootstrapControlPlaneScriptTests(unittest.TestCase):
    def run_script(self, repo_root: Path, config_path: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                "python3",
                str(SCRIPT_PATH),
                "--repo-root",
                str(repo_root),
                "--config-path",
                str(config_path),
            ],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_creates_missing_repo_local_skill_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "mister-smith"
            repo_root.mkdir()
            config_path = repo_root / "codex.toml"
            config_path.write_text("[mcp_servers.smith]\nenabled = true\n", encoding="utf-8")

            result = self.run_script(repo_root, config_path)
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["config"]["server_name"], "smith")
            self.assertEqual(sorted(payload["skills"]["created"]), sorted(CANONICAL_SKILLS))
            for skill_name in CANONICAL_SKILLS:
                skill_path = repo_root / ".codex" / "skills" / skill_name / "SKILL.md"
                self.assertTrue(skill_path.exists(), f"missing {skill_path}")
                text = skill_path.read_text(encoding="utf-8")
                self.assertIn("Use the `smith` MCP tools first", text)

    def test_accepts_legacy_control_plane_server_name(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            repo_root = Path(temp_dir) / "mister-smith"
            repo_root.mkdir()
            config_path = repo_root / "codex.toml"
            config_path.write_text(
                "[mcp_servers.mistersmith_control_plane]\nenabled = true\n",
                encoding="utf-8",
            )

            result = self.run_script(repo_root, config_path)
            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertEqual(payload["config"]["server_name"], "mistersmith_control_plane")
            self.assertFalse(payload["config"]["missing"])


if __name__ == "__main__":
    unittest.main()
