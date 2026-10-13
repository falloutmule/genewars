"""Pin static mode5 row preparation; compare original register-style row stepping.

No original machine code is executed. Texture sampling has its own receipt.
"""
from pathlib import Path
import hashlib
import json
import struct

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
sha = lambda b: hashlib.sha256(b).hexdigest()
candidate = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G / 'GW-CD-candidate.exe').read_bytes()) == candidate
code = (G / 'validated/memory-.object1.bin').read_bytes()
tables = {hex(a): hex(struct.unpack_from('<I', code, a - 0x10000 + 20)[0])
          for a in [0xe1a67, 0xe2374, 0xe2ca2, 0xe3121, 0xe3513]}
assert list(tables.values()) == ['0xe1ad3', '0xe23e0', '0xe2d0e', '0xe318d', '0xe3f93']
contracts = {
 'planets-texture/instructions-000e1900.txt': ['000e1901 SUB ESP,0x6c', '000e1913 CMP EAX,EBX', '000e1a2e JLE 0x000e35f2', '000e233b JLE 0x000e35f2', '000e2c80 IDIV EBX', '000e310d IDIV EBX'],
 'terrain-triangle/general-right-mode5.txt': ['000e1aeb SAR EAX,0x10', '000e1afc INC EBX', '000e1b07 SHRD EAX,EDX,0x10', '000e1b12 IDIV EBX', '000e1b55 IDIV dword ptr [ESP + 0x10]', '000e1d0f MOV EBX,dword ptr [ESP + 0x1c]', '000e1d38 ADD ECX,dword ptr [ESP + 0x28]'],
 'terrain-triangle/general-left-mode5.txt': ['000e2407 JZ 0x000e2457', '000e2409 INC EBX', '000e2414 SHRD EAX,EDX,0x10', '000e2457 MOV dword ptr [ESP + 0x3c],EAX', '000e248f IDIV dword ptr [ESP + 0x18]', '000e2649 MOV EAX,dword ptr [ESP + 0x1c]', '000e2672 ADD ECX,dword ptr [ESP + 0x2c]'],
 'terrain-triangle/flat-bottom-mode5.txt': ['000e2d10 SUB EBX,dword ptr [ECX]', '000e2d19 IDIV EBX', '000e2d40 IDIV dword ptr [ESP + 0x20]', '000e2dfe MOV dword ptr [EDI],EAX', '000e2e27 JNZ 0x000e2dfe'],
 'terrain-triangle/flat-top-mode5.txt': ['000e318f SUB EBX,dword ptr [ESI]', '000e3198 IDIV EBX', '000e31bf IDIV dword ptr [ESP + 0x20]', '000e3280 MOV dword ptr [EDI],EAX', '000e32a9 JNZ 0x000e3280'],
 'triangle-buffer-refs/instructions-000e88a3.txt': ['000e88c9 MOV [0x00100104],EAX', '000e88ce SUB EAX,dword ptr [0x00100110]', '000e88d4 MOV [0x00100100],EAX'],
 'terrain-mode5.txt': ['000e3fe4 ADD EDI,dword ptr [0x00100110]', '000e3fdb SAR EAX,0x10', '000e3fde SAR EBP,0x10'],
}
listings = []
for name, expected in contracts.items():
    raw = (G / name).read_bytes()
    for line in expected:
        assert line in raw.decode(), (name, line)
    listings.append({'private_path': 'private/ghidra/' + name, 'sha256': sha(raw)})

def i32(n):
    return (n + 0x80000000) % 0x100000000 - 0x80000000

