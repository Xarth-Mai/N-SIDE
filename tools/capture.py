#!/usr/bin/env python3
"""Run the real Viewer capture, retain provenance, and encode frames when ffmpeg exists.

Workflow adapted from htdt/godogen engines/bevy.md, revision
0b725bca053769a4727f76c332bf1f7b42e146ab, Copyright 2026 Alex Ermolov (MIT).
Retained license: third_party/skills/godogen/LICENSE.md. This implementation
uses the existing N:SIDE Viewer, its scripted inputs, and checked PNG output.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def digest(path: Path) -> str:
    checksum = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            checksum.update(chunk)
    return checksum.hexdigest()


def command_output(command: list[str]) -> str:
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    return result.stdout.strip() if result.returncode == 0 else "unavailable"


def run_logged(command: list[str], path: Path, timeout: float) -> int:
    with path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=os.name == "posix")
        try:
            return process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            if os.name == "posix":
                os.killpg(process.pid, signal.SIGKILL)
            else:
                process.kill()
            process.wait()
            log.write(f"\n[capture/wrapper] timeout after {timeout} seconds\n")
            return 124


def verify_outputs(output: Path, script: dict) -> dict:
    from PIL import Image

    state = json.loads((output / "state.json").read_text(encoding="utf-8"))
    expected = [f"frame{i:05}.png" for i in range(script["frames"])]
    actual = sorted(path.name for path in (output / "frames").glob("*.png"))
    if actual != expected:
        raise ValueError(f"frame sequence incomplete: expected {len(expected)}, found {len(actual)}")
    for relative in [*[f"frames/{name}" for name in expected], *[f"keyframes/frame{i:05}.png" for i in script["keyframes"]]]:
        path = output / relative
        try:
            with Image.open(path) as image:
                if image.format != "PNG" or image.size != (script["width"], script["height"]):
                    raise ValueError(f"invalid PNG format/resolution: {relative}")
                image.verify()
            with Image.open(path) as image:
                image.load()
        except (OSError, SyntaxError) as error:
            raise ValueError(f"unreadable PNG {relative}: {error}") from error
    if state["status"] != "PASS" or not state["checks"] or any(not check["passed"] for check in state["checks"]):
        failed = [check["name"] for check in state.get("checks", []) if not check["passed"]]
        raise ValueError(f"state assertions failed: {failed}; {state.get('error')}")
    return state


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--script", type=Path, default=ROOT / "game/capture/viewer-tour.json")
    parser.add_argument("--output", type=Path, required=True, help="fresh evidence directory")
    parser.add_argument("--binary", type=Path, help="use this prebuilt map_viewer; otherwise cargo build --locked")
    parser.add_argument("--no-video", action="store_true", help="retain PNG evidence only")
    args = parser.parse_args()
    try:
        import PIL
    except ImportError:
        parser.error("Pillow is required; install .agents/skills/create-game-assets/scripts/requirements.txt")
    output = args.output.resolve()
    try:
        script_path = args.script.resolve()
        script = json.loads(script_path.read_text(encoding="utf-8"))
        # The native parser owns the complete script contract
        timeout = script["timeout_seconds"]
        if not isinstance(timeout, int) or not 1 <= timeout <= 3600:
            raise ValueError("timeout_seconds must be an integer in 1..3600")
        output.mkdir(parents=True, exist_ok=False)
    except (OSError, ValueError, KeyError) as error:
        print(f"[capture/setup] {error}", file=sys.stderr)
        return 1
    report = {
        "status": "FAIL", "script": script, "script_sha256": digest(script_path),
        "git_commit": command_output(["git", "rev-parse", "HEAD"]),
        "git_worktree": command_output(["git", "status", "--porcelain"]),
        "cargo_lock_sha256": digest(ROOT / "game/Cargo.lock"),
        "platform": platform.platform(), "python": platform.python_version(), "pillow": PIL.__version__,
        "video": "NOT RUN", "visual_review": "NOT RUN: inspect real keyframes and consecutive frames/video separately",
    }
    started = time.monotonic()
    exit_code = 1
    try:
        shutil.copyfile(script_path, output / "script.json")
        binary = args.binary.resolve() if args.binary else ROOT / "game/target/debug" / ("map_viewer.exe" if os.name == "nt" else "map_viewer")
        if args.binary is None:
            build = ["cargo", "build", "--manifest-path", "game/Cargo.toml", "--features", "viewer", "--bin", "map_viewer", "--locked"]
            report["build_command"] = build
            code = run_logged(build, output / "build.log", 900)
            if code:
                raise RuntimeError(f"cargo build exited {code}; see build.log")
        report["binary_sha256"] = digest(binary)
        command = [str(binary), "--project-root", str(ROOT), "--capture", str(output / "script.json"), "--output", str(output)]
        report["command"] = command
        code = run_logged(command, output / "runtime.log", timeout + 30)
        report["runtime_exit_code"] = code
        if code:
            raise RuntimeError(f"Viewer exited {code}; see runtime.log and state.json when available")
        state = verify_outputs(output, script)
        report["checks"] = state["checks"]
        ffmpeg = shutil.which("ffmpeg")
        if ffmpeg and not args.no_video:
            report["ffmpeg_version"] = command_output([ffmpeg, "-version"]).splitlines()[0]
            encode = [ffmpeg, "-nostdin", "-hide_banner", "-loglevel", "warning", "-framerate", str(script["fps"]), "-i", str(output / "frames/frame%05d.png"), "-frames:v", str(script["frames"]), "-c:v", "libx264", "-pix_fmt", "yuv420p", "-movflags", "+faststart", str(output / "video.mp4")]
            report["video_command"] = encode
            code = run_logged(encode, output / "ffmpeg.log", 300)
            if code or not (output / "video.mp4").is_file() or (output / "video.mp4").stat().st_size == 0:
                raise RuntimeError(f"ffmpeg failed with exit {code}; see ffmpeg.log")
            report["video"] = "PASS"
        else:
            report["video"] = "NOT RUN: --no-video" if args.no_video else "NOT RUN: ffmpeg unavailable; PNG sequence retained"
        report["status"] = "PASS"
        exit_code = 0
    except (OSError, ValueError, KeyError, RuntimeError, ImportError) as error:
        report["error"] = str(error)
        print(f"[capture/failed] {error}", file=sys.stderr)
    finally:
        report["elapsed_wall_seconds"] = time.monotonic() - started
        (output / "run.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[capture/{report['status']}] {output / 'run.json'}")
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
