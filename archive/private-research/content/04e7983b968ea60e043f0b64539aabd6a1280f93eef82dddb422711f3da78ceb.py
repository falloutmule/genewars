#!/usr/bin/env python3
"""Rerun only checks marked false in the preserved 2026-10-13 aggregate."""
from __future__ import annotations

import hashlib
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
BASELINE = ROOT / "research/integration/verification-20261013-214905.json"
DEFAULT_OUTDIR = ROOT / "private/integration/verification-rerun-20261013-214905"
DEFAULT_RECEIPT = ROOT / "research/integration/verification-rerun-20261013-214905.json"


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def source_pins() -> dict[str, object]:
    """Hash executable/source inputs; private generated evidence is excluded."""
    roots = [ROOT / "rust", ROOT / "scripts", ROOT / "research/variants", ROOT / "research/ghidra"]
    suffixes = {".rs", ".py", ".toml", ".lock", ".java", ".cs", ".ps1", ".sh"}
    entries = []
    for base in roots:
        if not base.exists():
            continue
        for path in sorted(base.rglob("*")):
            if not path.is_file() or "private" in path.parts or path.suffix.lower() not in suffixes:
                continue
            rel = path.relative_to(ROOT).as_posix()
            entries.append((rel, sha(path)))
    h = hashlib.sha256()
    for rel, digest in entries:
        h.update(rel.encode("utf-8")); h.update(b"\0"); h.update(digest.encode("ascii")); h.update(b"\n")
    return {"files": len(entries), "sha256": h.hexdigest(), "entries": entries}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--labels", nargs="*", help="rerun only these failed-check labels")
    parser.add_argument("--output", help="write a separate receipt at this repository-relative path")
    args = parser.parse_args()
    outdir = DEFAULT_OUTDIR if not args.labels else ROOT / "private/integration/verification-rerun-inconclusive-20261013"
    receipt_path = ROOT / args.output if args.output else DEFAULT_RECEIPT
    if not BASELINE.is_file():
        raise SystemExit(f"missing preserved aggregate: {BASELINE}")
    baseline = json.loads(BASELINE.read_text(encoding="utf-8"))
    failed = {name for name, passed in baseline["checks"].items() if passed is False}
    commands = [entry for entry in baseline["commands"] if entry.get("label") in failed]
    if {entry["label"] for entry in commands} != failed:
        missing = sorted(failed - {entry["label"] for entry in commands})
        raise SystemExit(f"failed checks without command records: {missing}")
    if args.labels:
        selected = set(args.labels)
        unknown = selected - {entry["label"] for entry in commands}
        if unknown:
            raise SystemExit(f"labels are not failed checks in the baseline: {sorted(unknown)}")
        commands = [entry for entry in commands if entry["label"] in selected]
    outdir.mkdir(parents=True, exist_ok=True)
    results = []
    for idx, entry in enumerate(commands, 1):
        label = entry["label"]
        argv = entry.get("command")
        if not argv:
            raise SystemExit(f"no command for failed check {label}")
        before = source_pins()
        started = time.time()
        try:
            proc = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, timeout=900)
            exit_code = proc.returncode
            stdout, stderr = proc.stdout, proc.stderr
            timed_out = False
        except subprocess.TimeoutExpired as exc:
            exit_code = None
            stdout = exc.stdout.decode(errors="replace") if isinstance(exc.stdout, bytes) else (exc.stdout or "")
            stderr = exc.stderr.decode(errors="replace") if isinstance(exc.stderr, bytes) else (exc.stderr or "")
            timed_out = True
        after = source_pins()
        before_entries = dict(before["entries"])
        after_entries = dict(after["entries"])
        changed_paths = sorted(
            name for name in set(before_entries) | set(after_entries)
            if before_entries.get(name) != after_entries.get(name)
        )
        safe_label = re.sub(r"[^A-Za-z0-9_.-]+", "_", label)
        log = outdir / f"{idx:02d}-{safe_label}.log"
        log.write_text(
            f"LABEL: {label}\nCOMMAND: {json.dumps(argv, ensure_ascii=False)}\nEXIT_CODE: {exit_code}\n"
            f"TIMED_OUT: {timed_out}\nSOURCE_BEFORE: {before['sha256']}\nSOURCE_AFTER: {after['sha256']}\n\n"
            f"--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}", encoding="utf-8"
        )
        results.append({
            "label": label, "command": argv, "exit_code": exit_code, "timed_out": timed_out,
            "source_before_sha256": before["sha256"], "source_after_sha256": after["sha256"],
            "source_stable": before["sha256"] == after["sha256"],
            "changed_source_paths": changed_paths,
            "source_file_count_before": before["files"], "source_file_count_after": after["files"],
            "log": log.relative_to(ROOT).as_posix(), "duration_seconds": round(time.time() - started, 3),
        })
        print(json.dumps({"index": idx, "total": len(commands), "label": label,
                          "exit_code": exit_code, "timed_out": timed_out,
                          "source_stable": before["sha256"] == after["sha256"]}, separators=(",", ":")), flush=True)
    receipt = {
        "schema_version": 1,
        "baseline": BASELINE.relative_to(ROOT).as_posix(),
        "baseline_sha256": sha(BASELINE),
        "baseline_completed_utc": baseline.get("completed_utc"),
        "failed_check_count": len(failed),
        "rerun_count": len(results),
        "scope": "Sequential rerun of checks marked false in the preserved aggregate; does not claim the full aggregate is green.",
        "results": results,
    }
    tmp = receipt_path.with_suffix(".tmp")
    tmp.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    os.replace(tmp, receipt_path)
    print(json.dumps({"receipt": receipt_path.relative_to(ROOT).as_posix(), "failed": sum(x["exit_code"] != 0 or x["timed_out"] for x in results), "inconclusive": sum(not x["source_stable"] for x in results)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
