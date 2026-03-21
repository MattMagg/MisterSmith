import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "validate_ralph_prompt.py"


class ValidateRalphPromptScriptTests(unittest.TestCase):
    def run_validate(
        self,
        prompt_path: Path,
        repo_root: Path,
        command_name: str,
        *command_args: str,
    ) -> subprocess.CompletedProcess[str]:
        command = [
            "python3",
            str(SCRIPT_PATH),
            command_name,
            "--prompt-path",
            str(prompt_path),
            "--repo-root",
            str(repo_root),
            *command_args,
        ]
        return subprocess.run(
            command,
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_validate_passes_for_repo_relative_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            source_dir = temp_root / "inputs"
            source_dir.mkdir()
            source_path = source_dir / "issue.md"
            prompt_path = temp_root / "PROMPT.md"

            source_path.write_text("issue context\n", encoding="utf-8")
            prompt_path.write_text(
                "\n".join(
                    [
                        "<!-- ralph:generated-at: 2099-01-01T00:00:00Z -->",
                        "<!-- ralph:source: inputs/issue.md -->",
                        "",
                        "Mode",
                        "Goal",
                        "",
                    ]
                ),
                encoding="utf-8",
            )

            result = self.run_validate(prompt_path, temp_root, "validate")

            self.assertEqual(result.returncode, 0, result.stderr)

    def test_validate_fails_when_metadata_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            prompt_path = temp_root / "PROMPT.md"
            prompt_path.write_text("Mode\nGoal\n", encoding="utf-8")

            result = self.run_validate(prompt_path, temp_root, "validate")

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("generated-at line", result.stderr)

    def test_validate_fails_when_source_is_newer_than_generated_at(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            source_path = temp_root / "issue.md"
            prompt_path = temp_root / "PROMPT.md"

            source_path.write_text("issue context\n", encoding="utf-8")
            os.utime(source_path, (1760000050, 1760000050))
            prompt_path.write_text(
                "\n".join(
                    [
                        "<!-- ralph:generated-at: 2025-10-09T08:53:20Z -->",
                        f"<!-- ralph:source: {source_path} -->",
                        "",
                        "Mode",
                        "",
                    ]
                ),
                encoding="utf-8",
            )

            result = self.run_validate(prompt_path, temp_root, "validate")

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Prompt is stale", result.stderr)

    def test_guard_run_skips_explicit_prompt_override(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            prompt_path = temp_root / "PROMPT.md"
            prompt_path.write_text("Mode\nGoal\n", encoding="utf-8")

            result = self.run_validate(
                prompt_path,
                temp_root,
                "guard-run",
                "--",
                "run",
                "--prompt",
                "inline prompt",
                "--dry-run",
            )

            self.assertEqual(result.returncode, 0, result.stderr)

    def test_guard_run_skips_nondefault_prompt_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            default_prompt_path = temp_root / "PROMPT.md"
            other_prompt_path = temp_root / "other.md"
            default_prompt_path.write_text("Mode\nGoal\n", encoding="utf-8")
            other_prompt_path.write_text("Different prompt\n", encoding="utf-8")

            result = self.run_validate(
                default_prompt_path,
                temp_root,
                "guard-run",
                "--",
                "run",
                "--prompt-file",
                str(other_prompt_path),
                "--dry-run",
            )

            self.assertEqual(result.returncode, 0, result.stderr)

    def test_guard_run_validates_default_run_prompt(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            source_path = temp_root / "issue.md"
            prompt_path = temp_root / "PROMPT.md"

            source_path.write_text("issue context\n", encoding="utf-8")
            prompt_path.write_text(
                "\n".join(
                    [
                        "<!-- ralph:generated-at: 2099-01-01T00:00:00Z -->",
                        f"<!-- ralph:source: {source_path} -->",
                        "",
                        "Mode",
                        "",
                    ]
                ),
                encoding="utf-8",
            )

            result = self.run_validate(
                prompt_path,
                temp_root,
                "guard-run",
                "--",
                "run",
                "--dry-run",
            )

            self.assertEqual(result.returncode, 0, result.stderr)

    def test_guard_run_skips_help_requests(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            prompt_path = temp_root / "PROMPT.md"
            prompt_path.write_text("Mode\nGoal\n", encoding="utf-8")

            result = self.run_validate(
                prompt_path,
                temp_root,
                "guard-run",
                "--",
                "run",
                "--help",
            )

            self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
