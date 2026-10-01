"""Run the three existing saved-master exporters, preserving historical V15 reports."""
import json
import os
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
reports = {}
for asset, script in (("V-15", "build.py"), ("V-A08", "facade-build.py"), ("V-55", "build.py")):
    old_reports = {}
    if asset == "V-15":
        for name in ("component-bounds.json", "facade-checks.json", "export-report.json"):
            path = ROOT / "todo/evidence/TASK-049/v15-building-r2" / name
            old_reports[path] = path.read_bytes()
    command = ["blender", "--background", "-noaudio", "--threads", "2", "--python-exit-code", "1",
               "--python", f"source-assets/buildings/{asset}/{script}"]
    try:
        with (HERE / f"{asset}-export.log").open("w") as log:
            result = subprocess.run(command, cwd=ROOT, env={**os.environ, "ALSOFT_DRIVERS": "null"},
                                    stdout=log, stderr=subprocess.STDOUT, check=False)
        reports[asset] = {"command": command, "ALSOFT_DRIVERS": "null", "exit_code": result.returncode}
        (HERE / "export-commands.json").write_text(json.dumps(reports, indent=2) + "\n")
        assert result.returncode == 0, (asset, result.returncode)
    finally:
        for path, data in old_reports.items():
            (HERE / f"V-15-{path.name}").write_bytes(path.read_bytes())
            path.write_bytes(data)
print(json.dumps({"status": "PASS", "assets": reports}, indent=2))
