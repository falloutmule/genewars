#!/usr/bin/env python3
"""Bounded deterministic release replay of the current source-backed native world."""
from __future__ import annotations

from datetime import datetime, timezone
import argparse
import sys
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[2]
BINARY = ROOT / "private/build/rust-target/release/genewars-game.exe"
BUNDLE = ROOT / "private/integration/game-runtime"
REPLAY = "click 312 140\nload\ntick 4096\ntick 2047\nselect 0\n"
CODE_SOURCES = [
    "rust/crates/genewars-desktop/src/bin/genewars-game.rs",
    "rust/crates/genewars-desktop/src/bin/support/runtime_scene.rs",
    "rust/crates/genewars-desktop/src/bin/support/shared_scene.rs",
    "rust/crates/genewars-desktop/src/bin/support/actor_scene.rs",
    "rust/crates/genewars-desktop/src/bin/support/family8_scene.rs",
    "rust/crates/genewars-desktop/src/bin/support/effect_scene.rs",
    "rust/crates/genewars-desktop/src/bin/support/runtime_resources.rs",
    "rust/crates/genewars-runtime/src/family8_motion.rs",
    "rust/crates/genewars-runtime/src/actor_world/family8_updates.rs",
    "research/integration/runtime-resource-catalog.tsv",
]


def sha_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha_file(path: Path) -> str:
    return sha_bytes(path.read_bytes())


def run_case(binary: Path, work: Path, name: str, *, steps: int, replay_text: str, window: bool = False) -> dict:
    folder = work / name
    folder.mkdir()
    replay = folder / "replay.txt"
    replay.write_text(replay_text, encoding="utf-8", newline="\n")
    report_path = folder / "report.json"
    bmp_path = folder / "frame.bmp"
    command = [str(binary), str(BUNDLE), "--landing-seed", "5428",
               "--window-replay" if window else "--headless", str(replay),
               "--report", str(report_path), "--bmp", str(bmp_path)]
    if window:
        command += ["--frames", "3", "--seconds", "20"]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=900)
    if result.returncode:
        raise RuntimeError(f"{name} failed ({result.returncode}): {result.stderr[-2000:]}")
    stdout_report = json.loads(result.stdout)
    disk_report = json.loads(report_path.read_text(encoding="utf-8"))
    if stdout_report != disk_report:
        raise AssertionError(f"{name}: stdout and report file differ")
    mission = stdout_report.get("mission")
    if stdout_report.get("scene") != "Mission" or mission is None:
        raise AssertionError(f"{name}: replay did not reach mission scene")
    if mission.get("steps") != steps or mission.get("blocked") is not None:
        raise AssertionError(f"{name}: unexpected final step/block state {mission.get('steps')}/{mission.get('blocked')}")
    if mission.get("world_mode") != "landing-study" or mission.get("selected_actor") != 0:
        raise AssertionError(f"{name}: expected selected actor 0 in landing-study, got {mission}")
    if stdout_report.get("error") is not None:
        raise AssertionError(f"{name}: runtime error {stdout_report['error']!r}")
    if window and stdout_report.get("presented_frames") != 3:
        raise AssertionError(f"{name}: expected exactly 3 presented frames")
    bmp = bmp_path.read_bytes()
    if len(bmp) != 54 + 640 * 480 * 3 or bmp[:2] != b"BM":
        raise AssertionError(f"{name}: unexpected 640x480 BMP dimensions/length")
    return {
        "name": name,
        "window": window,
        "presented_frames": stdout_report.get("presented_frames"),
        "scene_history": stdout_report.get("scene_history"),
        "steps": mission["steps"],
        "blocked": mission["blocked"],
        "selected_actor": mission["selected_actor"],
        "active_effects": mission.get("active_effects"),
        "actor_states_sha256": sha_bytes(json.dumps(mission.get("actor_states"), sort_keys=True).encode()),
        "creature_states_sha256": sha_bytes(json.dumps(mission.get("creature_states"), sort_keys=True).encode()),
        "world_sha256": mission["world_sha256"],
        "reported_rgb_sha256": mission["rgb_sha256"],
        "bmp_sha256": sha_bytes(bmp),
        "bmp_path": bmp_path.relative_to(ROOT).as_posix(),
        "report_path": report_path.relative_to(ROOT).as_posix(),
    }


