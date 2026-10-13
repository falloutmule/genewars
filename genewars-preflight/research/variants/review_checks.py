"""Independent read-only checks; only safe numeric/hash metadata is printed.

Run from the project root: python -B research/variants/review_checks.py
No reference code or original game byte sequences are embedded here.
"""
import hashlib
import json
import math
import random
import struct
from pathlib import Path


def signed16(value):
    value &= 65535
    return value - 65536 if value >= 32768 else value


def main():
    root = Path(__file__).resolve().parents[2]
    primitive_mismatches = []
    for difference in range(65536):
        # Direct effect of low-word double/subtract, SAR r16,1, MOVSX.
        instruction_delta = signed16(2 * difference) >> 1
        rust_formula_delta = (difference + 16384) % 32768 - 16384
        if instruction_delta != rust_formula_delta:
            primitive_mismatches.append(difference)

    # Valid prefix code: symbol 1 has depth 1/code 0; symbols 0 and 2
    # have depth 2/codes 01 and 11 in LSB-first representation.
    # Only one committed bit remains before the next literal run.
    codes = [(0, 2, 1), (1, 1, 0), (2, 2, 3)]
    outcomes = []
    for mode, entries in [('symbol_index', codes), ('depth', sorted(codes, key=lambda c: c[1]))]:
        available, buffer, physical_cursor = 1, 0, 0
        for symbol, width, code in entries:
            if available < width:
                # Simulate refill touching a literal word whose low bit is 0.
                buffer |= 0x5a00 << available
                available += 16
                physical_cursor += 2
            if buffer & ((1 << width) - 1) == code:
                outcomes.append({'probe_order': mode, 'matched_symbol': symbol,
                                 'physical_cursor_advance': physical_cursor})
                break

    candidate = root / 'private/ghidra/GW-CD-candidate.exe'
    data = candidate.read_bytes()
    git_blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    helpers = json.loads((root / 'research/ghidra/helpers-tree.json').read_text())
    matches = [{'path': row['path'], 'git_blob_sha1': row['sha'], 'size': row.get('size')}
               for row in helpers['tree'] if row.get('sha') == git_blob]
    variants = json.loads((root / 'research/variant-comparison.json').read_text())
    executable_key = next(key for key, value in variants.items()
                          if isinstance(value, list) and value and isinstance(value[0], dict)
                          and 'variant' in value[0] and 'executable' in value[0])
    records = []
    for row in variants[executable_key]:
        exe = row['executable']
        records.append({'variant': row['variant'], 'sha256': row['sha256'], 'size': row['size'],
                        'entry_object': exe['entry_object'], 'entry_offset': exe['entry_offset'],
                        'objects': [{'base': obj['relocation_base'], 'virtual_size': obj['virtual_size']}
                                    for obj in exe['objects']]})
    image = (root / 'private/ghidra/primitives/memory-.object1.bin').read_bytes()
    seeds = struct.unpack_from('<32H', image, 0x97952 - 0x10000)
    seed_failures = [bit for bit, seed in enumerate(seeds)
                     if seed < math.isqrt((1 << (bit + 1)) - 1)]
    def static_sqrt(value):
        if value == 0:
            return 0
        estimate = seeds[value.bit_length() - 1]
        for _ in range(100):
            quotient = value // estimate
            # Actual JGE is signed; check that signedness never affects inputs.
            assert quotient < 1 << 31 and estimate < 1 << 31
            if quotient >= estimate:
                return estimate
            estimate = (estimate + quotient) >> 1
        raise AssertionError('Static algorithm did not terminate')
    sqrt_inputs = {0, 2**32 - 1}
    for n in range(1, 65536):
        sqrt_inputs.update((n*n - 1, n*n, n*n + 1))
    rng = random.Random(0x97910)
    sqrt_inputs.update(rng.randrange(2**32) for _ in range(10000))
    sqrt_mismatches = sum(static_sqrt(value) != math.isqrt(value) for value in sqrt_inputs)
    print(json.dumps({'primitive': {'differences_tested': 65536, 'mismatches': primitive_mismatches,
                                  'max_squared_sum': 2 * 16384**2},
                      'sqrt': {'static_model_cases': len(sqrt_inputs), 'mismatches': sqrt_mismatches,
                               'bsr_bins': 32, 'seed_bound_failure_bins': seed_failures,
                               'table_values_exported': False},
                      'synthetic_speculative_refill': outcomes,
                      'candidate': {'size': len(data), 'sha256': hashlib.sha256(data).hexdigest(),
                                    'git_blob_sha1': git_blob, 'upstream_metadata_matches': matches},
                      'variant_metadata_from_preflight': records}, indent=2))


if __name__ == '__main__':
    main()
