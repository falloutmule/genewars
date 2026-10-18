"""Atomic, repeatable writer for current pixel-validation receipts."""

from __future__ import annotations

import json
import os
from pathlib import Path
import uuid


def write_current_receipt(path: Path, payload: dict) -> str:
    """Atomically replace a current receipt while leaving dated history intact."""
    path.parent.mkdir(parents=True, exist_ok=True)
    encoded = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    temporary = path.with_name(f".{path.name}.{uuid.uuid4().hex}.tmp")
    try:
        temporary.write_text(encoded, encoding="utf-8", newline="\n")
        os.replace(temporary, path)
    finally:
        if temporary.exists():
            temporary.unlink()
    return encoded
