"""Stage the exact mapped construction border table from the validated EXE image."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "private/ghidra/validated/memory-.object3.bin"
SOURCE_SHA256 = "aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87"
TABLE_OFFSET = 0x112D24 - 0x100000
TABLE_SIZE = 8192
TABLE_SHA256 = "9f1dcbc35c350d6027f98be0f5c8b43b42ca52b7604459c0c42be3aa88913d47"
OUTPUT = ROOT / "private/integration/physics-tables/border.bin"
RECEIPT = ROOT / "research/integration/construction-border-inputs.json"


def stage() -> dict:
    source_bytes = SOURCE.read_bytes()
    source_hash = hashlib.sha256(source_bytes).hexdigest()
    if source_hash != SOURCE_SHA256:
        raise ValueError("validated object3 image changed")
    table = source_bytes[TABLE_OFFSET:TABLE_OFFSET + TABLE_SIZE]
    if len(table) != TABLE_SIZE or hashlib.sha256(table).hexdigest() != TABLE_SHA256:
        raise ValueError("mapped 0x112D24 border table differs from its validator pin")
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    if OUTPUT.exists():
        if OUTPUT.read_bytes() != table:
            raise ValueError("refusing to overwrite changed construction border input")
    else:
        with OUTPUT.open("xb") as stream:
            stream.write(table)
    result = {
        "source": "private/ghidra/validated/memory-.object3.bin",
        "source_sha256": SOURCE_SHA256,
        "source_va_base": "0x100000",
        "table_va": "0x112D24",
        "table_offset_in_object3": TABLE_OFFSET,
        "table_size": TABLE_SIZE,
        "table_sha256": TABLE_SHA256,
        "destination": "private/integration/physics-tables/border.bin",
        "validator": "scripts/ghidra/validate_construction_border.py",
        "original_execution": False,
    }
    RECEIPT.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return result


if __name__ == "__main__":
    print(json.dumps(stage(), sort_keys=True))
