#!/usr/bin/env python3
"""Run the production Rust BTURR selector-1 loader and exact-hold tests."""
from __future__ import annotations

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CARGO = ROOT / "scripts/cargo.py"

command = [sys.executable, "-B", str(CARGO), "test", "--offline", "--locked",
           "-p", "genewars-runtime", "building_resources::tests", "--", "--nocapture"]
result = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE,
                        stderr=subprocess.STDOUT, timeout=300, check=False)
sys.stdout.write(result.stdout)
if result.returncode:
    raise SystemExit(result.returncode)
if "cd_bturr_animation_zero_has_exact_three_zero_holds ... ok" not in result.stdout:
    raise SystemExit("production Rust loader did not run the exact BTURR three-frame hold test")
if "bturr_incomplete_common_path_draws_all_three_frames_and_clips_atomically ... ok" not in result.stdout:
    raise SystemExit("production Rust BTURR common-path renderer test did not run")
print("BTURR production loader and common-path render gate passed")
