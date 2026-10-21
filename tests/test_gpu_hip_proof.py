"""Failure-contract regressions; no compiler, sandbox, network, or seed required."""

import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/prove-gpu-hip.py"
spec = importlib.util.spec_from_file_location("hip_proof", SCRIPT)
proof = importlib.util.module_from_spec(spec)
spec.loader.exec_module(proof)


class CurrentFailureContractTest(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="hip-proof-contract-")
        self.addCleanup(temporary.cleanup)
        self.case = Path(temporary.name).resolve()
        self.entry = self.case / "negative.ncl"
        self.prefix = "/mantle/store"
        self.label = "hip-negative"
        self.key = self.prefix + "/" + "0" * 32 + "-hip-negative.drv"
        self.logs = self.case / "build/logs"
        self.logs.mkdir(parents=True)
        self.log = self.logs / (Path(self.key).name + ".log")
        self.report = {
            "schema": "crunch-build-report-v1", "file": str(self.entry),
            "output_dir": str(self.case / "store"), "state_dir": str(self.case / "state"),
            "store_dir": self.prefix, "diagnostic_persistence_failures": [],
            "counts": {"succeeded_total": 0, "built_total": 0, "cached_total": 0, "failed_total": 1},
            "outcomes": [], "failed": [{
                "root": self.label, "drv_key": self.key, "phase": "build", "error_class": "builder",
                "message": "", "saved_log_path": str(self.log),
            }],
        }
        self.diagnostics = {
            "network": "gpu-network-denied: Connection refused (os error 111)\n",
            "undeclared-host-path": (
                "/inputs/kernel/escape.hip:2:10: fatal error: '/tmp/mantle-gpu-undeclared-secret.h' file not found\n"
                "    2 | #include \"/tmp/mantle-gpu-undeclared-secret.h\"\n"
                "1 error generated when compiling for gfx900.\n"
                "gpu-compile-failed: exit status: 1\n"
            ),
            "missing-library": "gpu-device-library-missing-or-escaping: ocml.bc: gpu-input-missing: /inputs/ocml.bc\n",
        }

    def current_failure(self, diagnostic):
        message = "store error: build: nonzero exit code: exit status: 1\n" + diagnostic
        self.report["failed"][0]["message"] = message
        self.log.write_text(
            f"# crunch build log\n# derivation: {self.label}\n# drv_path: {Path(self.key).name}\n"
            f"# status: failure\n# timestamp: 1770000000\n\n{message}\n"
        )

    def classify(self, boundary, *, report=None, returncode=1, stderr=""):
        result = subprocess.CompletedProcess([], returncode, json.dumps(self.report if report is None else report), stderr)
        return proof.diagnosed_failure(
            result, self.entry, self.case, self.prefix, self.label, self.key, self.logs,
            proof.FAILURE_DIAGNOSTICS[boundary],
            forbidden=("gpu-network-escape:", "gpu-network-probe-inconclusive:") if boundary == "network" else (),
        )

    def test_historical_diagnostic_cannot_mask_current_unrelated_failure(self):
        historical = self.case / "state/logs/historical.drv.log"
        historical.parent.mkdir(parents=True)
        for boundary in ("network", "undeclared-host-path"):
            with self.subTest(boundary=boundary):
                historical.write_text(self.diagnostics[boundary])
                original = historical.read_bytes()
                self.current_failure("gpu-input-missing: /inputs/escape.hip: No such file or directory\n")
                # Even a matching current stderr is not the saved execution log.
                with self.assertRaisesRegex(RuntimeError, "intended negative boundary"):
                    self.classify(boundary, stderr=self.diagnostics[boundary])
                self.assertEqual(historical.read_bytes(), original)

    def test_current_diagnosed_expected_execution_is_admitted(self):
        for boundary, diagnostic in self.diagnostics.items():
            with self.subTest(boundary=boundary):
                self.current_failure(diagnostic)
                receipt = self.classify(boundary)
                self.assertEqual(receipt["drv_key"], self.key)
                self.assertEqual(receipt["saved_log_path"], str(self.log))
                self.assertEqual((receipt["phase"], receipt["error_class"], receipt["rejected"]),
                                 ("build", "builder", True))

    def test_preflight_planning_admission_and_other_derivation_are_not_execution(self):
        self.current_failure(self.diagnostics["network"])
        for field, value in (
            ("phase", "preflight"), ("error_class", "cache-verification"),
            ("error_class", "missing-source-input"), ("error_class", "sandbox"),
            ("root", "dependency"), ("drv_key", self.key.replace("negative", "dependency")),
            ("message", "dependency fixture failed"),
        ):
            with self.subTest(field=field, value=value):
                report = copy.deepcopy(self.report)
                report["failed"][0][field] = value
                with self.assertRaisesRegex(RuntimeError, "intended derivation"):
                    self.classify("network", report=report)

    def test_missing_ambiguous_or_successful_failure_facts_are_rejected(self):
        self.current_failure(self.diagnostics["network"])
        for failures in ([], self.report["failed"] * 2):
            report = copy.deepcopy(self.report)
            report["failed"] = failures
            with self.assertRaises(RuntimeError):
                self.classify("network", report=report)
        with self.assertRaises(RuntimeError):
            self.classify("network", returncode=0)
        report = copy.deepcopy(self.report)
        del report["failed"][0]["saved_log_path"]
        with self.assertRaises(RuntimeError):
            self.classify("network", report=report)

    def test_inherited_or_symlinked_log_is_rejected(self):
        self.current_failure(self.diagnostics["network"])
        inherited = self.case / "state/logs" / self.log.name
        inherited.parent.mkdir(parents=True)
        inherited.write_bytes(self.log.read_bytes())
        report = copy.deepcopy(self.report)
        report["failed"][0]["saved_log_path"] = str(inherited)
        with self.assertRaisesRegex(RuntimeError, "confined"):
            self.classify("network", report=report)
        self.log.unlink()
        self.log.symlink_to(inherited)
        with self.assertRaisesRegex(RuntimeError, "confined"):
            self.classify("network")

    def test_oversized_or_shared_log_is_rejected(self):
        self.current_failure(self.diagnostics["network"])
        with self.log.open("r+b") as stream:
            stream.truncate(proof.MAX_FAILURE_BYTES + 1)
        with self.assertRaisesRegex(RuntimeError, "bounded, unshared"):
            self.classify("network")
        self.current_failure(self.diagnostics["network"])
        os.link(self.log, self.case / "inherited-hardlink.log")
        with self.assertRaisesRegex(RuntimeError, "bounded, unshared"):
            self.classify("network")

    def test_contradictory_network_or_wrong_saved_identity_is_rejected(self):
        self.current_failure(self.diagnostics["network"] + "gpu-network-escape: host listener reachable\n")
        with self.assertRaisesRegex(RuntimeError, "intended negative boundary"):
            self.classify("network")
        self.current_failure(self.diagnostics["network"])
        self.log.write_text(self.log.read_text().replace("# status: failure", "# status: success"))
        with self.assertRaisesRegex(RuntimeError, "identity/status"):
            self.classify("network")

    def test_plan_binds_one_uncached_root_and_rejects_planning_failure(self):
        report = {key: self.report[key] for key in ("file", "output_dir", "state_dir", "store_dir")}
        report.update(schema="crunch-build-plan-v1", entries=[{
            "drv_key": self.key, "label": self.label, "action": "build",
        }])
        def planned():
            result = subprocess.CompletedProcess([], 0, json.dumps(report), "")
            return proof.planned_derivation(result, self.entry, self.case, self.prefix, self.label)
        self.assertEqual(planned(), self.key)
        for action in ("preflight-error", "cached", "substitute"):
            report["entries"][0]["action"] = action
            with self.assertRaisesRegex(RuntimeError, "uncached, executable"):
                planned()


if __name__ == "__main__":
    unittest.main()
