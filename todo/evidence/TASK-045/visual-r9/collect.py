"""Retain actual r9 run records and hashes before cleaning temporary visuals."""
from pathlib import Path
import hashlib
import json
import shutil

root = Path(__file__).resolve().parents[4]
out = Path(__file__).resolve().parent
rows = []
for directory in sorted((root / "output/visual-r9").iterdir()):
    if not (directory / "run.json").is_file():
        continue
    run = json.loads((directory / "run.json").read_text())
    state = json.loads((directory / "state.json").read_text()) if (directory / "state.json").exists() else {}
    samples = state.pop("samples", [])
    destination = out / directory.name
    destination.mkdir(exist_ok=True)
    for filename in ("run.json", "runtime.log", "script.json"):
        if (directory / filename).exists():
            shutil.copyfile(directory / filename, destination / filename)
    state["sample_count"] = len(samples)
    state["full_state_path"] = str(directory.relative_to(root) / "state.json")
    state["full_state_sha256"] = hashlib.sha256((directory / "state.json").read_bytes()).hexdigest() if samples else None
    state["first_ready_sample"] = next((sample for sample in samples if sample.get("world_ready")), None)
    (destination / "state-summary.json").write_text(json.dumps(state, ensure_ascii=False, indent=2) + "\n")
    visuals = []
    for path in sorted(directory.rglob("*")):
        if path.is_file() and path.suffix in (".png", ".mp4"):
            visuals.append({"file": str(path.relative_to(directory)), "bytes": path.stat().st_size,
                            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    # One compact JSON line per actual file keeps the complete cleanup trail readable
    (destination / "visual-files.jsonl").write_text("".join(json.dumps(item) + "\n" for item in visuals))
    rows.append({"run": directory.name, "status": run["status"], "frames": len(samples),
                 "native_checks": len(state.get("checks", [])),
                 "wrapper_checks": sum(c.get("source") == "capture wrapper" for c in run.get("checks", [])),
                 "visual_files": len(visuals), "runtime_exit_code": run.get("runtime_exit_code")})
(out / "summary.json").write_text(json.dumps(rows, indent=2) + "\n")
print(json.dumps(rows))
