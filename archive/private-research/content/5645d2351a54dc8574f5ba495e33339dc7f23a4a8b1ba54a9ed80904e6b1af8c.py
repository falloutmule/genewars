"""Validate explicit-input 7B7F0 strip planning, never execute original code."""
from pathlib import Path
import hashlib
import json
import struct

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
sha = lambda b: hashlib.sha256(b).hexdigest()
candidate = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G / 'GW-CD-candidate.exe').read_bytes()) == candidate
contracts = {
 'terrain-camera-refs/instructions-0007b7f0.txt': ['0007b800 MOV dword ptr [ESP + 0x7c],ECX', '0007b841 SAR EAX,0x1', '0007b860 TEST DL,0x1', '0007b8ad SUB EDX,EBX', '0007ba8a AND byte ptr [EAX + 0xb],0xfb', '0007baa2 JL 0x0007bc24', '0007bb54 CMP EBX,0x16e098', '0007bb73 MOV byte ptr [0x00102e25],BH', '0007bb8e CALL 0x000e1900', '0007bb9f JLE 0x0007bbdf', '0007bbcb MOV AL,0x6', '0007bc1f CALL 0x000e1900', '0007bcb0 AND byte ptr [EAX + 0xb],0xf7', '0007bcc2 JL 0x0007be39', '0007bd9d CALL 0x000e1900', '0007be3f MOV ESI,EBP', '0007be4a MOV dword ptr [ESP + 0x94],ECX', '0007be55 JG 0x0007b9b8', '0007be64 JG 0x0007b9b8'],
 'terrain-strip-refs/instructions-00083b20.txt': ['00083b55 MOVSX EBX,word ptr [ESI + 0x33]', '00083b69 SAR EAX,0x5', '00083b72 ADD EAX,EDX', '00083bce MOV BL,byte ptr [ESP + 0x4]', '00083bd4 JL 0x00083bb7', '00083c99 CMP EDX,0x14'],
}
listings = []
for name, expected in contracts.items():
    raw = (G / name).read_bytes()
    for line in expected:
        assert line in raw.decode(), (name, line)
    listings.append({'private_path': 'private/ghidra/' + name, 'sha256': sha(raw)})

def sample(region, world, camera, top):
    x, y = world
    i = ((y & 127) * 128 + (x & 127)) * 14
    light, height = struct.unpack_from('<hh', region, i)
    tex, flags = region[i + 4:i + 6]
    factor, overlay = region[i + 12:i + 14]
    dx, dy = x - camera[0], y - camera[1]
    return {'world': [x, y], 'map_index': i // 14,
            'position': [(dx - dy) * 32, (dx + dy) * 16 - 2 * (height >> 6) - top],
            'shade': 4 * light * factor + 0x8000, 'texture': tex,
            'orientation': (flags & 0x70) >> 4, 'flags11': region[i + 11], 'overlay13': overlay}

