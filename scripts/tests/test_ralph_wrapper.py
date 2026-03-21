import json
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "ralph"


class RalphWrapperTests(unittest.TestCase):
    def test_prompt_subcommand_renders_from_packet(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_root = Path(temp_dir)
            packet_path = temp_root / "packet.json"
            output_path = temp_root / "PROMPT.md"
            issue_path = temp_root / "issue.md"

            issue_path.write_text("issue", encoding="utf-8")
            packet_path.write_text(
                json.dumps(
                    {
                        "data": {
                            "rendered_prompt": "# Ralph Packet\n\nShip the slice.\n",
                            "source_docs": [str(issue_path)],
                        }
                    }
                ),
                encoding="utf-8",
            )

            result = subprocess.run(
                [
                    str(SCRIPT_PATH),
                    "prompt",
                    "--packet",
                    str(packet_path),
                    "--output",
                    str(output_path),
                    "--generated-at",
                    "2026-03-21T02:15:00Z",
                ],
                cwd=REPO_ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            rendered = output_path.read_text(encoding="utf-8")
            self.assertIn("# Ralph Packet", rendered)
            self.assertIn("Ship the slice.", rendered)
            self.assertIn(str(packet_path.resolve()), rendered)


if __name__ == "__main__":
    unittest.main()
