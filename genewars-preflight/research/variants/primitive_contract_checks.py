"""Safe synthetic vectors from independently reviewed CD instructions.

Prints synthetic numeric contracts only; reads private mapped bytes solely to
check function digests. It does not run the original program.
"""
import hashlib
import json
from pathlib import Path


def signed_word(value):
    return (value + 32768) % 65536 - 32768


def distance(query, coordinate):
    return sum((signed_word(2 * (to - start)) >> 1)**2
               for start, to in zip(query, coordinate))


def record_case(name, query=(0, 0), coordinate=(0, 0), radius_byte=0,
                marker=1, flags=1, kind=3, high_flags=0):
    record = bytearray(67)
    record[6] = marker
    for offset, component in zip((7, 9), coordinate):
        record[offset:offset+2] = (component & 65535).to_bytes(2, 'little')
    record[0x2d] = kind
    record[0x2e] = radius_byte
    record[0x31] = flags
    record[0x32] = high_flags
    d2 = distance(query, coordinate)
    radius = 1536 + 768 * radius_byte
    threshold = radius * radius & 0xffffffff
    qualifies = bool(marker and flags & 1 and kind == 3 and d2 <= threshold)
    return dict(name=name, query=list(query), coordinate=list(coordinate), radius_byte=radius_byte,
                marker=marker, flags=flags, kind=kind, ignored_high_flags=high_flags,
                squared_distance=d2, wrapped_squared_radius=threshold, expected=qualifies)


def touched(x, y, columns, rows):
    result = []
    for row in range(rows):
        y_index = (signed_word(y + row * 256) >> 8) & 127
        x_index = (signed_word(x) >> 8) & 127
        result.extend((y_index * 128 + x_index + column) * 14 + 11
                      for column in range(columns))
    return result


def main():
    root = Path(__file__).resolve().parents[2]
    image = (root / 'private/ghidra/primitives/memory-.object1.bin').read_bytes()
    digests = []
    for address, size in [(0x432c0, 124), (0x8d580, 106), (0x8d5f0, 105)]:
        value = image[address - 0x10000:address - 0x10000 + size]
        digests.append(dict(address=hex(address), size=size, sha256=hashlib.sha256(value).hexdigest()))
    records = [
        record_case('coincident_default'),
        record_case('inclusive_radius_boundary', coordinate=(1536, 0)),
        record_case('one_beyond_radius_boundary', coordinate=(1537, 0)),
        record_case('negative_boundary', coordinate=(-1536, 0)),
        record_case('wrapped_coordinate', coordinate=(-32768, -32768)),
        record_case('inactive_marker', marker=0),
        record_case('high_word_flag_does_not_qualify', flags=0, high_flags=1),
        record_case('other_flag_bits_ignored', flags=0xfe),
        record_case('kind_mismatch', kind=2),
        record_case('high_unsigned_threshold', coordinate=(16384, 16384), radius_byte=83),
        record_case('first_square_wrap_false', coordinate=(16384, 16384), radius_byte=84),
        record_case('exact_zero_square_wrap_true', radius_byte=254),
        record_case('exact_zero_square_wrap_false', coordinate=(1, 0), radius_byte=254),
        record_case('zero_square_wrap_modulo_equivalent', coordinate=(32767, 0), query=(-1, 0), radius_byte=254),
    ]
    grid_cases = []
    for name, args in [
        ('origin', (0, 0, 1, 1)),
        ('empty_columns', (0, 0, 0, 255)),
        ('empty_rows', (0, 0, 255, 0)),
        ('negative_coords', (-1, -1, 1, 1)),
        ('column_spills_into_next_linear_row', (32512, 0, 2, 1)),
        ('y_wraps_to_zero', (0, 32512, 1, 2)),
        ('last_cell_column_out_of_standard_grid', (-1, -1, 2, 1)),
        ('last_cell_three_row_spill', (-1, -1, 255, 1)),
    ]:
        offsets = touched(*args)
        grid_cases.append(dict(name=name, x=args[0], y=args[1], columns=args[2], rows=args[3],
                               touched_byte_offsets=offsets if len(offsets) <= 16 else None,
                               writes=len(offsets), first_offset=offsets[0] if offsets else None,
                               last_offset=offsets[-1] if offsets else None,
                               minimum_slice_len=max(offsets)+1 if offsets else 0,
                               fits_128_square_grid=all(offset < 128*128*14 for offset in offsets)))
    # Projection equivalence across the entire signed-word domain.
    projection_mismatches = sum(((signed_word(value) >> 8) & 127) != ((value >> 8) & 127)
                                for value in range(65536))
    print(json.dumps(dict(binary_sha256='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
                          function_digests=digests, record_cases=records,
                          grid_cases=grid_cases, projection_cases=65536,
                          projection_mismatches=projection_mismatches,
                          group_bytes=64*67, original_last_read_exclusive=63*67+0x33,
                          object3_fully_addressable_groups=(0x100000+695488-0x1645ae)//(64*67)), indent=2))


if __name__ == '__main__':
    main()
