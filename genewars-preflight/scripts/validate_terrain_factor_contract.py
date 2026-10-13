"""Bounded radius-4096 factor arithmetic, with callback side effects excluded."""
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PINS = {
    'instructions-00075b00.txt': '338546946ce863304620159905fd171845a8d655c4b09f0f6657b6a58fedc966',
    'instructions-00072ad0.txt': 'b3b313e20205eb57066681414db877493563bd8990149334125f06210936386b',
    'instructions-00084b40.txt': '9424f5091c9fdb3baa1f4e27c53310c85cdbf87a51ce4fa0aa11b54dadb363a5',
}


def local_factor(distance_squared, previous):
    if previous >= 63:
        return previous
    if distance_squared < 3840**2:
        return 63
    if distance_squared >= 4096**2:
        return previous
    candidate = ((4096 - math.isqrt(distance_squared)) * 64 - 64) // 256
    return max(previous, candidate)


def main():
    contracts = []
    text = {}
    for name, digest in PINS.items():
        path = ROOT / 'private/ghidra/camera-player-input' / name
        data = path.read_bytes()
        assert hashlib.sha256(data).hexdigest() == digest, name
        text[name] = data.decode()
        contracts.append(dict(private_path=path.relative_to(ROOT).as_posix(), sha256=digest))
    body = text['instructions-00075b00.txt']
    for line in ['00075b0c LEA EAX,[EDX + 0xffffff00]', '00075b93 INC AH', '00075b95 INC DL',
                 '00075c36 MOV AL,byte ptr [EBX + 0xc]', '00075c39 CMP EAX,0x3f',
                 '00075c8e SAR DX,0x1', '00075ca5 SAR AX,0x1', '00075cac SAR EDI,0x3',
                 '00075cbc JGE 0x00075cd4', '00075cbe MOV byte ptr [EBX + 0xc],0x3f',
                 '00075cd8 JGE 0x00075bff', '00075ce0 CALL 0x00097910',
                 '00075ce9 SHL EAX,0x6', '00075cec LEA EDX,[EAX + -0x40]',
                 '00075cf7 SBB EAX,EDX', '00075cf9 SAR EAX,0x8', '00075cff JBE 0x00075bff',
                 '00075d05 MOV byte ptr [EBX + 0xc],AL', '00075d10 CALL 0x00084b40']:
        assert line in body, line
    caller = text['instructions-00072ad0.txt']
    for line in ['00072bc4 MOV EDX,0x1000', '00072bc9 ADD EAX,0x9', '00072bcc CALL 0x00075b00']:
        assert line in caller, line
    # Every input high/low-byte combination gives exactly 33 scanned vertices.
    for center in range(65536):
        start = ((center - 4096) & 65535) >> 8
        end = ((((center + 4096) & 65535) >> 8) + 1) & 255
        assert (end - start) % 256 == 33
    cases = sorted({max(0, n*n + delta) for n in range(4097) for delta in [-1, 0, 1]})
    for squared in cases:
        for previous in [0, 1, 31, 62, 63, 64, 255]:
            result = local_factor(squared, previous)
            assert result >= previous
            if previous >= 63:
                assert result == previous
            else:
                assert result <= 63
                if 3840**2 <= squared < 4096**2:
                    assert result == max(previous, (4095 - math.isqrt(squared)) // 4)
    receipt = dict(status='passed-static-factor-plan-contract', checked_utc=datetime.now(timezone.utc).isoformat(),
        candidate_sha256='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
        instruction_contracts=contracts, function_va='0x75B00', admitted_radius=4096,
        radius_source='0x72BC4 loads EDX=0x1000 before0x75B00; center is explicit current selected-player+9',
        scan='33x33 row-major vertices; unsigned byte high coordinates wrap256; source-map cell indices wrap128',
        scan_coordinate_cases=65536, distance_boundary_cases=len(cases), previous_factor_values=[0,1,31,62,63,64,255],
        distance='wrapped15dx^2 + wrapped15dy^2 + ((height2_vertex-height2_center)>>3)^2',
        factor='skip old>=63; d2<3840^2 gives63; else d2<4096^2 gives max(old,floor((4095-isqrt(d2))/4)); else unchanged',
        maximum_squared_distance=3*16384**2, ordered_changes_only=True, map_mutated=False,
        callbacks_executed=False, original_execution=False,
        limits=['Pure local arithmetic plan on an immutable map snapshot, not whole0x75B00 equivalence',
                '0x84B40 modifies dirty flags/viewport/minimap and refreshes entity shades via virtual predicates; those side effects are excluded',
                'Factor gameplay meaning and complete visibility lifecycle remain separate',
                'No initial selected-player target or mission state inferred; no other radius admitted'])
    (ROOT / 'research/integration/terrain-factor-contract.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({'status': receipt['status'], 'coordinate_cases':65536, 'distance_boundary_cases':len(cases)}))


if __name__ == '__main__':
    main()
