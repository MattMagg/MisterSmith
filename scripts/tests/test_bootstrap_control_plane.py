import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from scripts.bootstrap_control_plane import (
    BootstrapPaths,
    LEGACY_SKILLS,
    REPO_ROOT,
    render_config_block,
    render_skill_shim,
    run_bootstrap,
    upsert_managed_block,
)


SCRIPT_PATH = REPO_ROOT / "scripts" / "bootstrap_control_plane.py"


class BootstrapControlPlaneTests(unittest.TestCase):
    def test_upsert_managed_block_is_idempotent(self) -> None:
        paths = BootstrapPaths(
            codex_home=Path("/tmp/.codex"),
            config_path=Path("/tmp/.codex/config.toml"),
            control_plane_repo=Path("/tmp/control-plane"),
        )
        initial = '[mcp_servers.context7]\ncommand = "npx"\n'
        block = render_config_block(paths)

        once = upsert_managed_block(initial, block)
        twice = upsert_managed_block(once, block)

        self.assertEqual(once, twice)
        self.assertIn("[mcp_servers.mistersmith_control_plane]", twice)
        self.assertEqual(twice.count("[mcp_servers.mistersmith_control_plane]"), 1)

    def test_render_skill_shim_points_to_repo_local_canonical_skill(self) -> None:
        shim = render_skill_shim(
            "symphony-linear-mister-smith",
            LEGACY_SKILLS["symphony-linear-mister-smith"],
        )

        canonical = REPO_ROOT / ".codex" / "skills" / "symphony-linear-mister-smith" / "SKILL.md"
        self.assertIn(str(canonical), shim)
        self.assertIn("migration shim", shim)

    def test_run_bootstrap_dry_run_reports_shim_and_config_changes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            codex_home = Path(temp_dir) / ".codex"
            control_plane_repo = Path(temp_dir) / "control-plane"
            (codex_home / "skills").mkdir(parents=True)
            (control_plane_repo / "node_modules" / ".bin").mkdir(parents=True)
            (control_plane_repo / "src").mkdir(parents=True)
            (control_plane_repo / "node_modules" / ".bin" / "tsx").write_text("", encoding="utf-8")
            (control_plane_repo / "src" / "server.ts").write_text("", encoding="utf-8")
            router_skill = REPO_ROOT / ".codex" / "skills" / "mister-smith-control-plane-router" / "SKILL.md"
            self.assertTrue(router_skill.exists(), router_skill)

            result = run_bootstrap(
                BootstrapPaths(
                    codex_home=codex_home,
                    config_path=codex_home / "config.toml",
                    control_plane_repo=control_plane_repo,
                ),
                dry_run=True,
            )

        self.assertTrue(result["config_changed"])
        self.assertEqual(len(result["shim_changes"]), len(LEGACY_SKILLS))
        self.assertEqual(result["problems"], [])

    def test_cli_outputs_json_summary(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            codex_home = Path(temp_dir) / ".codex"
            control_plane_repo = Path(temp_dir) / "control-plane"
            (codex_home / "skills").mkdir(parents=True)
            (control_plane_repo / "node_modules" / ".bin").mkdir(parents=True)
            (control_plane_repo / "src").mkdir(parents=True)
            (control_plane_repo / "node_modules" / ".bin" / "tsx").write_text("", encoding="utf-8")
            (control_plane_repo / "src" / "server.ts").write_text("", encoding="utf-8")

            router_skill = REPO_ROOT / ".codex" / "skills" / "mister-smith-control-plane-router" / "SKILL.md"
            self.assertTrue(router_skill.exists(), router_skill)

            result = subprocess.run(
                [
                    "python3",
                    str(SCRIPT_PATH),
                    "--dry-run",
                    "--codex-home",
                    str(codex_home),
                    "--control-plane-repo",
                    str(control_plane_repo),
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertTrue(payload["dry_run"])
        self.assertEqual(payload["control_plane_repo"], str(control_plane_repo))


if __name__ == "__main__":
    unittest.main()
