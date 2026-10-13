"""Launch the bounded native asset inspector for a fixed number of frames."""
import argparse
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("panel_rnc", type=Path)
    parser.add_argument("main_pal", type=Path)
    parser.add_argument("report_json", type=Path)
    parser.add_argument("--frames", type=int, default=3)
    args = parser.parse_args()
    if args.frames < 1:
        parser.error("--frames must be positive")
    panel = args.panel_rnc.absolute()
    palette = args.main_pal.absolute()
    report = args.report_json.absolute()
    command = [
        sys.executable, str(ROOT / "scripts" / "cargo.py"), "run", "--offline", "--locked", "-p", "genewars-desktop", "--",
        "panel", str(panel), str(palette), "--frames", str(args.frames), "--report", str(report),
    ]
    try:
        result = subprocess.run(command, cwd=ROOT, timeout=30)
    except subprocess.TimeoutExpired:
        raise SystemExit("native smoke launch exceeded 30 seconds")
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
