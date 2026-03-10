import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "validate_deploy_assets.py"


class ValidateDeployAssetsScriptTests(unittest.TestCase):
    def run_script(self, *paths: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", str(SCRIPT_PATH), *paths],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_valid_repository_assets_pass(self) -> None:
        result = self.run_script(
            "deploy/dashboards/mister-smith-autonomy.json",
            "deploy/alerts/mister-smith-autonomy-rules.yml",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("json ok: deploy/dashboards/mister-smith-autonomy.json", result.stdout)
        self.assertIn("yaml ok: deploy/alerts/mister-smith-autonomy-rules.yml", result.stdout)

    def test_invalid_json_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            invalid_json = Path(temp_dir) / "broken.json"
            invalid_json.write_text("{not valid json", encoding="utf-8")

            result = self.run_script(str(invalid_json))

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid json", result.stderr)

    def test_invalid_yaml_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            invalid_yaml = Path(temp_dir) / "broken.yml"
            invalid_yaml.write_text("groups: [broken", encoding="utf-8")

            result = self.run_script(str(invalid_yaml))

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid yaml", result.stderr)


if __name__ == "__main__":
    unittest.main()
