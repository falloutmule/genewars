"""B11 evidence probe F20: replay the F18 measured capture (frame 6010 slot 1)
through the unchanged owned-width reference model with a store-watch on the
record at 0x1542be+124 offsets 0x2d..0x32 inclusive. Logs (pc, record-offset,
width, value) for every store overlapping that window, plus value comparisons
of the final record against the F18 oracle output. Read-only evidence probe:
no tracked file is modified, no oracle arm changes.
"""
import importlib.util, json
from pathlib import Path
P = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('owned_width_source', P / 'research/variants/owned-mule-captured-width-review.py')
rv = importlib.util.module_from_spec(spec); spec.loader.exec_module(rv)
g = rv.g
SLOT = 1
BASE = 0x1542be + SLOT * 124
LO, HI = BASE + 0x2d, BASE + 0x33  # watch record bytes 0x2d..0x32
BEFORE = 'private/review/attempt31-6010-measure/campaign/capture/frame-6010-slot-1-blocked-before-owned'
CAPTURE = 'private/review/owned-mule-captured-width/59dc2afb22b64ab6bf89e1ccb85df79b'

class Watch(rv.Source):
    def __init__(self, backing):
        super().__init__(backing)
        self.writes = []
    def store(self, a, n, v):
        if a < HI and a + n > LO:
            self.writes.append((self.pc, a - BASE, n, v))
        super().store(a, n, v)

buffers = g.read_buffers(Path(BEFORE) if not BEFORE.startswith('/') else Path(BEFORE))
state, seed, resources = g.prepare(buffers)
g.k.put(state, 0x11878c, 1, 1)
vm = Watch(buffers['backing.bin'])
run, _, trace = vm.run(entry=0x5ed20, counter=6010, seed=seed, state_before=state,
                       registers={'EAX': BASE}, instruction_budget=500000)
captured = [int(l, 16) for l in open(P / CAPTURE / 'trace.txt') if l.strip()]
executed = set(trace)
print('instructions', run['steps'], 'draws', run['draws'],
      'trace_matches_capture', list(trace) == captured)

in_trace = [w for w in vm.writes if w[0] in executed]
print('writes overlapping 0x2d..0x32 during run:', len(vm.writes),
      'from executed pcs:', len(in_trace))
seen = set()
for pc, off, n, v in in_trace:
    key = (pc, off, n)
    if key in seen:
        continue
    seen.add(key)
    print(f'  pc=0x{pc:06x} rec+0x{off:02x} width={n} value=0x{v:x}')

# final record values
final = vm.load(BASE, 124)
before = buffers['creatures.bin'][SLOT * 124:(SLOT + 1) * 124]
print('before[0x2d:0x33]', before[0x2d:0x33].hex(), '-> after', final[0x2d:0x33].hex() if isinstance(final, bytes) else hex(final))
# per-byte final
vals = [vm.load(BASE + i, 1) for i in range(0x2d, 0x33)]
print('after bytes 0x2d..0x32:', ' '.join(f'{v:02x}' for v in vals))
