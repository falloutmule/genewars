"""Compare captured corrected landing callbacks with the literal Mule oracle.

The captures retain their own compilation pins. This verifies one callback per
seed, not a fresh replay of every preceding inter-family callback.
"""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import struct
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def pin(path):
    return {"path": path.relative_to(ROOT).as_posix(),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def main():
    cases = []
    for seed in (5428, 0, 65535):
        receipt = ROOT / f"research/integration/trajectory450-seed{seed}.json"
        compiled = json.loads(receipt.read_text(encoding="utf-8"))
        assert compiled["status"] == "passed" and compiled["completed_frames"] == 450
        work = ROOT / compiled["private_output"] / "frame450"
        before, after = work / "before-mule", work / "after-mule"
        for path in list(before.glob("*.bin")) + list(after.glob("*.bin")):
            if path.name.startswith("source-") or path.name.endswith("fixture.bin"):
                continue
            relative = path.relative_to(work.parent).as_posix()
            assert hashlib.sha256(path.read_bytes()).hexdigest() == compiled["captures"][relative]
        subprocess.run([sys.executable, "-B", "research/variants/creature-hungry-source.py", "450",
                        "--snapshot=" + before.relative_to(ROOT).as_posix()], cwd=ROOT,
                       capture_output=True, check=True, timeout=180)
        actual = (after / "creatures.bin").read_bytes()[:124]
        expected = (before / "source-after450.bin").read_bytes()
        assert expected[:31] + expected[39:] == actual[:31] + actual[39:]
        backing = bytearray((before / "backing.bin").read_bytes())
        backing[:0x355e] = (before / "source-prefix-after450.bin").read_bytes()
        backing[0x355e:0x3b55e] = (before / "source-map-after450.bin").read_bytes()
        backing[0x43d4e:0x48f4e] = (before / "source-plants-after450.bin").read_bytes()
        assert bytes(backing) == (after / "backing.bin").read_bytes()
        trace = json.loads((before / "source-trace450.json").read_text())
        report = trace["report"]
        assert struct.unpack("<HBI", (after / "rng.bin").read_bytes()) == (
            report["rng_seed_after"], report["rng_ring_after"], report["rng_draw_count_after"])
        cases.append({"seed": seed, "frame": 450, "compilation_receipt": pin(receipt),
                      "source_trace": pin(before / "source-trace450.json"),
                      "native_record": pin(after / "creatures.bin"),
                      "compared_bytes": len(backing) + 116 + 7,
                      "source_instructions": report["steps"],
                      "state_animation_target": [actual[0x5e], actual[0x19], actual[0x57]],
                      "position": list(struct.unpack_from("<hh", actual, 7))})
    result = {"status": "passed", "utc": datetime.now(timezone.utc).isoformat(), "cases": cases,
              "compared_bytes": sum(c["compared_bytes"] for c in cases),
              "original_execution": False,
              "scope": "Three corrected-trajectory frame450 Mule callbacks; native pointer gaps excluded, complete backing and RNG compared. Historical beforeimages do not prove preceding callbacks or full-game equivalence.",
              "validator": pin(Path(__file__))}
    destination = ROOT / "research/integration/landing-450-source-validation.json"
    destination.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": result["status"], "cases": cases, "compared_bytes": result["compared_bytes"]}))


if __name__ == "__main__":
    main()
