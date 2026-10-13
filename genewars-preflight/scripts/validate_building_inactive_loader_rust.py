#!/usr/bin/env python3
"""Compile the production inactive building loader in an isolated Rust harness.

The harness imports the actual checked-in building_pool.rs and compares its
packed-to-native results and rejection paths with the pinned instruction model.
No Cargo workspace files or production sources are modified.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]
POOL = ROOT / "rust/crates/genewars-runtime/src/building_pool.rs"
BUILDING = ROOT / "rust/crates/genewars-runtime/src/building.rs"
BASE_VALIDATOR = ROOT / "scripts/validate_building_inactive_loader.py"
PROOF_SCRIPT = Path(__file__).resolve()
LOAD45010 = ROOT / "private/ghidra/mission-tail-readers/instructions-00045010.txt"
COPY6D490 = ROOT / "private/ghidra/mission-tail-readers/instructions-0006d490.txt"
MISSION = ROOT / "private/integration/game-runtime/terrain/CONQ001.DAT"
MISSION_PROOF = ROOT / "research/ghidra/mission-load-evidence.json"
RUSTC = ROOT / "private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe"
OUT = ROOT / "research/formats/building-inactive-loader-rust-proof.json"
SCRATCH = ROOT / "private/review/building-inactive-loader-rust"

PACKED_SIZE = 59
NATIVE_SIZE = 67
COUNT = 256
GROUP_OFFSET = 301_630
MISSION_SHA = "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2"

HARNESS = r'''use std::io::{self, Read, Write};
#[path="building.rs"] mod building;
#[path="building_pool.rs"] mod building_pool;
use building::BuildingAnimationBinding;
use building_pool::{BuildingPool, BuildingPoolError};

fn take<'a>(b: &'a [u8], p: &mut usize, n: usize) -> &'a [u8] {
    let e = p.checked_add(n).expect("input offset overflow");
    let s = b.get(*p..e).expect("truncated harness input"); *p = e; s
}
fn u32le(b: &[u8], p: &mut usize) -> usize {
    u32::from_le_bytes(take(b,p,4).try_into().unwrap()) as usize
}
fn main() {
    let mut input=Vec::new(); io::stdin().read_to_end(&mut input).unwrap();
    let mut p=0usize; let cases=u32le(&input,&mut p); let mut out=io::BufWriter::new(io::stdout().lock());
    for _ in 0..cases {
        let plen=u32le(&input,&mut p); let packed=take(&input,&mut p,plen);
        let prior_bytes=take(&input,&mut p,256*67);
        let bindings_bytes=take(&input,&mut p,256);
        let prior:[[u8;67];256]=std::array::from_fn(|i| prior_bytes[i*67..(i+1)*67].try_into().unwrap());
        let bindings:[Option<BuildingAnimationBinding>;256]=std::array::from_fn(|i| {
            if bindings_bytes[i]==0 {None} else {Some(BuildingAnimationBinding{selector:bindings_bytes[i]})}
        });
        match BuildingPool::from_packed_inactive(packed,&prior,bindings) {
            Ok(pool) => { out.write_all(&[0]).unwrap(); for r in pool.records() {out.write_all(r).unwrap();} }
            Err(BuildingPoolError::PackedLength{..}) => out.write_all(&[1]).unwrap(),
            Err(BuildingPoolError::ActivePackedRecord(i)) => {out.write_all(&[2]).unwrap();out.write_all(&(i as u16).to_le_bytes()).unwrap();}
            Err(BuildingPoolError::InvalidBinding(i)) => {out.write_all(&[3]).unwrap();out.write_all(&(i as u16).to_le_bytes()).unwrap();}
            Err(_) => out.write_all(&[4]).unwrap(),
        }
    }
    assert_eq!(p,input.len(),"unused harness input"); out.flush().unwrap();
}
'''


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def pin(path: Path) -> dict[str, str]:
    return {"path": path.relative_to(ROOT).as_posix(), "sha256": sha(path.read_bytes())}


def load_base_module():
    spec = importlib.util.spec_from_file_location("inactive_loader_reference", BASE_VALIDATOR)
    module = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(module)
    return module


def case_bytes(packed: bytes, prior_records: list[bytes], bindings: bytes) -> bytes:
    assert len(prior_records) == COUNT and len(bindings) == COUNT
    return struct.pack("<I", len(packed)) + packed + b"".join(prior_records) + bindings


def literal_record(model, code, packed: bytes, prior: bytes) -> tuple[bytes, bytes]:
    out, staged, _ = model.run_record(code, packed, prior)
    return out, staged


def private_rust_run(exe: Path, cases: list[tuple[bytes, list[bytes], bytes]]) -> list[tuple[int, bytes]]:
    payload = bytearray(struct.pack("<I", len(cases)))
    for packed, priors, bindings in cases:
        payload.extend(case_bytes(packed, priors, bindings))
    proc = subprocess.run([str(exe)], input=payload, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, check=False, timeout=120)
    if proc.returncode:
        raise RuntimeError(f"private Rust harness failed: {proc.stderr.decode(errors='replace')[:1000]}")
    cursor = 0
    replies = []
    for _ in cases:
        code = proc.stdout[cursor]; cursor += 1
        if code == 0:
            n = COUNT * NATIVE_SIZE
            replies.append((code, proc.stdout[cursor:cursor+n])); cursor += n
        elif code in (2, 3):
            slot = int.from_bytes(proc.stdout[cursor:cursor+2], "little"); cursor += 2
            replies.append((code, struct.pack("<H", slot)))
        else:
            replies.append((code, b""))
    if cursor != len(proc.stdout):
        raise RuntimeError("private Rust harness emitted unexpected trailing bytes")
    return replies


def expected_batch(model, code, packed: bytes, priors: list[bytes]) -> tuple[bytes, bytes]:
    out = bytearray()
    staged = bytearray()
    for slot in range(COUNT):
        source = packed[slot*PACKED_SIZE:(slot+1)*PACKED_SIZE]
        native, post = literal_record(model, code, source, priors[slot])
        out.extend(native)
        staged.extend(post)
    return bytes(out), bytes(staged)


def main() -> None:
    model = load_base_module()
    source_paths = (POOL, BUILDING, BASE_VALIDATOR, PROOF_SCRIPT, LOAD45010, COPY6D490, MISSION,
                    MISSION_PROOF, RUSTC)
    before = [pin(p) for p in source_paths]
    if not RUSTC.is_file() or RUSTC.stat().st_size == 0:
        raise SystemExit(f"private rustc unavailable: {RUSTC}")
    SCRATCH.mkdir(parents=True, exist_ok=True)
    (SCRATCH / "building.rs").write_bytes(BUILDING.read_bytes())
    shutil.copyfile(POOL, SCRATCH / "building_pool.rs")
    (SCRATCH / "main.rs").write_text(HARNESS, encoding="utf-8", newline="\n")
    production_pool_hash = sha((SCRATCH / "building_pool.rs").read_bytes())
    compile_command = [str(RUSTC), "--edition=2021", "--crate-name", "building_inactive_private_probe",
                       str(SCRATCH / "main.rs"), "-o", str(SCRATCH / "probe.exe"), "-C", "opt-level=2"]
    env = os.environ.copy()
    tools_path = ROOT / "private/tools/rust/cargo/bin"
    toolchain_bin = ROOT / "private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin"
    env["PATH"] = os.pathsep.join((str(tools_path), str(toolchain_bin), env.get("PATH", "")))
    compiled = subprocess.run(compile_command, cwd=SCRATCH, env=env, capture_output=True, text=True, timeout=120)
    if compiled.returncode:
        raise RuntimeError("private rustc compile failed:\n" + compiled.stderr[-3000:])

    code = model.load_listing(LOAD45010) | model.load_listing(COPY6D490)
    mission = MISSION.read_bytes()
    if len(mission) != 343_958 or sha(mission) != MISSION_SHA:
        raise RuntimeError("CONQ001 identity mismatch")
    real_packed = mission[GROUP_OFFSET:GROUP_OFFSET + COUNT * PACKED_SIZE]
    if len(real_packed) != COUNT * PACKED_SIZE or any(real_packed[i*PACKED_SIZE+6] for i in range(COUNT)):
        raise RuntimeError("CONQ001 family-3 inactive-group admission changed")
    real_prior = [bytes((slot * 29 + j * 17 + 0xA5) & 0xff for j in range(NATIVE_SIZE)) for slot in range(COUNT)]
    real_case = (real_packed, real_prior, bytes(COUNT))

    # Four complete pools provide 1,024 independent patterned packed records.
    # The literal interpreter executes on all 256 records in every pool, so
    # every byte of each Rust pool result has a listing-derived comparator.
    import random
    rng = random.Random(0x450106D490)
    synthetic_cases = []
    for batch in range(4):
        records = bytearray()
        for slot in range(COUNT):
            packed_row = bytearray(rng.randrange(256) for _ in range(PACKED_SIZE))
            packed_row[6] = 0
            case = batch * COUNT + slot
            if case % 4 == 0: packed_row[0x15] = 0xff
            if case % 4 == 1: packed_row[0x17] = 0
            records.extend(packed_row)
        priors = [bytes(rng.randrange(256) for _ in range(NATIVE_SIZE)) for _ in range(COUNT)]
        synthetic_cases.append((bytes(records), priors, bytes(COUNT)))

    negative_cases = []
    # Wrong length is a real method call, not a source-code guard string.
    negative_cases.append((real_packed[:-1], real_prior, bytes(COUNT)))
    active = bytearray(real_packed); active[17*PACKED_SIZE+6] = 4
    negative_cases.append((bytes(active), real_prior, bytes(COUNT)))
    stale = bytearray(COUNT); stale[91] = 3
    negative_cases.append((real_packed, real_prior, bytes(stale)))

    all_cases = [real_case, *synthetic_cases, *negative_cases]
    replies = private_rust_run(SCRATCH / "probe.exe", all_cases)
    if replies[0][0] != 0:
        raise RuntimeError(f"real CONQ001 rejected with private Rust status {replies[0][0]}")
    expected_real, _ = expected_batch(model, code, real_packed, real_prior)
    if replies[0][1] != expected_real:
        mismatch = next(i for i,(a,b) in enumerate(zip(replies[0][1],expected_real)) if a!=b)
        raise RuntimeError(f"real Rust/interpreter mismatch byte {mismatch}")

    synthetic_outputs = hashlib.sha256()
    literal_rows = 0
    for i, ((status, output), (packed, priors, _)) in enumerate(zip(replies[1:1+len(synthetic_cases)], synthetic_cases)):
        if status != 0:
            raise RuntimeError(f"synthetic Rust case {i} unexpectedly rejected ({status})")
        expected, _ = expected_batch(model, code, packed, priors)
        if output != expected:
            mismatch = next(j for j,(a,b) in enumerate(zip(output,expected)) if a!=b)
            raise RuntimeError(f"synthetic case {i} full-pool Rust/interpreter mismatch byte {mismatch}")
        synthetic_outputs.update(output)
        literal_rows += COUNT

    neg = replies[1+len(synthetic_cases):]
    expected_errors = [(1,b""),(2,struct.pack("<H",17)),(3,struct.pack("<H",91))]
    if neg != expected_errors:
        raise RuntimeError(f"Rust rejection mismatch: {neg!r}")

    after = [pin(p) for p in source_paths]
    if after != before:
        raise RuntimeError("a source pin changed during validation")
    report = {
        "schema_version": 1,
        "status": "passed-private-rust-inactive-loader-crosscheck",
        "scope": "Actual checked-in BuildingPool source copied unchanged into an isolated rustc harness; no production/Cargo files changed.",
        "method": "rustc compiled actual production source; result bytes and rejection variants compared against literal 45010/6D490 instruction interpreter.",
        "production_pool_source_sha256": production_pool_hash,
        "compiled_harness_sha256": sha((SCRATCH / "probe.exe").read_bytes()),
        "private_rustc_sha256": before[-1]["sha256"],
        "real_conq001": {"records": COUNT, "packed_bytes": len(real_packed), "native_bytes_compared": len(expected_real),
                         "complete_byte_match": True, "output_sha256": sha(replies[0][1])},
        "synthetic": {"complete_pool_cases": len(synthetic_cases), "packed_records": literal_rows, "rust_successes": len(synthetic_cases),
                      "literal_interpreter_varied_records": literal_rows,
                      "complete_pool_outputs_compared": len(synthetic_cases),
                      "aggregate_output_sha256": synthetic_outputs.hexdigest()},
        "rust_rejections": {"wrong_length": neg[0][0] == 1, "active_record_slot_17": neg[1] == (2, struct.pack("<H",17)),
                            "stale_binding_slot_91": neg[2] == (3, struct.pack("<H",91))},
        "source_pins_before_and_after_equal": True,
        "source_pins": before,
        "original_executable_run": False,
        "private_record_bytes_written_to_public_report": False,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    temp = OUT.with_name(f".{OUT.name}.{uuid.uuid4().hex}.tmp")
    try:
        temp.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
        os.replace(temp, OUT)
    finally:
        if temp.exists(): temp.unlink()
    print(json.dumps({"status": report["status"], "real_records": COUNT,
                      "synthetic_cases": len(synthetic_cases), "rejections": report["rust_rejections"],
                      "output": OUT.relative_to(ROOT).as_posix()}, indent=2))


if __name__ == "__main__":
    main()
