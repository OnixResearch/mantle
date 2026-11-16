"""Reject crashes and launch failures as native negative-probe evidence."""
import os
from pathlib import Path
import signal
import sys
import tempfile
import unittest

from native_row_qualification_matrix import call


@unittest.skipUnless(os.name == "posix", "native probes use POSIX process groups")
class NativeNegativeProbeTests(unittest.TestCase):
    def test_diagnostic_rejection_is_distinct_from_signal_or_launch_failure(self):
        with tempfile.TemporaryDirectory(prefix="mantle-native-probe-") as directory:
            work = Path(directory)
            diagnostic = "import sys; sys.stderr.write('invalid input\\n'); "
            call([sys.executable, "-c", diagnostic + "sys.exit(1)"], work,
                 "ordinary-rejection", expected="reject")
            for code in (126, 127, 139):
                with self.subTest(exit_code=code):
                    with self.assertRaisesRegex(RuntimeError, "crashed, failed to execute"):
                        call([sys.executable, "-c", diagnostic + f"sys.exit({code})"], work,
                             f"invalid-rejection-{code}", expected="reject")
            with self.assertRaisesRegex(RuntimeError, "crashed, failed to execute"):
                call([sys.executable, "-c", diagnostic +
                      f"import os; os.kill(os.getpid(), {signal.SIGTERM})"], work,
                     "signalled-rejection", expected="reject")


if __name__ == "__main__":
    unittest.main()
