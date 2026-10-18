"""Pin the explicit-target camera calculation; do not infer startup state."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = {
    'private/ghidra/terrain-camera-refs/instructions-00082e20.txt':
        '66286b34e27300bf69ee0b62c036059c9e981596ae85703fd021a940983c7eb3',
    'private/ghidra/terrain-camera-refs/instructions-0007d090.txt':
        'b9a4d6c79119564ba2ec9bf4cc120ce206530663808300faeab3c3c2993d98ac',
}


def signed32(value):
    value &= 0xffffffff
    return value - 0x100000000 if value & 0x80000000 else value


def instruction_correction(height):
    eax = height >> 6
    edx = (eax >> 31) & 0xffffffff
    carry = (edx >> 27) & 1  # Last bit shifted out by SHL EDX,5.
    edx = (edx << 5) & 0xffffffff
    eax = signed32(eax - edx - carry)
    return eax >> 5


def main():
    listings = {}
    for relative, expected in CONTRACTS.items():
        data = (ROOT / relative).read_bytes()
        assert hashlib.sha256(data).hexdigest() == expected, relative
        listings[relative] = data.decode()
    anchor = listings[next(iter(CONTRACTS))]
    for item in ['00082e2d CALL 0x0007b140', '00082e32 SAR EAX,0x6',
                 '00082e3a SHL EDX,0x5', '00082e3d SBB EAX,EDX',
                 '00082e3f SAR EAX,0x5', '00082e4b ADD AH,0xc',
                 '00082e5a ADD AL,0x2', '00082e62 XOR AH,AH',
                 '00082e66 MOV word ptr [ESI + 0x2],AX', '00082e6a MOV word ptr [ESI],DX']:
        assert item in anchor, item
    caller = listings['private/ghidra/terrain-camera-refs/instructions-0007d090.txt']
    for item in ['0007d594 MOV AL,[0x0011873d]', '0007d599 CMP EAX,EBX',
                 '0007d5a2 MOV EDX,dword ptr [ECX + 0x9]', '0007d5a5 CALL 0x00082e20',
                 '0007d6c1 IMUL ECX,EDX,0x70d', '0007d6d2 ADD ECX,0x118d74']:
        assert item in caller, item
    comparisons = 0
    for height in range(-65536, 65535):
        shifted = height // 64
        truncated = (abs(shifted) // 32) * (-1 if shifted < 0 else 1)
        assert instruction_correction(height) == truncated
        comparisons += 1
    wraps = 0
    for high in range(256):
        for correction in range(-32, 32):
            # Model AH/AL addition and BL/DL subtraction as individual bytes.
            x = (high - ((correction + 12) & 255)) & 255
            y = (high - ((correction + 2) & 255)) & 255
            assert x == (high - correction - 12) % 256
            assert y == (high - correction - 2) % 256
            wraps += 1
    vectors = []
    for x, y, height in [(0, 0, 0), (-256, -32768, 0), (0, 0, -2),
                          (0, 0, -65536), (0, 0, 65534), (32767, -1, 2050)]:
        q = instruction_correction(height)
        vectors.append(dict(target=[x, y], doubled_height=height, correction=q,
                            camera_cells=[((x >> 8) - q - 12) & 255, ((y >> 8) - q - 2) & 255]))
    receipt = dict(status='passed-static-explicit-target-contract', checked_utc=datetime.now(timezone.utc).isoformat(),
        candidate_sha256='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
        instruction_contracts=[dict(private_path=p, sha256=h) for p, h in CONTRACTS.items()],
        function_va='0x82E20', caller_va='0x7D5A5', target_source='selected player record +9, stride0x70D; current live state',
        height_quantization_values=comparisons, byte_wrap_pairs=wraps, vectors=vectors,
        formula='q=trunc_toward_zero((doubled_height>>6)/32); cells=((x_high-q-12)&255,(y_high-q-2)&255), zero-extended words',
        original_execution=False,
        limits=['Explicit target only; not an inferred initial mission camera',
                'Loader restoration and serialized tail prevent assuming raw file player+9 is live target',
                'Viewport/minimap/sound/redraw side effects of82E20 are excluded'])
    target = ROOT / 'research/integration/camera-target-contract.json'
    target.write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({'status': receipt['status'], 'height_quantization_values': comparisons, 'byte_wrap_pairs': wraps}))


if __name__ == '__main__':
    main()
