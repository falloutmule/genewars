"""Build and open the Rust partial world with privately staged exact CD resources."""
from pathlib import Path
import subprocess
import sys
from runtime_inputs import ROOT, stage


def main():
    directory, _ = stage()
    command = [sys.executable, '-B', str(ROOT/'scripts/cargo.py'), 'build', '-p', 'genewars-desktop',
               '--bin', 'genewars-game', '--release', '--offline', '--locked']
    result = subprocess.run(command, cwd=ROOT)
    if result.returncode:
        return result.returncode
    binary = ROOT/'private/build/rust-target/release/genewars-game.exe'
    return subprocess.run([str(binary), str(directory), *sys.argv[1:]], cwd=ROOT).returncode


if __name__ == '__main__':
    raise SystemExit(main())
