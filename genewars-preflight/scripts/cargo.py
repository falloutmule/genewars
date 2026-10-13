"""Run Cargo with project-private tool homes and build outputs; no global settings."""
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def environment():
    tools = ROOT / 'private/tools/rust'
    env = os.environ.copy()
    env.update(RUSTUP_HOME=str(tools / 'rustup'), CARGO_HOME=str(tools / 'cargo'),
               CARGO_TARGET_DIR=str(ROOT / 'private/build/rust-target'))
    env['PATH'] = str(tools / 'cargo/bin') + os.pathsep + env.get('PATH', '')
    return env


if __name__ == '__main__':
    cargo = ROOT / 'private/tools/rust/cargo/bin/cargo.exe'
    if not cargo.is_file():
        raise SystemExit('Project-local Rust toolchain is not installed')
    raise SystemExit(subprocess.run([str(cargo), *sys.argv[1:]], cwd=ROOT / 'rust', env=environment()).returncode)