def pair_calls(a, b, c, d, context_flags):
    calls = []
    patches = []
    if context_flags & 4:
        patches.extend([[a['map_index'], 0xfb], [b['map_index'], 0xf7]])
    for half, owner, other, points, uv in [
            (0, a, b, [a, c, b], [0, 2, 3]),
            (1, b, b, [b, c, d], [0, 1, 2])]:
        if all(p['position'][1] < 0 for p in points):
            continue
        if owner['texture'] >= 160:
            raise ValueError('emitted base texture must be0..159')
        base = {'half': half, 'mode': 0 if all(p['shade'] == 0 for p in points) else 5,
                'owner': owner['map_index'], 'texture': owner['texture'],
                'orientation': owner['orientation'], 'corner_indices': uv,
                'world': [p['world'] for p in points], 'positions': [p['position'] for p in points],
                'shades': [p['shade'] for p in points]}
        calls.append(base)
        overlay = owner['overlay13']
        if overlay > 1:
            calls.append(base | {'mode': 6, 'overlay_table_index': 2 if overlay >= 97 else (overlay - 1) // 32})
        if context_flags & 0x40 and owner['flags11'] & 1 and not other['flags11'] & 0x20:
            calls.append(base | {'mode': 15})
    return calls, patches

def strip(region, camera, column, top, bottom, context_flags):
    if len(region) != 128 * 128 * 14 or any(v not in range(128) for v in camera):
        raise ValueError('exact map region and explicit camera0..127 required')
    if column not in range(20) or not 0 <= top < bottom <= 480 or context_flags not in range(256):
        raise ValueError('safe strip0..19, 0<=top<bottom<=480, and byteflags required')
    n, parity = column >> 1, column & 1
    x0, y0 = camera[0] + n, camera[1] - n - 1 - parity
    calls, patches = [], []
    for k in range(48):
        x, y = x0 + k, y0 + k
        a, b, c, d = [sample(region, p, camera, top) for p in
                      [(x, y), (x, y + 1), (x + 1, y + 1), (x + 1, y + 2)]]
        pair, changes = pair_calls(a, b, c, d, context_flags)
        calls.extend(pair)
        patches.extend(changes)
        if bottom - top <= c['position'][1] and bottom - top <= d['position'][1]:
            return {'pairs': k + 1, 'calls': calls, 'dirty_and_masks': patches}
    raise AssertionError('signed16 height bound guarantees stop within48pairs')

def cache_strip(region, camera, column, top, bottom, context_flags):
    """Literal alternating cache evolution and baseline increments, independent projection."""
    x, y = camera[0] + (column >> 1), camera[1] - (column >> 1) - 1 - (column & 1)
    baseline = -16 * (1 + (column & 1)) - top
    def cached(wx, wy, screen_x, base_y):
        p = sample(region, [wx, wy], camera, top)
        off = p['map_index'] * 14
        height = struct.unpack_from('<h', region, off + 2)[0]
        p['position'] = [screen_x, base_y - 2 * (height >> 6)]
        return p
    a = cached(x, y, 32 * (column + 1), baseline)
    baseline += 16
    b = cached(x, y + 1, 32 * column, baseline)
    calls, patches = [], []
    k = 0
    while True:
        x += 1
        y += 1
        baseline += 16
        c = cached(x, y, 32 * (column + 1), baseline)
        baseline += 16
        d = cached(x, y + 1, 32 * column, baseline)
        pair, changes = pair_calls(a, b, c, d, context_flags)
        calls.extend(pair)
        patches.extend(changes)
        k += 1
        a, b = c, d
        if not (bottom - top > b['position'][1] or bottom - top > a['position'][1]):
            return {'pairs': k, 'calls': calls, 'dirty_and_masks': patches}

# Safe synthetic fixtures include all dispatch modes and dirty-mask requests.
region = bytearray(128 * 128 * 14)
for i in range(16384):
    struct.pack_into('<hh', region, i * 14, (i % 9 - 4) * 512, (i % 17 - 8) * 64)
    region[i * 14 + 4] = i % 160
    region[i * 14 + 5] = (i % 8) << 4
    region[i * 14 + 11] = [0, 1, 0x21, 0x0c][i % 4]
    region[i * 14 + 12] = i % 64
    region[i * 14 + 13] = [1, 2, 33, 65, 97][i % 5]
vectors = []
checks = 0
for camera in [[0, 0], [17, 29], [127, 127]]:
    for column in range(20):
        for top, bottom in [[0, 480], [0, 128], [127, 128], [64, 320]]:
            for flags in [0, 4, 0x40, 0x44]:
                a = strip(region, camera, column, top, bottom, flags)
                assert a == cache_strip(region, camera, column, top, bottom, flags)
                checks += 1
                if camera == [0, 0] and column in [0, 1] and (top, bottom, flags) == (0, 128, 0x44):
                    vectors.append({'camera': camera, 'column': column, 'top': top, 'bottom': bottom,
                                    'context_flags': flags, 'pairs': a['pairs'], 'first_calls': a['calls'][:6],
                                    'first_dirty_and_masks': a['dirty_and_masks'][:4]})
level = (G / 'campaign-resources/152.dat').read_bytes()
assert sha(level) == '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
actual_region = level[0x355e:0x3b55e]
private = G / 'terrain-strip'; private.mkdir(exist_ok=True)
actual = []
for camera, column, top, bottom in [([0, 0], 0, 0, 480), ([17, 29], 4, 0, 480), ([127, 127], 19, 64, 320)]:
    output = strip(actual_region, camera, column, top, bottom, 0)
    assert output == cache_strip(actual_region, camera, column, top, bottom, 0)
    raw = (json.dumps(output, separators=(',', ':')) + '\n').encode()
    name = f'camera-{camera[0]}-{camera[1]}-column-{column}.json'
    (private / name).write_bytes(raw)
    actual.append({'camera': camera, 'column': column, 'top': top, 'bottom': bottom,
                   'context_flags': 0, 'pairs': output['pairs'], 'calls': len(output['calls']),
                   'mode_counts': {str(m): sum(c['mode'] == m for c in output['calls']) for m in [0, 5, 6, 15]},
                   'private_output': 'private/ghidra/terrain-strip/' + name, 'sha256': sha(raw)})
assert 0x11c2be + len(actual_region) == 0x1542be < 0x16e098
receipt = {'candidate_sha256': candidate, 'instruction_contracts': listings,
 'scope': 'One complete7B7F0 strip with explicit camera, column, vertical range and contextflags; draw-call planning and dirty AND requests, no original execution.',
 'inputs': {'camera': [0, 127], 'column': [0, 19], 'vertical': '0<=top<bottom<=480', 'context_flags': 'byte, original object+1C', 'map': 'exact229376B14-byte cells; caller-supplied state'},
 'start': 'n=column>>1,p=column&1,x0=cameraX+n,y0=cameraY-n-1-p; baselineA=-16*(1+p)-top',
 'pair': 'k>=0: A=(x0+k,y0+k),B=(x0+k,y0+k+1),C=(x0+k+1,y0+k+1),D=(x0+k+1,y0+k+2); C/D become nextA/B',
 'order': 'First A,C,B withownerA/corners0,2,3; thenB,C,D withownerB/corners0,1,2. Base thenoptional6 thenoptional15 for eachhalf.',
 'stop': 'After drawing pair stop if bothC.y andD.y>=bottom-top; otherwisecontinue. Signed16 height and bottom<=480 guarantee<=48pairs.',
 'cull': 'Individual triangle skipped if all3projectedY<0; equality0 admits. No driver earlybelow cull; triangle routine clips.',
 'base_mode': 'mode0 onlywhenall3shades==0; otherwise5. Rawpointer16E098 fallback is unreachable in exact map region11C2BE..1542BE.',
 'shade': '4*signed16(cell+0)*u8(cell+12)+32768, pervertex; textureu8+4 andorientationbits70of+5 fromownerA/B.',
 'optional_calls': {'mode6': 'owner+13>1; tableindex2if>=97else(value-1)//32; texturepointer115E60[index], samegeometry/UV/shade', 'mode15': 'contextflags40 andowner+11bit0 andnotother+11bit20; other=Bforbothhalves'},
 'dirty_requests': 'Ifcontextflags4, beforeeachhalf cull requestownerA+11&=FB andownerB+11&=F7 respectively; plannerreturnsrequests ratherthanmutatinginput.',
 'outer_caller_scope': '83B20 main interval startscolumn0ifobject+1D==0else4, endsbeforetrunc(signedword+33/32)+start; optionalrightseam endsbefore20. Scrollwrap/remnant callers remain separate.',
 'synthetic_vectors': vectors, 'synthetic_cache_comparisons': checks,
 'actual_conq001_fixtures': actual, 'source_entry': 'root/2/disc/152', 'source_sha256': sha(level),
 'limits': ['Mode6/15 pixel semantics remain unimplemented by thisreceipt; do not silentlyclaimcompleteframewhileomittingthem.', 'Camera/initialfactor/viewrectangle choices arecallerinputs, not startupstateclaims.', 'Exact20-column scene and viewport placement require explicit outer-context values; thisAPIplansone7B7F0strip.']}
(P / 'research/ghidra/terrain-strip-evidence.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(f'PASS {len(listings)} contracts; {checks} direct-projection/cache strip comparisons; 3 exactCONQ001 fixtures; maximum48pairs admitted')
