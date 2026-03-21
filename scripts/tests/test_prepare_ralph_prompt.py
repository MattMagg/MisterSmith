import subprocess
import tempfile
import unittest
import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "prepare_ralph_prompt.py"


class PrepareRalphPromptScriptTests(unittest.TestCase):
    def run_script(
        self,
        input_path: str,
        output_path: Path,
        *sources: str,
        generated_at: str = "2026-03-21T02:15:00Z",
    ) -> subprocess.CompletedProcess[str]:
        command = [
            "python3",
            str(SCRIPT_PATH),
            "--input",
            input_path,
            "--output",
            str(output_path),
            "--generated-at",
            generated_at,
        ]
        for source in sources:
            command.extend(["--source", source])

        return subprocess.run(
            command,
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

    def run_packet_script(
        self,
        packet_path: Path,
        output_path: Path,
        *sources: str,
        generated_at: str = "2026-03-21T02:15:00Z",
    ) -> subprocess.CompletedProcess[str]:
        command = [
            "python3",
            str(SCRIPT_PATH),
            "--packet",
            str(packet_path),
            "--output",
            str(output_path),
            "--generated-at",
            generated_at,
        ]
        for source in sources:
            command.extend(["--source", source])

        return subprocess.run(
            command,
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_writes_metadata_and_preserves_body(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            prompt_input = temp_root / "prompt-input.md"
            source_one = temp_root / "issue.md"
            source_two = temp_root / "workpad.md"
            output_path = temp_root / "PROMPT.md"

            prompt_input.write_text("# Goal\n\nShip the helper.\n", encoding="utf-8")
            source_one.write_text("issue", encoding="utf-8")
            source_two.write_text("workpad", encoding="utf-8")

            result = self.run_script(
                str(prompt_input),
                output_path,
                str(source_one),
                str(source_two),
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            rendered = output_path.read_text(encoding="utf-8")
            self.assertEqual(
                rendered,
                "\n".join(
                    [
                        "<!-- ralph:generated-at: 2026-03-21T02:15:00Z -->",
                        f"<!-- ralph:source: {source_one.resolve()} -->",
                        f"<!-- ralph:source: {source_two.resolve()} -->",
                        "",
                        "# Goal",
                        "",
                        "Ship the helper.",
                        "",
                    ]
                ),
            )

    def test_replaces_existing_managed_metadata_block(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            prompt_input = temp_root / "prompt-input.md"
            output_path = temp_root / "PROMPT.md"

            prompt_input.write_text(
                "\n".join(
                    [
                        "<!-- ralph:generated-at: 2026-03-20T00:00:00Z -->",
                        "<!-- ralph:source: old-source.md -->",
                        "",
                        "Mode",
                        "Goal",
                        "",
                    ]
                ),
                encoding="utf-8",
            )

            result = self.run_script(str(prompt_input), output_path)

            self.assertEqual(result.returncode, 0, result.stderr)
            rendered = output_path.read_text(encoding="utf-8")
            self.assertEqual(
                rendered,
                "\n".join(
                    [
                        "<!-- ralph:generated-at: 2026-03-21T02:15:00Z -->",
                        f"<!-- ralph:source: {prompt_input.resolve()} -->",
                        "",
                        "Mode",
                        "Goal",
                        "",
                    ]
                ),
            )

    def test_records_repo_local_sources_as_repo_relative_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "PROMPT.md"

            result = self.run_script(
                str(REPO_ROOT / "PROMPT.md"),
                output_path,
                str(REPO_ROOT / "WORKFLOW.md"),
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            rendered = output_path.read_text(encoding="utf-8")
            lines = rendered.splitlines()
            self.assertEqual(lines[0], "<!-- ralph:generated-at: 2026-03-21T02:15:00Z -->")
            self.assertEqual(lines[1], "<!-- ralph:source: WORKFLOW.md -->")

    def test_requires_explicit_source_when_reading_from_stdin(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            output_path = Path(temp_dir) / "PROMPT.md"
            command = [
                "python3",
                str(SCRIPT_PATH),
                "--input",
                "-",
                "--output",
                str(output_path),
            ]

            result = subprocess.run(
                command,
                cwd=REPO_ROOT,
                text=True,
                input="Mode\nGoal\n",
                capture_output=True,
                check=False,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("At least one --source is required", result.stderr)

    def test_packet_mode_uses_rendered_prompt_and_source_docs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            issue_path = temp_root / "issue.md"
            workpad_path = temp_root / "workpad.md"
            packet_path = temp_root / "packet.json"
            output_path = temp_root / "PROMPT.md"

            issue_path.write_text("issue", encoding="utf-8")
            workpad_path.write_text("workpad", encoding="utf-8")
            packet_path.write_text(
                json.dumps(
                    {
                        "rendered_prompt": "# Ralph Packet\n\nUse the active issue.\n",
                        "source_docs": [str(issue_path), str(workpad_path)],
                    }
                ),
                encoding="utf-8",
            )

            result = self.run_packet_script(packet_path, output_path)

            self.assertEqual(result.returncode, 0, result.stderr)
            rendered = output_path.read_text(encoding="utf-8")
            self.assertEqual(
                rendered,
                "\n".join(
                    [
                        "<!-- ralph:generated-at: 2026-03-21T02:15:00Z -->",
                        f"<!-- ralph:source: {packet_path.resolve()} -->",
                        f"<!-- ralph:source: {issue_path.resolve()} -->",
                        f"<!-- ralph:source: {workpad_path.resolve()} -->",
                        "",
                        "# Ralph Packet",
                        "",
                        "Use the active issue.",
                        "",
                    ]
                ),
            )

    def test_packet_mode_accepts_smith_response_envelope(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            issue_path = temp_root / "issue.md"
            packet_path = temp_root / "packet.json"
            output_path = temp_root / "PROMPT.md"

            issue_path.write_text("issue", encoding="utf-8")
            packet_path.write_text(
                json.dumps(
                    {
                        "status": "ok",
                        "data": {
                            "rendered_prompt": "Mode\n\nGoal\n",
                            "source_docs": [str(issue_path)],
                        },
                    }
                ),
                encoding="utf-8",
            )

            result = self.run_packet_script(packet_path, output_path)

            self.assertEqual(result.returncode, 0, result.stderr)
            rendered = output_path.read_text(encoding="utf-8")
            lines = rendered.splitlines()
            self.assertEqual(lines[0], "<!-- ralph:generated-at: 2026-03-21T02:15:00Z -->")
            self.assertEqual(lines[1], f"<!-- ralph:source: {packet_path.resolve()} -->")
            self.assertEqual(lines[2], f"<!-- ralph:source: {issue_path.resolve()} -->")
            self.assertEqual(lines[4], "Mode")


if __name__ == "__main__":
    unittest.main()
