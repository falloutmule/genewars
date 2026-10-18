#!/usr/bin/env python3
"""Source-pinned release replay of the picked-move BTURR construction cycle."""
from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[2]
BINARY = ROOT / "private/build/rust-target/release/genewars-game.exe"
BUNDLE = ROOT / "private/integration/game-runtime"
OUTPUT = ROOT / "research/integration/build-completion-native-current.json"
SCRIPT = Path(__file__).resolve()


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def source_files() -> list[Path]:
    files = sorted((ROOT / "rust").glob("Cargo.*"))
    files += sorted((ROOT / "rust/crates").glob("*/Cargo.toml"))
    files += sorted((ROOT / "rust/crates").glob("*/src/**/*.rs"))
    files += [SCRIPT, ROOT / "research/integration/runtime-resource-catalog.tsv",
              ROOT / "research/integration/runtime-inputs.json"]
    return sorted(set(files))


def pins(paths: list[Path]) -> dict[str, str]:
    return {path.relative_to(ROOT).as_posix(): sha_file(path) for path in paths}


def run_replay(binary: Path, folder: Path, replay: str, *, window: bool) -> dict:
    folder.mkdir(parents=True, exist_ok=False)
    replay_path = folder / "replay.txt"
    replay_path.write_text(replay, encoding="utf-8", newline="\n")
    report_path, bmp_path = folder / "report.json", folder / "frame.bmp"
    command = [str(binary), str(BUNDLE), "--landing-seed", "5428",
               "--window-replay" if window else "--headless", str(replay_path),
               "--report", str(report_path), "--bmp", str(bmp_path)]
    if window:
        command += ["--frames", "3", "--seconds", "20"]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=900)
    (folder / "stdout.txt").write_text(result.stdout, encoding="utf-8")
    (folder / "stderr.txt").write_text(result.stderr, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(f"{folder.name} failed ({result.returncode}): {result.stderr[-2000:]}")
    report = json.loads(result.stdout)
    if json.loads(report_path.read_text(encoding="utf-8")) != report:
        raise AssertionError(f"{folder.name}: stdout and report file differ")
    mission = report.get("mission")
    if report.get("scene") != "Mission" or mission is None:
        raise AssertionError(f"{folder.name}: replay did not reach mission scene")
    if mission.get("blocked") is not None or report.get("error") is not None:
        raise AssertionError(f"{folder.name}: blocked runtime {mission.get('blocked') or report.get('error')}")
    if mission.get("steps") != 689:
        raise AssertionError(f"{folder.name}: expected 689 normal ticks, got {mission.get('steps')}")
    if window and report.get("presented_frames") != 3:
        raise AssertionError(f"{folder.name}: expected exactly 3 presented frames")
    balances = {row["slot"]: row["balance"] for row in mission["player_funds"]}
    if balances.get(0) != 1312:
        raise AssertionError(f"{folder.name}: expected exact source debit 2000->1312, got {balances}")
    buildings = mission["buildings"]
    completed = [b for b in buildings if b["slot"] == 0 and b["selector"] == 1
                 and b["progress_word_2b"] == 120 and b["flags_word_31"] & 1]
    if len(completed) != 1:
        raise AssertionError(f"{folder.name}: completed BTURR record missing: {buildings}")
    actors = mission["actor_states"]
    architect = next((a for a in actors if a["slot"] == 0), None)
    if architect is None or architect["state"] != 1 or architect["pending_move"] or architect["assignment_pending"]:
        raise AssertionError(f"{folder.name}: architect did not return to ordinary idle: {architect}")
    if mission["terrain_jobs"]["count"] != 0:
        raise AssertionError(f"{folder.name}: terrain jobs remain after completion")
    bmp = bmp_path.read_bytes()
    if len(bmp) != 54 + 640 * 480 * 3 or bmp[:2] != b"BM":
        raise AssertionError(f"{folder.name}: expected 640x480 BMP")
    return {"name": folder.name, "window": window, "steps": mission["steps"],
            "presented_frames": report.get("presented_frames"),
            "world_sha256": mission["world_sha256"], "reported_rgb_sha256": mission["rgb_sha256"],
            "bmp_sha256": sha_bytes(bmp), "funds": balances,
            "completed_building": completed[0], "architect": architect,
            "terrain_job_count": mission["terrain_jobs"]["count"],
            "report_path": report_path.relative_to(ROOT).as_posix(),
            "bmp_path": bmp_path.relative_to(ROOT).as_posix()}


def main() -> int:
    if not BUNDLE.is_dir():
        raise FileNotFoundError("staged private runtime bundle is required")
    paths = source_files()
    source_pins = pins(paths)
    test = subprocess.run([sys.executable, "-B", "scripts/cargo.py", "test", "--release", "--offline", "--locked",
                           "-p", "genewars-desktop", "--bin", "genewars-game",
                           "development_build_flow_uses_picked_move_and_opcode5_idle_guard", "--", "--nocapture"],
                          cwd=ROOT, capture_output=True, text=True, timeout=600)
    if test.returncode:
        raise RuntimeError(f"development build fixture failed ({test.returncode}):\n{test.stdout[-2500:]}\n{test.stderr[-2500:]}")
    coordinate = re.search(r"development build pick: screen=\[(\d+), (\d+)\], exact_picker_target=\[(-?\d+), (-?\d+)\]", test.stdout + test.stderr)
    if not coordinate:
        raise AssertionError("source-backed fixture did not report its actual terrain-picker coordinate")
    screen = [int(coordinate.group(1)), int(coordinate.group(2))]
    picked_target = [int(coordinate.group(3)), int(coordinate.group(4))]
    if pins(paths) != source_pins:
        raise RuntimeError("Rust sources changed during picked-build fixture")
    build = subprocess.run([sys.executable, "-B", "scripts/cargo.py", "build", "--release", "--offline", "--locked",
                           "-p", "genewars-desktop", "--bin", "genewars-game"],
                          cwd=ROOT, capture_output=True, text=True, timeout=600)
    if build.returncode:
        raise RuntimeError(f"release build failed ({build.returncode}): {build.stderr[-3000:]}")
    if pins(paths) != source_pins:
        raise RuntimeError("Rust sources changed while building native replay binary")

    work = ROOT / "private/integration/build-completion-native" / uuid.uuid4().hex
    work.mkdir(parents=True, exist_ok=False)
    copied_binary = work / "genewars-game.exe"
    shutil.copyfile(BINARY, copied_binary)
    binary_sha = sha_file(copied_binary)
    for path in paths:
        archived = work / "compilation_sources" / path.relative_to(ROOT)
        archived.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, archived)
    input_bytes = (ROOT / "research/integration/runtime-inputs.json").read_bytes()
    (work / "runtime-inputs.json").write_bytes(input_bytes)
    commands = ["click 312 140", "load", "pan -897 -641", "tick 189", "select 0",
                f"build-order {screen[0]} {screen[1]}", "tick 100", "build-start", "tick 400"]
    replay = "\n".join(commands) + "\n"
    (work / "compilation.json").write_text(json.dumps({
        "binary_sha256": binary_sha, "source_pins": source_pins,
        "runtime_inputs_sha256": sha_bytes(input_bytes), "runtime_catalog_sha256": sha_file(ROOT / "research/integration/runtime-resource-catalog.tsv"),
        "terrain_picker_screen": screen, "picked_target": picked_target, "commands": commands,
        "original_execution": False,
    }, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    cases = [run_replay(copied_binary, work / "headless-a", replay, window=False),
             run_replay(copied_binary, work / "headless-b", replay, window=False)]
    if (cases[0]["world_sha256"], cases[0]["bmp_sha256"]) != (cases[1]["world_sha256"], cases[1]["bmp_sha256"]):
        raise AssertionError("repeated headless construction replay is not deterministic")
    cases.append(run_replay(copied_binary, work / "window-3frames", replay, window=True))
    if (cases[0]["world_sha256"], cases[0]["bmp_sha256"]) != (cases[2]["world_sha256"], cases[2]["bmp_sha256"]):
        raise AssertionError("three-frame window final image/state differs from the headless result")
    report = {
        "schema_version": 1, "status": "passed-source-backed-construction-native-replay",
        "verified_at_utc": datetime.now(timezone.utc).isoformat(),
        "scope": "Source-pinned native release replay uses the terrain picker, normal actor callbacks, guarded opcode-5 assignment, source-funded BTURR creation/completion and architect return to idle; two headless runs and a bounded 3-frame window run are compared. Not an original executable run or full-game claim.",
        "binary": {"sha256": binary_sha, "size": copied_binary.stat().st_size,
                   "archived_path": copied_binary.relative_to(ROOT).as_posix()},
        "source_pins": source_pins, "runtime_inputs_sha256": sha_bytes(input_bytes),
        "runtime_catalog_sha256": sha_file(ROOT / "research/integration/runtime-resource-catalog.tsv"),
        "seed": 5428, "picker_screen": screen, "picked_target": picked_target,
        "commands": commands, "cases": cases,
        "deterministic_headless_repeat": True, "window_matches_headless": True,
        "original_execution": False, "private_run_directory": work.relative_to(ROOT).as_posix(),
    }
    tmp = OUTPUT.with_suffix(".json.tmp")
    tmp.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    tmp.replace(OUTPUT)
    print(json.dumps({"status": report["status"], "binary_sha256": binary_sha,
                      "picker_screen": screen, "picked_target": picked_target,
                      "cases": [{k: v for k, v in case.items() if k not in ("bmp_path", "report_path")} for case in cases],
                      "receipt": OUTPUT.relative_to(ROOT).as_posix()}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
