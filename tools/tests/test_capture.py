"""The evidence gate must reject missing frames and failed runtime assertions."""
import json
from pathlib import Path
import tempfile
import unittest
import sys
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from capture import verify_outputs, run_logged


class CaptureTests(unittest.TestCase):
    def test_evidence_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / "frames").mkdir()
            (output / "keyframes").mkdir()
            script = {"frames": 2, "keyframes": [0], "width": 640, "height": 360}
            state = {"status": "PASS", "checks": [{"name": "movement", "passed": True}]}
            (output / "state.json").write_text(json.dumps(state))
            for relative in ["frames/frame00000.png", "frames/frame00001.png", "keyframes/frame00000.png"]:
                Image.new("RGB", (640, 360), "orange").save(output / relative)
            self.assertEqual(verify_outputs(output, script)["status"], "PASS")
            state["checks"][0]["passed"] = False
            (output / "state.json").write_text(json.dumps(state))
            with self.assertRaisesRegex(ValueError, "movement"):
                verify_outputs(output, script)
            state["checks"][0]["passed"] = True
            (output / "state.json").write_text(json.dumps(state))
            corrupt = output / "frames/frame00001.png"
            original = corrupt.read_bytes()
            bad_crc = bytearray(original)
            bad_crc[bad_crc.index(b"IDAT") + 4] ^= 1
            corrupt.write_bytes(bad_crc)
            with self.assertRaisesRegex(ValueError, "unreadable PNG"):
                verify_outputs(output, script)
            corrupt.write_bytes(original[:24])
            with self.assertRaisesRegex(ValueError, "unreadable PNG"):
                verify_outputs(output, script)
            corrupt.unlink()
            with self.assertRaisesRegex(ValueError, "incomplete"):
                verify_outputs(output, script)

    def test_process_timeout_is_nonzero(self):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "timeout.log"
            self.assertEqual(run_logged([sys.executable, "-c", "import time; time.sleep(10)"], log, 0.05), 124)
            self.assertIn("timeout", log.read_text())


if __name__ == "__main__":
    unittest.main()
