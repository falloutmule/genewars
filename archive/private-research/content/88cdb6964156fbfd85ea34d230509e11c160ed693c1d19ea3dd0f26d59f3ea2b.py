"""Run Cargo with project-private tool homes and build outputs; no global settings."""
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
TOOLCHAIN = '1.99.0-x86_64-pc-windows-gnu'
TARGET = 'x86_64-pc-windows-gnu'


def environment():
    tools = ROOT / 'private/tools/rust'
    env = os.environ.copy()
    env.update(RUSTUP_HOME=str(tools / 'rustup'), CARGO_HOME=str(tools / 'cargo'),
               RUSTUP_TOOLCHAIN=TOOLCHAIN,
               CARGO_TARGET_DIR=str(ROOT / 'private/build/rust-target'))
    # windows-sys raw-dylib imports require dlltool on PATH. The private LLVM
    # alias avoids the bundled GNU dlltool's unavailable assembler subprocess.
    llvm_tools = ROOT / 'private/tools/llvm-dlltool/1.99.0'
    gnu_tools = tools / 'rustup/toolchains' / TOOLCHAIN / 'lib/rustlib' / TARGET / 'bin/self-contained'
    env['PATH'] = os.pathsep.join((str(tools / 'cargo/bin'), str(llvm_tools), str(gnu_tools), env.get('PATH', '')))
    return env


if __name__ == '__main__':
    cargo = ROOT / 'private/tools/rust/cargo/bin/cargo.exe'
    if not cargo.is_file():
        raise SystemExit('Project-local Rust toolchain is not installed')
    raise SystemExit(subprocess.run([str(cargo), *sys.argv[1:]], cwd=ROOT / 'rust', env=environment()).returncode)
