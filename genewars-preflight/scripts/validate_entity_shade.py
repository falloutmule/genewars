"""Static instruction pins and bounded arithmetic oracle for CD 0x7B240."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import random

ROOT = Path(__file__).resolve().parents[1]
listing = ROOT / 'private/ghidra/camera-player-input/instructions-0007b240.txt'
text = listing.read_text()
for line in ['0007b28c MOV BL,byte ptr [ESI + EAX*0x1 + 0xc]',
             '0007b290 MOVSX EAX,word ptr [ESI + EAX*0x1]',
             '0007b294 IMUL EBX,EAX', '0007b2a1 SAR EBX,0x6',
             '0007b2e4 CMP ECX,EDX', '0007b2e6 JL 0x0007b325',
             '0007b31e SAR EAX,0x7', '0007b353 SAR EAX,0x7',
             '0007b358 SUB EAX,0x1e00', '0007b35d SAR EAX,0x8',
             '0007b360 LEA EDX,[EAX + 0x10]', '0007b363 CMP EDX,0x40',
             '0007b368 MOV EDX,0x40', '0007b373 XOR EAX,EDX']:
    assert line in text, line

rng = random.Random(0x7B240)
checks = 0
for i in range(200000):
    lights = [rng.randrange(-32768, 32768) for _ in range(4)]
    factors = [rng.randrange(256) for _ in range(4)]
    a, b, c, d = [light * factor // 64 for light, factor in zip(lights, factors)]
    fx, fy = rng.randrange(256), rng.randrange(256)
    if fx < fy:
        delta = (d - c) * fx + (c - a) * fy
        weighted = (256 - fy) * a + (fy - fx) * c + fx * d
    else:
        delta = (b - a) * fx + (d - b) * fy
        weighted = (256 - fx) * a + (fx - fy) * b + fy * d
    assert -2**31 <= delta < 2**31
    assert 2 * a + (delta >> 7) == weighted // 128
    final = max(0, min(64, ((2 * a + (delta >> 7) - 7680) >> 8) + 16))
    assert 0 <= final <= 64
    checks += 1

record = dict(status='passed-static-entity-shade-contract', checked_utc=datetime.now(timezone.utc).isoformat(),
              candidate_sha256='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
              function_va='0x7B240', listing=listing.relative_to(ROOT).as_posix(),
              listing_sha256=hashlib.sha256(listing.read_bytes()).hexdigest(),
              barycentric_equivalence_checks=checks,
              vertex='arithmetic_floor(signed_cell_0 * unsigned_cell_12 / 64) at each wrapped map vertex',
              interpolation='Same two triangular regions as height; doubled value with arithmetic shift by7',
              result='clamp(((doubled-7680)>>8)+16,0,64); endpoint64 is intentional and not a FADE row',
              limits=['Pure explicit-coordinate helper; no entity mutation, virtual predicate, map update, or live-state initialization',
                      'Static mathematical contract only; native CLI real-coordinate comparisons are recorded separately'])
(ROOT / 'research/integration/entity-shade-contract.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record))
