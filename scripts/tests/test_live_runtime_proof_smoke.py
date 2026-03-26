import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "scripts" / "live_runtime_proof_smoke.py"


def load_module():
    spec = importlib.util.spec_from_file_location("live_runtime_proof_smoke", SCRIPT_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError("failed to load live_runtime_proof_smoke module")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class LiveRuntimeProofSmokeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.module = load_module()

    def test_build_artifact_dir_creates_timestamped_run_folder(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            artifact_dir = self.module.build_artifact_dir(root, "20260326T153000Z")

            self.assertEqual(artifact_dir, root / "20260326T153000Z")
            self.assertTrue(artifact_dir.is_dir())

    def test_redaction_helpers_mask_email_and_password(self) -> None:
        auth_status = "Authenticated ChatGPT account: ops@example.com (pro)"
        database_url = "postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/ms_test"

        self.assertEqual(
            self.module.redact_email_addresses(auth_status),
            "Authenticated ChatGPT account: <redacted-email> (pro)",
        )
        self.assertEqual(
            self.module.redact_database_url(database_url),
            "postgres://mistersmith:<redacted>@127.0.0.1:5432/ms_test",
        )

    def test_summarize_task_status_extracts_runtime_markers_and_step_summaries(self) -> None:
        task_status = {
            "task_id": "workflow-1",
            "status": "completed",
            "result": {
                "proof_outcome": "graph_formed_and_completed",
                "result": {
                    "provider_kind": "openai_chatgpt",
                    "model_id": "gpt-5.4",
                    "proof_outcome": "graph_formed_and_completed",
                    "runtime_execution_mode": {
                        "workflow_runner": "tokio_task",
                        "planner_lifecycle": "supervised_actor",
                        "executor_lifecycle": "supervised_actor",
                        "execution_boundary": "tool_bus",
                        "tool_name": "workflow.execute_step",
                        "provider_kind": "openai_chatgpt",
                        "model_id": "gpt-5.4",
                    },
                    "aggregated_result": [{}, {}, {}],
                    "step_results": [
                        {
                            "task_id": "step-1",
                            "worker_id": "worker-1",
                            "action": "trace startup",
                            "result": {
                                "execution_boundary": "tool_bus",
                                "tool_name": "workflow.execute_step",
                            },
                        }
                    ],
                },
            },
        }

        summary = self.module.summarize_task_status(task_status)

        self.assertEqual(summary["provider_kind"], "openai_chatgpt")
        self.assertEqual(summary["model_id"], "gpt-5.4")
        self.assertEqual(summary["proof_outcome"], "graph_formed_and_completed")
        self.assertEqual(summary["step_result_count"], 1)
        self.assertEqual(summary["aggregated_result_count"], 3)
        self.assertEqual(summary["step_summaries"][0]["task_id"], "step-1")
        self.assertEqual(summary["step_summaries"][0]["tool_name"], "workflow.execute_step")

    def test_assert_task_summary_rejects_missing_tool_bus_marker(self) -> None:
        summary = {
            "status": "completed",
            "provider_kind": "openai_chatgpt",
            "model_id": "gpt-5.4",
            "runtime_execution_mode": {
                "workflow_runner": "tokio_task",
                "planner_lifecycle": "supervised_actor",
                "executor_lifecycle": "supervised_actor",
                "execution_boundary": "tool_bus",
                "tool_name": "workflow.execute_step",
                "provider_kind": "openai_chatgpt",
                "model_id": "gpt-5.4",
            },
            "step_summaries": [
                {
                    "execution_boundary": "http",
                    "tool_name": "workflow.execute_step",
                }
            ],
        }

        with self.assertRaises(self.module.SmokeHarnessError):
            self.module.assert_task_summary(
                summary,
                expected_provider_kind="openai_chatgpt",
                expected_model_id="gpt-5.4",
            )

    def test_assert_autonomy_status_requires_completed_graph_and_histories(self) -> None:
        payload = {
            "graph": {
                "workflow_id": "workflow-1",
                "state": "Completed",
                "branch_count": 2,
                "node_count": 2,
            },
            "topology": {"parallelism_width": 2},
            "routing_history": [{}],
            "step_routing_history": [{}],
            "branches": [{}, {}],
        }

        self.module.assert_autonomy_status(payload, "workflow-1")

        payload["step_routing_history"] = []
        with self.assertRaises(self.module.SmokeHarnessError):
            self.module.assert_autonomy_status(payload, "workflow-1")

    def test_assert_runtime_log_markers_requires_all_expected_markers(self) -> None:
        complete_log = "\n".join(self.module.REQUIRED_RUNTIME_LOG_MARKERS)
        self.module.assert_runtime_log_markers(complete_log)

        with self.assertRaises(self.module.SmokeHarnessError):
            self.module.assert_runtime_log_markers("Runtime task execution service ready")


if __name__ == "__main__":
    unittest.main()
