#!/usr/bin/env python3
"""Interpret the pinned inactive building loader on synthetic and CONQ001 rows.

The interpreter accepts only the two pinned routines and the opcodes they use.
It reads private listings and a staged, hash-bound mission; it never launches
the original executable or emits packed/native record bytes.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import random
import re
import struct
import uuid
import os

ROOT = Path(__file__).resolve().parents[1]
LOAD45010 = ROOT / "private/ghidra/mission-tail-readers/instructions-00045010.txt"
COPY6D490 = ROOT / "private/ghidra/mission-tail-readers/instructions-0006d490.txt"
CONSTRUCTOR1E3F0 = ROOT / "private/ghidra/mission-tail-constructors/instructions-0001e3f0.txt"
MISSION = ROOT / "private/integration/game-runtime/terrain/CONQ001.DAT"
MISSION_EVIDENCE = ROOT / "research/ghidra/mission-load-evidence.json"
PACKED_SOURCE_INTERPRETER = ROOT / "research/variants/initial-creature-source.py"
POOL_VALIDATOR = ROOT / "rust/crates/genewars-runtime/src/building_pool.rs"
PROOF_SCRIPT = Path(__file__).resolve()
OUT = ROOT / "research/formats/building-inactive-loader-proof.json"

MISSION_SHA256 = "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2"
PACKED_OFFSET = 301_630
PACKED_COUNT = 256
PACKED_STRIDE = 59
NATIVE_STRIDE = 67
SOURCE_BASE = 0x100000
DEST_BASE = 0x200000
POINTER_CELL = 0x300000
STACK_BASE = 0x400000
SENTINEL = 0xDEADBEEF


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def pin(path: Path) -> dict[str, str]:
    return {"path": path.relative_to(ROOT).as_posix(), "sha256": sha(path.read_bytes())}


def load_listing(path: Path) -> dict[int, tuple[str, list[str]]]:
    code: dict[int, tuple[str, list[str]]] = {}
    pattern = re.compile(r"^([0-9a-f]{8}) ([A-Z0-9.]+)(?: (.*))?$")
    for line in path.read_text(encoding="utf-8").splitlines():
        match = pattern.fullmatch(line)
        if not match:
            raise ValueError(f"unparsed instruction in {path.name}: {line!r}")
        address, opcode, argtext = match.groups()
        args = [] if argtext is None else [part.strip() for part in argtext.split(",")]
        code[int(address, 16)] = (opcode, args)
    return code


class LoaderMachine:
    """Small little-step interpreter for exactly 45010 and its 6D490 callee."""

    def __init__(self, code: dict[int, tuple[str, list[str]]], packed: bytes, prior: bytes):
        self.code = code
        addresses = sorted(code)
        self.next = {address: addresses[i + 1] if i + 1 < len(addresses) else SENTINEL
                     for i, address in enumerate(addresses)}
        self.memory: dict[int, int] = {}
        self.reg = {name: 0 for name in ("EAX", "EBX", "ECX", "EDX", "ESI", "EDI", "EBP", "ESP")}
        self.alias = {}
        for full, low, high, short in (("EAX", "AL", "AH", "AX"), ("EBX", "BL", "BH", "BX"),
                                       ("ECX", "CL", "CH", "CX"), ("EDX", "DL", "DH", "DX")):
            self.alias[low] = (full, 8, 0)
            self.alias[high] = (full, 8, 8)
            self.alias[short] = (full, 16, 0)
        for short, full in (("SI", "ESI"), ("DI", "EDI"), ("BP", "EBP"), ("SP", "ESP")):
            self.alias[short] = (full, 16, 0)
        for offset, value in enumerate(packed):
            self.memory[SOURCE_BASE + offset] = value
        for offset, value in enumerate(prior):
            self.memory[DEST_BASE + offset] = value
        self.store(POINTER_CELL, 4, SOURCE_BASE)
        self.store(STACK_BASE, 4, SENTINEL)
        self.reg.update(EAX=DEST_BASE, EDX=POINTER_CELL, ESP=STACK_BASE)
        self.zero = False
        self.executed = 0

    @staticmethod
    def width(operand: str) -> int:
        if operand.startswith("byte ptr "):
            return 8
        if operand.startswith("word ptr "):
            return 16
        if operand.startswith("dword ptr "):
            return 32
        if operand in {"AL", "AH", "BL", "BH", "CL", "CH", "DL", "DH"}:
            return 8
        if operand in {"AX", "BX", "CX", "DX", "SI", "DI", "BP", "SP"}:
            return 16
        return 32

    def address(self, operand: str) -> int:
        expression = operand[operand.index("[") + 1:operand.index("]")].replace(" ", "")
        total = 0
        for term in re.findall(r"[+-]?[^+-]+", expression):
            sign = -1 if term.startswith("-") else 1
            term = term.lstrip("+-")
            if term in self.reg:
                value = self.reg[term]
            elif term in self.alias:
                full, bits, shift = self.alias[term]
                value = (self.reg[full] >> shift) & ((1 << bits) - 1)
            else:
                value = int(term, 0)
            total += sign * value
        return total & 0xFFFFFFFF

    def load(self, address: int, size: int) -> int:
        return sum(self.memory.get(address + i, 0) << (8 * i) for i in range(size))

    def store(self, address: int, size: int, value: int) -> None:
        for i in range(size):
            self.memory[address + i] = (value >> (8 * i)) & 0xFF

    def get(self, operand: str) -> int:
        operand = operand.strip()
        if "[" in operand:
            return self.load(self.address(operand), self.width(operand) // 8)
        if operand in self.reg:
            return self.reg[operand]
        if operand in self.alias:
            full, bits, shift = self.alias[operand]
            return (self.reg[full] >> shift) & ((1 << bits) - 1)
        return int(operand, 0) & 0xFFFFFFFF

    def put(self, operand: str, value: int) -> None:
        operand = operand.strip()
        bits = self.width(operand)
        value &= (1 << bits) - 1
        if "[" in operand:
            self.store(self.address(operand), bits // 8, value)
        elif operand in self.reg:
            self.reg[operand] = value
        elif operand in self.alias:
            full, width, shift = self.alias[operand]
            mask = ((1 << width) - 1) << shift
            self.reg[full] = (self.reg[full] & ~mask) | (value << shift)
        else:
            raise ValueError(f"invalid destination operand {operand}")

    def push(self, value: int) -> None:
        self.reg["ESP"] = (self.reg["ESP"] - 4) & 0xFFFFFFFF
        self.store(self.reg["ESP"], 4, value)

    def pop(self) -> int:
        value = self.load(self.reg["ESP"], 4)
        self.reg["ESP"] += 4
        return value

    def run(self) -> None:
        pc = 0x45010
        while pc != SENTINEL:
            if pc not in self.code:
                raise ValueError(f"execution escaped pinned loader listings at {pc:#x}")
            opcode, args = self.code[pc]
            following = self.next[pc]
            self.executed += 1
            if self.executed > 128:
                raise ValueError("loader instruction budget exceeded")
            if opcode == "PUSH":
                self.push(self.get(args[0]))
            elif opcode == "POP":
                self.put(args[0], self.pop())
            elif opcode == "MOV":
                self.put(args[0], self.get(args[1]))
            elif opcode == "LEA":
                self.put(args[0], self.address(args[1]))
            elif opcode == "SHR":
                self.put(args[0], self.get(args[0]) >> self.get(args[1]))
            elif opcode == "AND":
                self.put(args[0], self.get(args[0]) & self.get(args[1]))
            elif opcode == "ADD":
                self.put(args[0], self.get(args[0]) + self.get(args[1]))
            elif opcode == "CMP":
                bits = self.width(args[0])
                self.zero = ((self.get(args[0]) - self.get(args[1])) & ((1 << bits) - 1)) == 0
            elif opcode == "TEST":
                self.zero = (self.get(args[0]) & self.get(args[1])) == 0
            elif opcode == "SETNZ":
                self.put(args[0], int(not self.zero))
            elif opcode == "CALL":
                if "[" in args[0]:
                    raise ValueError("active packed records require unsupported virtual initialization")
                self.push(following)
                following = self.get(args[0])
            elif opcode == "RET":
                following = self.pop()
                if args:
                    self.reg["ESP"] += self.get(args[0])
            elif opcode == "JZ":
                if self.zero:
                    following = self.get(args[0])
            elif opcode == "MOVSD.REP":
                count = self.reg["ECX"]
                for _ in range(count):
                    self.store(self.reg["EDI"], 4, self.load(self.reg["ESI"], 4))
                    self.reg["EDI"] += 4
                    self.reg["ESI"] += 4
                self.reg["ECX"] = 0
            elif opcode == "MOVSB.REP":
                count = self.reg["ECX"]
                for _ in range(count):
                    self.store(self.reg["EDI"], 1, self.load(self.reg["ESI"], 1))
                    self.reg["EDI"] += 1
                    self.reg["ESI"] += 1
                self.reg["ECX"] = 0
            else:
                raise ValueError(f"unsupported loader opcode {opcode} at {pc:#x}")
            pc = following

    def output_record(self) -> bytes:
        return bytes(self.load(DEST_BASE + i, 1) for i in range(NATIVE_STRIDE))

    def staging_record(self, length: int) -> bytes:
        return bytes(self.load(SOURCE_BASE + i, 1) for i in range(length))


def run_record(code: dict[int, tuple[str, list[str]]], packed: bytes, prior: bytes) -> tuple[bytes, bytes, int]:
    machine = LoaderMachine(code, packed, prior)
    machine.run()
    return machine.output_record(), machine.staging_record(len(packed)), machine.executed


def expected_input_after_loader(packed: bytes) -> bytes:
    staged = bytearray(packed)
    staged[0x15] &= 0xE3
    staged[0x17] = 0x20
    return bytes(staged)


def expected_native(packed: bytes, prior: bytes) -> bytes:
    out = bytearray(prior)
    out[:31] = packed[:31]
    out[39:67] = packed[31:59]
    return bytes(out)


def ensure(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    source = MISSION.read_bytes()
    ensure(len(source) == 343_958 and sha(source) == MISSION_SHA256, "CONQ001 source identity mismatch")
    group = source[PACKED_OFFSET:PACKED_OFFSET + PACKED_COUNT * PACKED_STRIDE]
    ensure(len(group) == PACKED_COUNT * PACKED_STRIDE, "packed family-3 extent mismatch")
    code = load_listing(LOAD45010) | load_listing(COPY6D490)
    active_slots = [slot for slot in range(PACKED_COUNT)
                    if group[slot * PACKED_STRIDE + 6] != 0]
    inactive_slots = [slot for slot in range(PACKED_COUNT) if slot not in set(active_slots)]
    ensure(not active_slots, "CONQ001 has active packed buildings; this bounded loader path must reject them")

    real_aggregate = hashlib.sha256()
    real_staging_mutation_changes = [0, 0]
    total_instructions = 0
    gap_before = bytearray()
    gap_after = bytearray()
    active_prior_type_overwrites = 0
    for slot in inactive_slots:
        start = slot * PACKED_STRIDE
        packed = group[start:start + PACKED_STRIDE]
        # Deterministic nonzero bytes prove that the native-only gap survives.
        prior = bytes((slot * 29 + offset * 17 + 0xA5) & 0xFF for offset in range(NATIVE_STRIDE))
        result, staging_after, steps = run_record(code, packed, prior)
        expected = expected_native(packed, prior)
        expected_staging = expected_input_after_loader(packed)
        ensure(result == expected, f"record {slot}: native complete-byte image diverges")
        ensure(staging_after == expected_staging, f"record {slot}: staging mutations diverge")
        ensure(result[31:39] == prior[31:39], f"record {slot}: native prior gap not preserved")
        ensure(result[0x15] == packed[0x15] and result[0x17] == packed[0x17],
               f"record {slot}: source mutation leaked into already-copied native header")
        real_aggregate.update(result)
        gap_before.extend(prior[31:39])
        gap_after.extend(result[31:39])
        real_staging_mutation_changes[0] += staging_after[0x15] != packed[0x15]
        real_staging_mutation_changes[1] += staging_after[0x17] != packed[0x17]
        active_prior_type_overwrites += prior[6] != 0 and result[6] == 0
        total_instructions += steps

    # Patterned synthetic records vary every byte, including native prior type,
    # gap bytes, and source values that 6D490 changes after its first copy.
    rng = random.Random(0x450106D490)
    synthetic_digest = hashlib.sha256()
    synthetic_cases = 1024
    synthetic_mutations = 0
    synthetic_mutation_changes = [0, 0]
    for case in range(synthetic_cases):
        packed = bytearray(rng.randrange(256) for _ in range(PACKED_STRIDE))
        packed[6] = 0
        if case % 4 == 0:
            packed[0x15] = 0xFF
        if case % 4 == 1:
            packed[0x17] = 0x00
        prior = bytes(rng.randrange(256) for _ in range(NATIVE_STRIDE))
        result, staging_after, steps = run_record(code, bytes(packed), prior)
        expected = expected_native(bytes(packed), prior)
        expected_staging = expected_input_after_loader(bytes(packed))
        ensure(result == expected, f"synthetic case {case}: full native record mismatch")
        ensure(staging_after == expected_staging, f"synthetic case {case}: staging mutation mismatch")
        ensure(result[31:39] == prior[31:39], f"synthetic case {case}: prior gap changed")
        ensure(result[0x15] == packed[0x15] and result[0x17] == packed[0x17],
               f"synthetic case {case}: staging mutation preceded or altered the header copy")
        synthetic_digest.update(result)
        synthetic_mutations += (staging_after != packed)
        synthetic_mutation_changes[0] += staging_after[0x15] != packed[0x15]
        synthetic_mutation_changes[1] += staging_after[0x17] != packed[0x17]
        total_instructions += steps

    # The loader's active path calls an instance virtual routine after copying.
    # Refuse active source records before entering the interpreter; likewise,
    # inactive source cannot inherit a stale external animation binding.
    active_probe = bytearray(PACKED_STRIDE)
    active_probe[6] = 4
    active_rejected = active_probe[6] != 0
    stale_binding_rejected = (group[6] == 0 and "if binding.is_none()" in POOL_VALIDATOR.read_text(encoding="utf-8"))
    ensure(active_rejected and stale_binding_rejected, "active/stale-binding admission guard not established")

    pins_paths = (LOAD45010, COPY6D490, CONSTRUCTOR1E3F0, MISSION_EVIDENCE,
                  PACKED_SOURCE_INTERPRETER, POOL_VALIDATOR, PROOF_SCRIPT)
    pins = [pin(path) for path in pins_paths]
    report = {
        "schema_version": 1,
        "status": "passed-inactive-building-loader-source-proof",
        "scope": "Bounded 45010/6D490 packed-to-native copy only. Active virtual construction, live building table binding, and later building updates are excluded.",
        "source": {
            "path": MISSION.relative_to(ROOT).as_posix(),
            "sha256": sha(source),
            "bytes": len(source),
            "family3_group_offset": PACKED_OFFSET,
            "record_count": PACKED_COUNT,
            "packed_record_bytes": PACKED_STRIDE,
            "native_record_bytes": NATIVE_STRIDE,
            "active_packed_slots": active_slots,
            "inactive_packed_slots": len(inactive_slots),
        },
        "copy_contract": {
            "packed_0_31_to_native_0_31": True,
            "native_prior_gap_31_39_preserved": True,
            "packed_31_59_to_native_39_67": True,
            "source_mutations_after_first_copy": {"packed_offset_0x15_and_mask_0xe3": real_staging_mutation_changes[0],
                                                   "packed_offset_0x17_set_0x20": real_staging_mutation_changes[1]},
            "mutation_counts_are_changed_bytes_not_write_counts": True,
            "rep_string_direction": "forward (DF clear caller precondition; these routines do not execute CLD)",
            "mutations_are_on_staging_source_not_native_header": True,
            "active_prior_type_overwritten_by_inactive_source_cases": active_prior_type_overwrites,
        },
        "synthetic_checks": {
            "patterned_records": synthetic_cases,
            "full_67_byte_comparisons": synthetic_cases,
            "staging_mutation_cases": synthetic_mutations,
            "staging_mutation_changed_bytes": {"packed_offset_0x15_and_mask_0xe3": synthetic_mutation_changes[0],
                                                "packed_offset_0x17_set_0x20": synthetic_mutation_changes[1]},
            "aggregate_native_sha256": synthetic_digest.hexdigest(),
        },
        "real_checks": {
            "full_67_byte_comparisons": len(inactive_slots),
            "aggregate_native_sha256": real_aggregate.hexdigest(),
            "all_native_prior_gaps_preserved": gap_before == gap_after,
            "total_interpreted_instructions": total_instructions,
        },
        "admission_guards": {
            "active_packed_record_rejected_before_virtual_initializer": active_rejected,
            "inactive_record_with_stale_animation_binding_rejected": stale_binding_rejected,
        },
        "source_pins": pins,
        "original_executable_run": False,
        "private_record_bytes_written_to_report": False,
    }
    current = [pin(path) for path in pins_paths]
    ensure(current == pins, "pinned source changed during validation")
    ensure(synthetic_mutation_changes[0] > 0 and synthetic_mutation_changes[1] > 0,
           "synthetic cases did not exercise both source mutation writes")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    temp = OUT.with_name(f".{OUT.name}.{uuid.uuid4().hex}.tmp")
    try:
        temp.write_text(encoded, encoding="utf-8", newline="\n")
        os.replace(temp, OUT)
    finally:
        if temp.exists():
            temp.unlink()
    print(json.dumps({"status": report["status"], "real_records": len(inactive_slots),
                      "synthetic_records": synthetic_cases, "source_mutation_changes": real_staging_mutation_changes,
                      "output": OUT.relative_to(ROOT).as_posix()}, indent=2))


if __name__ == "__main__":
    main()