def div(n, d):
    assert d > 0
    return i32(abs(n) // d * (-1 if n < 0 else 1))

def sub(a, b):
    return i32(a - b)

def slope(a, b, height):
    return div(i32(sub(b, a) << 16), height)

def classify(v):
    """Cyclic input order is retained; reversed winding is rejected."""
    if len({p[1] for p in v}) == 1:
        return None
    for k in range(3):
        t, p, q = v[k:] + v[:k]
        if t[1] == p[1] < q[1]:
            return ('flat_top', t, p, q) if p[0] > t[0] else None
        if t[1] < p[1] == q[1]:
            return ('flat_bottom', t, q, p) if p[0] > q[0] else None
        if t[1] < p[1] < q[1]:
            return 'right_middle', t, p, q
        if t[1] < q[1] < p[1]:
            return 'left_middle', t, q, p
    raise AssertionError(v)

def prepare(v, clip_height):
    """Rows are [y,left_fixed,right_fixed,U,V,S]; deltas [dU,dV,dS]."""
    if clip_height not in range(1, 481):
        raise ValueError('safe clip-height admission is1..480')
    if any(len(p) != 5 or any(n != i32(n) for n in p) or
           any(n not in range(-4096, 4097) for n in p[:2]) for p in v):
        raise ValueError('safe integer coordinates are-4096..4096 and attributes i32')
    c = classify(v)
    if c is None:
        return None
    kind, t, m, b = c
    top, bottom = t[1], b[1]
    if bottom <= 0 or top >= clip_height:
        return None
    h = bottom - top
    ta, ma, ba = t[2:], m[2:], b[2:]
    if kind == 'flat_top':
        # t=left top, m=right top, b=bottom.
        left0, right0 = t[0] << 16, m[0] << 16
        sl, sr = slope(t[0], b[0], h), slope(m[0], b[0], h)
        gradients = [div(sub(r, l), m[0] - t[0]) for l, r in zip(ta, ma)]
        da = [div(sub(r, l), h) for l, r in zip(ta, ba)]
        middle_h, lower_left, lower_right, lower_da = h, sl, sr, da
    elif kind == 'flat_bottom':
        # t=top, m=left bottom, b=right bottom.
        left0 = right0 = t[0] << 16
        sl, sr = slope(t[0], m[0], h), slope(t[0], b[0], h)
        gradients = [div(sub(r, l), b[0] - m[0]) for l, r in zip(ma, ba)]
        da = [div(sub(r, l), h) for l, r in zip(ta, ma)]
        middle_h, lower_left, lower_right, lower_da = h, sl, sr, da
    else:
        middle_h = m[1] - top
        long_slope = slope(t[0], b[0], h)
        upper_slope = slope(t[0], m[0], middle_h)
        lower_slope = slope(m[0], b[0], h - middle_h)
        ratio = div(middle_h << 16, h)
        if kind == 'right_middle':
            if upper_slope <= long_slope:
                return None
            width = i32(sub(m[0], t[0]) + (i32(sub(t[0], b[0]) * ratio) >> 16))
            numerators = [i32(i32((sub(l, r) * ratio) >> 16) + mid - l)
                          for l, mid, r in zip(ta, ma, ba)]
            sl, sr = long_slope, upper_slope
            lower_left, lower_right = sl, lower_slope
            da = [div(sub(r, l), h) for l, r in zip(ta, ba)]
            lower_da = da
        else:
            if long_slope <= upper_slope:
                return None
            width = i32(sub(t[0], m[0]) + (i32(sub(b[0], t[0]) * ratio) >> 16))
            numerators = [i32(i32((sub(r, l) * ratio) >> 16) + l - mid)
                          for l, mid, r in zip(ta, ma, ba)]
            sl, sr = upper_slope, long_slope
            lower_left, lower_right = lower_slope, sr
            da = [div(sub(r, l), middle_h) for l, r in zip(ta, ma)]
            lower_da = [div(sub(r, l), h - middle_h) for l, r in zip(ma, ba)]
        if width < 0:
            return None
        if width == 0:
            raise ValueError('general zero-width gradient setup reads uninitialized stack fields')
        gradients = [div(n, width + 1) for n in numerators]
        left0 = right0 = t[0] << 16
    rows = []
    for y in range(max(top, 0), min(bottom, clip_height)):
        k = y - top
        a = min(k, middle_h)
        z = max(k - middle_h, 0)
        left = i32(left0 + sl * a + lower_left * z)
        right = i32(right0 + sr * a + lower_right * z)
        if z and kind == 'right_middle':
            right = i32((m[0] << 16) + lower_right * z)
        if z and kind == 'left_middle':
            left = i32((m[0] << 16) + lower_left * z)
        # Reset the short edge at the exact middle row as well.
        if k == middle_h and kind == 'right_middle':
            right = m[0] << 16
        if k == middle_h and kind == 'left_middle':
            left = m[0] << 16
        attrs = [i32(value + upper * a + lower * z)
                 for value, upper, lower in zip(ta, da, lower_da)]
        rows.append([y, left, right, *attrs])
    return {'kind': kind, 'gradients': gradients, 'rows': rows,
            'canonical': [t, m, b], 'steps': [sl, sr, lower_left, lower_right, *da, *lower_da],
            'middle_height': middle_h}

def stepping_rows(result, clip_height):
    """Independent register-style repeated ADD and middle-edge reset."""
    t, m, b = result['canonical']
    kind = result['kind']
    sl, sr, ll, lr, *attr_steps = result['steps']
    da, lower_da = attr_steps[:3], attr_steps[3:]
    left = t[0] << 16
    right = (m[0] if kind == 'flat_top' else t[0]) << 16
    attrs = list(t[2:])
    rows = []
    for k, y in enumerate(range(t[1], b[1])):
        if k == result['middle_height']:
            sl, sr, da = ll, lr, lower_da
            if kind == 'right_middle':
                right = m[0] << 16
            if kind == 'left_middle':
                left = m[0] << 16
        if 0 <= y < clip_height:
            rows.append([y, left, right, *attrs])
        left, right = i32(left + sl), i32(right + sr)
        attrs = [i32(a + d) for a, d in zip(attrs, da)]
    return rows

fixtures = [
 ('flat_top', [[2, 1, 0, 0, 0x208000], [10, 1, 0x1fffff, 0, 0x208000], [6, 7, 0x100000, 0x1fffff, 0x208000]], 10),
 ('flat_bottom', [[6, 1, 0x100000, 0, 0x208000], [10, 7, 0x1fffff, 0x1fffff, 0x208000], [2, 7, 0, 0x1fffff, 0x208000]], 10),
 ('right_middle', [[3, 1, 0, 0, 0x208000], [12, 4, 0x1fffff, 0x100000, 0x218000], [5, 9, 0, 0x1fffff, 0x208000]], 10),
 ('left_middle', [[7, 1, 0x100000, 0, 0x208000], [11, 9, 0x1fffff, 0x1fffff, 0x208000], [1, 4, 0, 0x100000, 0x218000]], 10),
 ('top_clip_above_middle', [[3, -5, 0, 0, 0x208000], [12, 3, 0x1fffff, 0x100000, 0x218000], [5, 9, 0, 0x1fffff, 0x208000]], 6),
 ('top_clip_below_middle', [[7, -7, 0x100000, 0, 0x208000], [11, 9, 0x1fffff, 0x1fffff, 0x208000], [1, -3, 0, 0x100000, 0x218000]], 6),
 ('negative_rounding', [[0, -1, 1, -1, -1], [7, 2, -0x1fffff, 0x1fffff, -0x20001], [1, 8, -0x100001, -0x100001, 0x3fffff]], 5),
 ('wrapped_attributes', [[1, 0, 0x7fffffff, -0x80000000, 0x7fffffff], [9, 3, -0x80000000, 0x7fffffff, -1], [2, 8, -1, 1, -0x80000000]], 8),
]
vectors = []
checks = 0
for name, v, height in fixtures:
    r = prepare(v, height)
    assert r and r['rows'] == stepping_rows(r, height)
    vectors.append({'name': name, 'vertices_x_y_u_v_s': v, 'clip_height': height,
                    'kind': r['kind'], 'gradients_du_dv_ds': r['gradients'], 'rows_y_left_right_u_v_s': r['rows']})
    for shift in range(-10, 11):
        for clip in [1, 3, 6, 10, 480]:
            shifted = [[p[0], p[1] + shift, *p[2:]] for p in v]
            for rotation in range(3):
                rotated = shifted[rotation:] + shifted[:rotation]
                r = prepare(rotated, clip)
                if r:
                    assert r['rows'] == stepping_rows(r, clip)
                checks += 1
for name, v, h in fixtures:
    assert prepare([v[0], v[2], v[1]], h) is None
    checks += 1
zero_width = [[0, 0, 0, 0, 0], [1, 1, 1, 1, 1], [1, 3, 2, 2, 2]]
try:
    prepare(zero_width, 4)
except ValueError:
    pass
else:
    raise AssertionError('zero-width general gradient must reject')

receipt = {'candidate_sha256': candidate, 'instruction_contracts': listings, 'mode5_dispatch_tables': tables,
 'scope': 'Explicit integer vertex to prepared mode5 rows; sampler has separate mode5-span receipt; no camera or startup state chosen.',
 'point_layout': '20bytes: signed32 x,y,U,V,S; x/y integer, attributes16.16 bitpatterns',
 'row_layout': 'Each row [y,left16.16,right16.16,U,V,S], attributes anchored to left edge without fractional-x compensation.',
 'vertical_coverage': '[max(top_y,0),min(bottom_y,clip_height)); top inclusive/bottom exclusive. E88A3 stores framebuffer minus pitch before sampler pre-increment.',
 'integer_semantics': 'Wrapped32 subtraction/addition; signed division truncates toward0; x slopes shift16 before division; general x width uses arithmetic shift of low32 product, attribute numerator uses low32 of signed64 product shifted16.',
 'middle_edge': 'Reset short-edge x to exact middleX<<16; retain accumulated attributes. Left-middle changes attribute vertical gradient at middle without resetting attributes to middle values.',
 'admission': {'coordinate_min': -4096, 'coordinate_max': 4096, 'clip_height': [1, 480], 'attribute': 'all signed32 bitpatterns', 'zero_general_width': 'reject as uninitialized-gradient source case', 'reason': 'Explicit safe implementation limits; not original runtime validation'},
 'vectors': vectors, 'row_stepping_checks': checks,
 'limits': ['Positive clip-width and source/FADE address validation remain sampler preconditions.', 'Do not replace the rounded general-gradient width+1 with determinant/barycentric division.', 'This receipt does not choose camera, visibility, initial factor values, draw order, or gameplay state.']}
(P / 'research/ghidra/mode5-triangle-evidence.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(f'PASS {len(listings)} instruction contracts; 8 complete synthetic row vectors; {checks} cyclic/clipped/winding row-stepping checks; zero-width general rejection')
