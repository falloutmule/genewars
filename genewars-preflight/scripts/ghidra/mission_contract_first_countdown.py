"""Literal first-callback countdown ordering for fresh Exceland only."""
from pathlib import Path
import importlib.util, hashlib, json
P = Path(__file__).resolve().parents[2]
s = importlib.util.spec_from_file_location('source', P/'research/variants/initial-creature-source.py')
m = importlib.util.module_from_spec(s); s.loader.exec_module(m)
files = [P/'private/ghidra/config-defaults/instructions-00013780.txt',
         P/'private/ghidra/mission-array-refs/instructions-0007a280.txt']
x = m.Source(); x.code.update(m.m.Model(files, b'').code)
# Stop at the explicit family8/presentation boundary, using the original epilogue.
# No claim is made about the omitted tail or countdown-zero activation.
x.code[0x139d2] = ('JMP', ['0x00013a33'])
order = sorted(x.code); x.next = dict(zip(order, order[1:]))
rows = []
for countdown in [1000, 999, 2, 65535]:
    state = {0x11c2bc:countdown & 255, 0x11c2bd:countdown >> 8,
             0x11945b:0, 0x11873d:0}
    report, _, _ = x.run(state_before=state, entry=0x7a280, registers={'EAX':0x118d74})
    assert x.load(0x11c2bc, 2) == countdown
    actor_steps = report['steps']
    report, _, _ = x.run(state_before=state, entry=0x13780)
    expected = countdown if countdown == 65535 else countdown - 1
    assert x.load(0x11c2bc, 2) == expected
    rows.append({'before':countdown, 'after_7a280':countdown,
                 'after_13780':expected, 'steps_7a280':actor_steps,
                 'steps_13780':report['steps']})
result = {'status':'passed-bounded-first-countdown', 'cases':rows,
          'source_pins':{f.relative_to(P).as_posix():hashlib.sha256(f.read_bytes()).hexdigest() for f in files},
          'first_callback':'Frame0: initial countdown1000 survives7A280; later13780 changes it to999. Outer frame increment follows callbacks and presentation.',
          'scope':'Progress0 and positive countdown excluding transition tozero. Does not prove countdown-zero activation or special progress14/20 branches.',
          'boundary':'139D2 redirected to original13A33 epilogue; family8 callbacks and presentation text tail are outside this prefix proof.',
          'original_execution':False}
(P/'research/campaign/first-countdown-order.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps(result))