def main() -> int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--steps',type=int,default=6143)
    parser.add_argument('--output',type=Path,default=ROOT/'research/integration/family8-native-validation-current.json')
    args=parser.parse_args()
    assert 1<=args.steps<=32768
    commands=['click 312 140','load']
    remaining=args.steps
    while remaining:
        chunk=min(remaining,4096);commands.append(f'tick {chunk}');remaining-=chunk
    commands.append('select 0');replay_text='\n'.join(commands)+'\n'
    sources=sorted((ROOT/'rust').glob('Cargo.*'))+sorted((ROOT/'rust/crates').glob('*/Cargo.toml'))
    sources+=sorted((ROOT/'rust/crates').glob('*/src/**/*.rs'))+[Path(__file__), ROOT/'research/integration/runtime-resource-catalog.tsv']
    compilation_pins={p.relative_to(ROOT).as_posix():sha_file(p)for p in sources}
    built=subprocess.run([sys.executable,'-B','scripts/cargo.py','build','--release','-p','genewars-desktop','--bin','genewars-game','--offline','--locked'],cwd=ROOT,capture_output=True,text=True,timeout=300)
    if built.returncode:raise RuntimeError(built.stderr[-4000:])
    assert compilation_pins=={p.relative_to(ROOT).as_posix():sha_file(p)for p in sources},'sources changed during build; retry'
    if not BINARY.is_file() or not BUNDLE.is_dir():
        raise FileNotFoundError("the pinned release executable and staged private runtime bundle are required")
    work = ROOT / "private/integration/family8-native" / uuid.uuid4().hex
    work.mkdir(parents=True, exist_ok=False)
    binary_copy = work / "genewars-game.exe"
    with binary_copy.open("xb") as out:
        out.write(BINARY.read_bytes())
    binary_sha = sha_file(binary_copy)
    for rel, digest in compilation_pins.items():
        contents = (ROOT / rel).read_bytes()
        assert sha_bytes(contents) == digest, 'source changed before archival: ' + rel
        archived = work / 'compilation_sources' / rel
        archived.parent.mkdir(parents=True, exist_ok=True)
        archived.write_bytes(contents)
    source_pins = {rel: sha_file(ROOT / rel) for rel in CODE_SOURCES}
    input_bytes = (ROOT / "research/integration/runtime-inputs.json").read_bytes()
    inputs = json.loads(input_bytes)
    catalog_sha = sha_file(ROOT / "research/integration/runtime-resource-catalog.tsv")
    assert catalog_sha == compilation_pins['research/integration/runtime-resource-catalog.tsv']
    (work / 'runtime-inputs.json').write_bytes(input_bytes)
    cases = [run_case(binary_copy, work, "headless-a",steps=args.steps,replay_text=replay_text), run_case(binary_copy, work, "headless-b",steps=args.steps,replay_text=replay_text)]
    if cases[0]["world_sha256"] != cases[1]["world_sha256"] or cases[0]["bmp_sha256"] != cases[1]["bmp_sha256"]:
        raise AssertionError("repeated headless family8 replay was not deterministic")
    cases.append(run_case(binary_copy, work, "window-3frames",steps=args.steps,replay_text=replay_text, window=True))
    if (cases[2]["world_sha256"], cases[2]["bmp_sha256"]) != (cases[0]["world_sha256"], cases[0]["bmp_sha256"]):
        raise AssertionError("bounded window replay differs from the headless final frame")
    payload = {
        "schema_version": 1,
        "status": "passed",
        "verified_at_utc": datetime.now(timezone.utc).isoformat(),
        "scope": f"Same copied release binary replays menu load, {args.steps:,} landing-study steps, and actor selection; repeated headless result equality plus a three-frame native window presentation. Not an original executable run or full-game claim.",
        "binary": {"source_path": BINARY.relative_to(ROOT).as_posix(), "copied_path": binary_copy.relative_to(ROOT).as_posix(), "sha256": binary_sha, "size": binary_copy.stat().st_size, "profile": "release"},
        "source_pins_at_run": source_pins,
        "compilation_source_pins": compilation_pins,
        "resource_catalog_sha256": catalog_sha,
        "runtime_inputs_receipt_sha256": sha_bytes(input_bytes),
        "runtime_resource_count": len(inputs["resources"]),
        "replay": {"seed": 5428, "commands": commands, "steps": args.steps},
        "results": {"deterministic_headless_repeat": True, "window_matches_headless": True,
                    "family8_native_dispatch": "current admitted family8 callbacks; rendering is included in final RGB/BMP hash; no per-slot telemetry is asserted",
                    "cases": cases},
        "original_execution": False,
        "private_run_directory": work.relative_to(ROOT).as_posix(),
    }
    receipt = args.output.resolve()
    assert receipt.is_relative_to(ROOT.resolve())
    encoded = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if receipt.name=='family8-native-validation.json' and receipt.exists():
        raise FileExistsError('preserve the dated native receipt; use a current or new output path')
    temporary=receipt.with_suffix('.json.tmp');temporary.write_text(encoded,encoding='utf-8',newline='\n');temporary.replace(receipt)
    print(json.dumps({"receipt": receipt.relative_to(ROOT).as_posix(), "binary_sha256": binary_sha,
                      "cases": [{k: v for k, v in case.items() if k not in ("report_path",)} for case in cases]}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
