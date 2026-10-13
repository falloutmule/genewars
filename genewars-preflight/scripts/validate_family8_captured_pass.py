"""Replay captured owned-world13780 passes through the private listing interpreter.

The recorded native trajectory is an input/output fixture, not the oracle.
This gate does not rebuild that trajectory or claim its current source pins.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,struct,uuid
P=Path(__file__).resolve().parents[1]
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
OVERLAYS={1:'creatures.bin',3:'actors.bin',5:'effects.bin',8:'family8.bin'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);model=m.Source()
    extra=['mission-array-refs/instructions-00013780.txt','mission-array-refs/instructions-000101d0.txt','countdown-expiry/instructions-0001ee80.txt','mission-tail-init/instructions-00010060.txt','terrain-map-refs/instructions-0007b240.txt']
    extra += [f'countdown-expiry/instructions-000{x}.txt'for x in ['118e0','11943','1eab0','1eac0','1ea90','11270','11380']]
    extra += ['construction-command/instructions-0001ee40.txt']
    files=[P/'private/ghidra'/f for f in extra];model.code.update(m.m.Model(files,b'').code)
    for address,cleanup in [(0x1e810,8),(0x98210,0),(0x981d3,0),(0x41ce0,20),(0x41910,16),(0x984c0,0)]:model.code[address]=('RET',[hex(cleanup)]if cleanup else[])
    order=sorted(model.code);model.next=dict(zip(order,order[1:]))
    anbpath=P/'private/integration/game-runtime/countdown/ETHEREAL.ANB';anb=bytearray(anbpath.read_bytes())
    assert sha(anbpath)=='318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4'
    frames,animations=struct.unpack_from('<II',anb,10)
    for i in range(9):
        for off in [0,4]:struct.pack_into('<I',anb,animations+10*i+off,0x500000+frames+struct.unpack_from('<I',anb,animations+10*i+off)[0])
    work=P/'private/review/family8-captured-pass'/uuid.uuid4().hex;work.mkdir(parents=True)
    compared=steps=0;cases=[];fixture_digest=hashlib.sha256()
    for frame in range(6001,6046):
        fixture=P/f'private/integration/family8-combined009-seed5428-frame{frame}'
        before={f.name:f.read_bytes()for f in(fixture/'before').glob('*.bin')};actual={f.name:f.read_bytes()for f in(fixture/'after').glob('*.bin')}
        for role,values in [('before',before),('after',actual)]:
            for name in sorted(values):fixture_digest.update(f'{frame}/{role}/{name}\0'.encode()+values[name])
        b=before['backing.bin'];state={0x118d60+i:v for i,v in enumerate(b[:0x355e])}
        state.update({0x11c2be+i:v for i,v in enumerate(b[0x355e:0x3b55e])})
        state.update({0x118d74+i:v for i,v in enumerate(before['players.bin'])})
        for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
            for slot in range(count):
                if kind in OVERLAYS:native=bytearray(before[OVERLAYS[kind]][slot*width:(slot+1)*width])
                else:
                    raw=b[start+slot*stride:start+(slot+1)*stride];native=bytearray(raw[:31]+bytes(8)+raw[31:])
                if kind==8:
                    struct.pack_into('<I',native,0x23,0x11242c);struct.pack_into('<I',native,0x1f,0x500000+animations+80)
                state.update({address+slot*width+j:v for j,v in enumerate(native)})
        state.update({0x500000+i:v for i,v in enumerate(anb)})
        state.update({0x116820+i:v for i,v in enumerate(struct.pack('<I',0x500000+animations))})
        seed,ring,draws=struct.unpack('<HBI',before['rng.bin']);state[0x11872c]=ring;state[0x11878c]=4;state[0x118770]=0
        state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))})
        report,_,trace=model.run(counter=frame,seed=seed,state_before=state,entry=0x13780,stack_fill=[0,85,255][frame%3])
        expected={k:bytearray(v)for k,v in before.items()};out=expected['backing.bin']
        out[0x3559]=model.load(0x11c2b9,1);out[0x355c:0x355e]=model.load(0x11c2bc,2).to_bytes(2,'little')
        out[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
        for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
            for slot in range(count):
                native=bytearray(model.load(address+slot*width+j,1)for j in range(width))
                if kind in OVERLAYS:
                    name=OVERLAYS[kind];native[31:39]=before[name][slot*width+31:slot*width+39];expected[name][slot*width:(slot+1)*width]=native
                else:out[start+slot*stride:start+(slot+1)*stride]=native[:31]+native[39:]
        expected['players.bin']=bytes(model.load(0x118d74+i,1)for i in range(7220))
        expected['rng.bin']=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
        for name,value in expected.items():
            assert value==actual[name],(frame,name,[(i,a,b)for i,(a,b)in enumerate(zip(value,actual[name]))if a!=b][:16])
            compared+=len(value)
        callbacks=trace.count(0x118e0);assert callbacks==4
        steps+=report['steps'];cases.append(dict(frame=frame,callbacks=callbacks,factory_calls=trace.count(0x101d0),literal_instructions=report['steps']));print(frame,'PASS',flush=True)
    blocked=P/'private/integration/family8-combined009-seed5428-frame6046'
    assert all(f.read_bytes()==(blocked/'after'/f.name).read_bytes()for f in(blocked/'before').glob('*.bin'))
    receipt=dict(status='passed-recorded-native-captures',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=compared,literal_instructions=steps,
        captured_probe_binary_sha256=json.loads((P/'private/integration/family8-combined009-provenance.json').read_text())['captured_probe_binary_sha256'],fixture_stream_sha256=fixture_digest.hexdigest(),
        interpreter_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+[P/'research/variants/initial-creature-source.py',Path(__file__)]},
        original_execution=False,rollback_at_6046=True,scope='Full13780 across45 recorded seeded-world passes, including expiry factories, same-frame callbacks, all backing/native pools/players/RNG. Audio/UI/queue consumers are ports. Fixture binary identity retained; this gate does not rebuild the original trajectory or assert current production pins. Selector1 onward remains unproved here.')
    (P/'research/integration/family8-captured-pass-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:receipt[k]for k in ['status','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
