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

    def test_build_runtime_config_toml_for_budget_profile_contains_cascade_and_budget_root(self) -> None:
        config_text = self.module.build_runtime_config_toml(
            self.module.DEFAULT_BUDGET_AWARE_PROFILE
        )

        self.assertIsNotNone(config_text)
        self.assertIn('policy = "cascade"', config_text)
        self.assertIn('budget_root = "runtime.task_path"', config_text)
        self.assertIn('provider_kind = "openai_chatgpt"', config_text)
        self.assertIn('provider_kind = "mock"', config_text)

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
                        "routing_policy": "round_robin",
                        "registered_provider_count": 1,
                        "budget_root": "disabled",
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
        self.assertEqual(summary["routing_policy"], "round_robin")
        self.assertEqual(summary["registered_provider_count"], 1)
        self.assertEqual(summary["budget_root"], "disabled")
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

    def test_assert_task_summary_accepts_budget_profile_runtime_markers(self) -> None:
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
                "routing_policy": "cascade",
                "registered_provider_count": 2,
                "budget_root": "runtime.task_path",
            },
            "step_summaries": [
                {
                    "execution_boundary": "tool_bus",
                    "tool_name": "workflow.execute_step",
                }
            ],
        }

        self.module.assert_task_summary(
            summary,
            expected_provider_kind="openai_chatgpt",
            expected_model_id="gpt-5.4",
            expected_routing_policy="cascade",
            expected_registered_provider_count=2,
            expected_budget_root="runtime.task_path",
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

    def test_assert_autonomy_step_routing_expectations_checks_budget_checkpoint(self) -> None:
        payload = {
            "step_routing_history": [
                {
                    "tier": "primary",
                    "action": "downgrade",
                    "triggered_checkpoints": ["budget_policy", "confidence_review"],
                }
            ]
        }

        self.module.assert_autonomy_step_routing_expectations(
            payload,
            expected_action="downgrade",
            expected_tier="primary",
            required_checkpoints=("budget_policy",),
        )

        with self.assertRaises(self.module.SmokeHarnessError):
            self.module.assert_autonomy_step_routing_expectations(
                payload,
                expected_action="fallback",
                expected_tier="primary",
                required_checkpoints=("budget_policy",),
            )

    def test_assert_supervision_surfaces_requires_packet021_fields_and_consistency(self) -> None:
        task_status = {
            "result": {
                "supervision_evidence": {
                    "target_scope": {
                        "kind": "branch",
                        "graph_id": "graph-1",
                        "branch_id": "branch-1",
                    },
                    "decision_basis": "fingerprint_reinforced",
                    "proof_boundary": "supported task path only",
                    "fingerprint_ref": {
                        "fingerprint_key": "executor:branch-1",
                    },
                    "repair_lineage_ref": {
                        "source": "packet-020",
                        "checkpoint_ref": "checkpoint-retry",
                    },
                }
            }
        }
        autonomy_status = {
            "supervision_evidence": {
                "target_scope": {
                    "kind": "branch",
                    "graph_id": "graph-1",
                    "branch_id": "branch-1",
                },
                "decision_basis": "fingerprint_reinforced",
                "proof_boundary": "supported task path only",
                "fingerprint_ref": {
                    "fingerprint_key": "executor:branch-1",
                },
                "repair_lineage_ref": {
                    "source": "packet-020",
                    "checkpoint_ref": "checkpoint-retry",
                },
            }
        }

        self.module.assert_supervision_surfaces(
            task_status,
            autonomy_status,
            allowed_target_kinds=("branch", "node"),
            require_decision_basis=True,
            require_proof_boundary=True,
            require_detailed_payload=True,
            require_consistency=True,
        )

        autonomy_status["supervision_evidence"]["decision_basis"] = "live_signals_only"
        with self.assertRaises(self.module.SmokeHarnessError):
            self.module.assert_supervision_surfaces(
                task_status,
                autonomy_status,
                allowed_target_kinds=("branch", "node"),
                require_decision_basis=True,
                require_proof_boundary=True,
                require_detailed_payload=True,
                require_consistency=True,
            )

    def test_assert_runtime_truth_surfaces_requires_packet023_fields_and_consistency(self) -> None:
        task_status = {
            "result": {
                "runtime_truth": {
                    "evidence_class": "placeholder_or_simulated_step_completion",
                    "proof_boundary": {
                        "graph_execution": "workflow graph executed successfully",
                        "semantic_completion": "semantic completion not yet proven",
                        "grounded_tool_execution": "grounded tool execution: none/minimal",
                        "task_proof": "result is orchestration proof, not substantive task proof",
                    },
                    "run_trace": {
                        "trace_root_id": "workflow-1",
                        "workflow_id": "workflow-1",
                        "relationships": ["graph", "tool_boundary", "supervision"],
                    },
                    "grounded_evidence": [],
                }
            }
        }
        autonomy_status = {
            "runtime_truth": {
                "evidence_class": "placeholder_or_simulated_step_completion",
                "proof_boundary": {
                    "graph_execution": "workflow graph executed successfully",
                    "semantic_completion": "semantic completion not yet proven",
                    "grounded_tool_execution": "grounded tool execution: none/minimal",
                    "task_proof": "result is orchestration proof, not substantive task proof",
                },
                "run_trace": {
                    "trace_root_id": "workflow-1",
                    "workflow_id": "workflow-1",
                    "relationships": ["graph", "tool_boundary", "supervision"],
                },
                "grounded_evidence": [],
            }
        }

        self.module.assert_runtime_truth_surfaces(
            task_status,
            autonomy_status,
            require_consistency=True,
        )

        autonomy_status["runtime_truth"]["proof_boundary"][
            "task_proof"
        ] = "fresh live task proof captured"
        with self.assertRaises(self.module.SmokeHarnessError):
            self.module.assert_runtime_truth_surfaces(
                task_status,
                autonomy_status,
                require_consistency=True,
            )

    def test_assert_step_policy_surfaces_requires_packet025_fields_and_honest_wording(
        self,
    ) -> None:
        task_status = {
            "result": {
                "step_policy": {
                    "difficulty_assessment": {
                        "workflow_id": "workflow-1",
                        "step_id": "planner.step.2",
                        "difficulty_bucket": "high",
                        "confidence_label": "deterministic",
                        "reason_codes": [
                            "weak_current_evidence",
                            "unstable_recent_step_history",
                        ],
                    },
                    "budget_pressure": {
                        "workflow_id": "workflow-1",
                        "step_id": "planner.step.2",
                        "pressure_level": "softcap",
                        "pressure_source": "step_routing_history",
                        "policy_hint": "prefer_local_correction_before_escalation",
                    },
                    "policy_decision": {
                        "workflow_id": "workflow-1",
                        "step_id": "planner.step.2",
                        "chosen_action": "clarify",
                        "action_reason": "difficulty=high budget=softcap missing_context",
                        "requires_operator_attention": True,
                    },
                    "input_refs": {
                        "runtime_truth": "packet-023:placeholder_or_simulated_step_completion",
                    },
                    "proof_boundary_ref": {
                        "owner_packet": "023",
                        "task_proof": "result is orchestration proof, not substantive task proof",
                    },
                    "display_note": "placeholder orchestration proof only; local correction preferred before escalation",
                }
            }
        }
        autonomy_status = {
            "step_policy": {
                "difficulty_assessment": {
                    "workflow_id": "workflow-1",
                    "step_id": "planner.step.2",
                    "difficulty_bucket": "high",
                    "confidence_label": "deterministic",
                    "reason_codes": [
                        "weak_current_evidence",
                        "unstable_recent_step_history",
                    ],
                },
                "budget_pressure": {
                    "workflow_id": "workflow-1",
                    "step_id": "planner.step.2",
                    "pressure_level": "softcap",
                    "pressure_source": "step_routing_history",
                    "policy_hint": "prefer_local_correction_before_escalation",
                },
                "policy_decision": {
                    "workflow_id": "workflow-1",
                    "step_id": "planner.step.2",
                    "chosen_action": "clarify",
                    "action_reason": "difficulty=high budget=softcap missing_context",
                    "requires_operator_attention": True,
                },
                "input_refs": {
                    "runtime_truth": "packet-023:placeholder_or_simulated_step_completion",
                },
                "proof_boundary_ref": {
                    "owner_packet": "023",
                    "task_proof": "result is orchestration proof, not substantive task proof",
                },
                "display_note": "placeholder orchestration proof only; local correction preferred before escalation",
            }
        }

        self.module.assert_step_policy_surfaces(
            task_status,
            autonomy_status,
            require_consistency=True,
        )

        autonomy_status["step_policy"]["proof_boundary_ref"]["task_proof"] = (
            "result is grounded task proof"
        )
        with self.assertRaises(self.module.SmokeHarnessError):
            self.module.assert_step_policy_surfaces(
                task_status,
                autonomy_status,
                require_consistency=True,
            )

        autonomy_status["step_policy"]["proof_boundary_ref"]["task_proof"] = (
            "result is orchestration proof, not substantive task proof"
        )
        autonomy_status["step_policy"]["input_refs"]["runtime_truth"] = (
            "packet-025:step-policy-upgraded-proof"
        )
        with self.assertRaises(self.module.SmokeHarnessError):
            self.module.assert_step_policy_surfaces(
                task_status,
                autonomy_status,
                require_consistency=True,
            )

    def test_build_config_sets_packet021_probe_expectations(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            namespace = type(
                "Args",
                (),
                {
                    "compose_file": Path(temp_dir) / "docker-compose.yml",
                    "artifact_root": Path(temp_dir),
                    "database_name": None,
                    "http_port": 8080,
                    "timeout_seconds": 1.0,
                    "poll_interval_seconds": 0.01,
                    "profile": "baseline",
                    "scenario": "packet021_supervision_probe",
                    "task_description": None,
                },
            )()

            config = self.module.build_config(namespace)

        self.assertTrue(config.require_supervision_evidence)
        self.assertTrue(config.require_runtime_truth)
        self.assertEqual(config.expected_topology_kind, "Hybrid")
        self.assertEqual(config.min_parallelism_width, 2)
        self.assertEqual(config.allowed_supervision_target_kinds, ("branch", "node", "graph"))

    def test_assert_runtime_log_markers_requires_all_expected_markers(self) -> None:
        complete_log = "\n".join(self.module.REQUIRED_RUNTIME_LOG_MARKERS)
        self.module.assert_runtime_log_markers(complete_log)

        with self.assertRaises(self.module.SmokeHarnessError):
            self.module.assert_runtime_log_markers("Runtime task execution service ready")

    def test_wait_for_runtime_log_markers_reads_until_markers_exist(self) -> None:
        class StubProcess:
            returncode = None

            @staticmethod
            def poll():
                return None

        with tempfile.TemporaryDirectory() as temp_dir:
            artifact_dir = Path(temp_dir)
            (artifact_dir / "runtime.log").write_text(
                "\n".join(self.module.REQUIRED_RUNTIME_LOG_MARKERS),
                encoding="utf-8",
            )
            config = self.module.HarnessConfig(
                run_id="20260326T200000Z",
                compose_file=artifact_dir / "docker-compose.yml",
                artifact_dir=artifact_dir,
                database_name="ms_test",
                database_url="postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/ms_test",
                http_port=8080,
                base_url="http://127.0.0.1:8080",
                timeout_seconds=1.0,
                poll_interval_seconds=0.01,
                provider_kind="openai_chatgpt",
                model_id="gpt-5.4",
                profile="baseline",
                scenario="baseline",
                task_description=self.module.DEFAULT_TASK_DESCRIPTION,
                runtime_config_path=None,
                routing_policy="round_robin",
                registered_provider_count=1,
                budget_root=None,
                budget_policy=None,
                expected_step_action=None,
                expected_step_tier=None,
                required_step_checkpoints=(),
                expected_topology_kind=None,
                min_parallelism_width=None,
                max_parallelism_width=None,
                require_supervision_evidence=False,
                allowed_supervision_target_kinds=(),
                require_supervision_decision_basis=False,
                require_supervision_proof_boundary=False,
                require_detailed_supervision_payload=False,
                require_supervision_consistency=False,
            )

            log_text = self.module.wait_for_runtime_log_markers(config, StubProcess())
            self.assertIn("Mister Smith ready", log_text)

    def test_annotate_task_status_artifact_marks_planner_output_untrusted(self) -> None:
        payload = {
            "result": {
                "result": {
                    "planner_output": {
                        "steps": [{"id": "join-proof-boundary-summary", "role": "worker"}]
                    }
                }
            }
        }

        annotated = self.module.annotate_task_status_artifact(payload)
        result = annotated["result"]["result"]
        self.assertEqual(result["planner_output_trust"], "raw_untrusted")
        self.assertIn("runtime_execution_mode", result["planner_output_note"])

    def test_wait_for_terminal_task_status_retries_transient_fetch_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            config = self.module.HarnessConfig(
                run_id="20260326T200100Z",
                compose_file=Path("/tmp/docker-compose.yml"),
                artifact_dir=Path(temp_dir),
                database_name="ms_test",
                database_url="postgres://mistersmith:mistersmith_dev@127.0.0.1:5432/ms_test",
                http_port=8080,
                base_url="http://127.0.0.1:8080",
                timeout_seconds=1.0,
                poll_interval_seconds=0.01,
                provider_kind="openai_chatgpt",
                model_id="gpt-5.4",
                profile="baseline",
                scenario="baseline",
                task_description=self.module.DEFAULT_TASK_DESCRIPTION,
                runtime_config_path=None,
                routing_policy="round_robin",
                registered_provider_count=1,
                budget_root=None,
                budget_policy=None,
                expected_step_action=None,
                expected_step_tier=None,
                required_step_checkpoints=(),
                expected_topology_kind=None,
                min_parallelism_width=None,
                max_parallelism_width=None,
                require_supervision_evidence=False,
                allowed_supervision_target_kinds=(),
                require_supervision_decision_basis=False,
                require_supervision_proof_boundary=False,
                require_detailed_supervision_payload=False,
                require_supervision_consistency=False,
            )

            class StubResponse:
                def __init__(self, payload):
                    self.payload = payload

            responses = iter(
                [
                    self.module.SmokeHarnessError("temporary 503"),
                    StubResponse({"task_id": "task-1", "status": "completed"}),
                ]
            )
            original_fetch_json = self.module.fetch_json

            def stub_fetch_json(_url: str):
                response = next(responses)
                if isinstance(response, Exception):
                    raise response
                return response

            self.module.fetch_json = stub_fetch_json
            try:
                payload = self.module.wait_for_terminal_task_status(config, "task-1")
            finally:
                self.module.fetch_json = original_fetch_json

            self.assertEqual(payload["status"], "completed")
            poll_log = (config.artifact_dir / "task-poll.log").read_text(encoding="utf-8")
            self.assertIn("error=temporary 503", poll_log)
            self.assertIn("status=completed", poll_log)


if __name__ == "__main__":
    unittest.main()
