"""Replay captured owned-world13780 passes through the private listing interpreter.

The recorded native trajectory is an input/output fixture, not the oracle.
This gate does not rebuild that trajectory or claim its current source pins.
"""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,importlib.util,json,struct,uuid
P=Path(__file__).resolve().parents[1]
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
OVERLAYS={1:'creatures.bin',3:'actors.bin',5:'effects.bin',8:'family8.bin'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state1',action='store_true',help='Compare the later recorded travel trajectory through6233')
    parser.add_argument('--trajectory',type=Path,help='Pinned isolated trajectory directory for hover checkpoints')
    parser.add_argument('--return-states',action='store_true',help='Compare return, ascent and retirement through6441 in the supplied trajectory')
    parser.add_argument('--corrected',action='store_true',help='Compare corrected-footstep trajectory through6331')
    args=parser.parse_args()
    assert not args.corrected or args.trajectory
    assert not args.return_states or args.trajectory
    trajectory=args.trajectory.resolve()if args.trajectory else None
    if trajectory:
        assert trajectory.is_relative_to((P/'private').resolve())
        provenance=json.loads((trajectory/'result.json').read_text(encoding='utf-8'))
        assert provenance['seed']==5428 and provenance['completed_frames']>=6289
        assert sha(trajectory/'probe.exe')==provenance['binary_sha256']
    version='011' if args.state1 else '010'
    frames_to_check=([6144,6145,6146,6150,6160,6180,6200,6210,6220,6230,6232,6233] if args.state1 else [6046,6047,6048,6049,6050,6051,6064,6079,6080,6100,6120,6142,6143])
    boundary=6234 if args.state1 else 6144
    if trajectory:frames_to_check=[6234,6240,6250,6260,6270,6280,6287,6288,6289];boundary=6290
    if args.return_states:
        assert provenance['completed_frames']>=6441
        frames_to_check=[6290,6291,6300,6320,6340,6360,6380,6400,6410,6420,6430,6440,6441];boundary=6442
    if args.corrected:
        assert provenance['completed_frames']>=6331
        frames_to_check=[6001,6045,6046,6050,6080,6143,6144,6200,6250,6290,6300,6310,6320,6330,6331];boundary=6332
    spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);model=m.Source()
    extra=['mission-array-refs/instructions-00013780.txt','mission-array-refs/instructions-000101d0.txt','countdown-expiry/instructions-0001ee80.txt','mission-tail-init/instructions-00010060.txt','terrain-map-refs/instructions-0007b240.txt']
    extra += [f'countdown-expiry/instructions-000{x}.txt'for x in ['118e0','11943','1eab0','1eac0','1ea90','11270','11380','11aeb','12e11','114b0','13e60']]
    extra += ['construction-command/instructions-0001ee40.txt','actor-activation/instructions-00073e00.txt','mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt']
    extra += [f'countdown-expiry/instructions-000{x}.txt'for x in ['13108','11400','73a50']]
    extra += ['landing-contact/instructions-0001f030.txt']
    if trajectory:extra += [f'countdown-expiry/instructions-000{x}.txt'for x in ['11720','1ea80']]
    if args.return_states or args.corrected:
        extra += ['countdown-expiry/instructions-000132fa.txt','initial-creature/instructions-0006d230.txt','terrain-map-refs/instructions-000305f0.txt','countdown-expiry/instructions-000103f0.txt','panel-activation-refs/instructions-000724c0.txt']
    files=[P/'private/ghidra'/f for f in extra];model.code.update(m.m.Model(files,b'').code)
    if trajectory:
        model.code={address:value for address,value in model.code.items()if not 0x11720<=address<=0x1188d}
        model.code.update(m.m.Model([P/'private/ghidra/countdown-expiry/instructions-00011720.txt'],b'').code)
    for address,cleanup in [(0x1e810,8),(0x98210,0),(0x981d3,0),(0x41ce0,20),(0x41910,16),(0x984c0,0)]:model.code[address]=('RET',[hex(cleanup)]if cleanup else[])
    model.code[0x97830]=('XOR',['AL','AL']);model.code[0x97831]=('RET',[])
    if args.return_states or args.corrected:
        for address in [0x1e800,0x409f0,0x2c200]:model.code[address]=('RET',[])
    order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
    anbpath=P/'private/integration/game-runtime/countdown/ETHEREAL.ANB';anb=bytearray(anbpath.read_bytes())
    assert sha(anbpath)=='318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4'
    frames,animations=struct.unpack_from('<II',anb,10)
    for i in range(9):
        for off in [0,4]:struct.pack_into('<I',anb,animations+10*i+off,0x500000+frames+struct.unpack_from('<I',anb,animations+10*i+off)[0])
    eff=bytearray((P/'private/integration/game-runtime/effects/EFF.ANB').read_bytes());eff_frames,eff_animations=struct.unpack_from('<II',eff,10)
    for i in range(struct.unpack_from('<H',eff)[0]):
        for off in [0,4]:struct.pack_into('<I',eff,eff_animations+10*i+off,0x610000+eff_frames+struct.unpack_from('<I',eff,eff_animations+10*i+off)[0])
    work=P/'private/review/family8-captured-pass'/uuid.uuid4().hex;work.mkdir(parents=True)
    compared=steps=0;cases=[];fixture_digest=hashlib.sha256()
    for frame in frames_to_check:
        fixture=trajectory/f'frame{frame}'if trajectory else P/f'private/integration/family8-combined{version}-seed5428-frame{frame}'
        before={f.name:f.read_bytes()for f in(fixture/'before').glob('*.bin')};actual={f.name:f.read_bytes()for f in(fixture/'after').glob('*.bin')}
        if trajectory:
            for role,values in [('before',before),('after',actual)]:
                for name,value in values.items():assert hashlib.sha256(value).hexdigest()==provenance['captures'][f'frame{frame}/{role}/{name}']
        sizes={'actors.bin':4000,'backing.bin':343958,'creatures.bin':37200,'effects.bin':16200,'family8.bin':1632,'players.bin':7220,'rng.bin':7}
        assert {name:len(value)for name,value in before.items()}==sizes,(frame,'before fixture shape')
        assert {name:len(value)for name,value in actual.items()}==sizes,(frame,'after fixture shape')
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
                vt={1:0x111fec,2:0x1120ac,3:0x112194,5:0x112254,8:0x11242c}.get(kind)
                if vt:struct.pack_into('<I',native,35,vt)
                if kind==5:struct.pack_into('<I',native,31,0x610000+eff_animations+native[69]*10)
                if kind==8:
                    struct.pack_into('<I',native,0x23,0x11242c);struct.pack_into('<I',native,0x1f,0x500000+animations+(0 if native[0x55]==1 else 80))
                state.update({address+slot*width+j:v for j,v in enumerate(native)})
        if args.return_states or args.corrected:state.update({0x185ec4+i:0 for i in range(4)})
        state.update({0x500000+i:v for i,v in enumerate(anb)})
        state.update({0x610000+i:v for i,v in enumerate(eff)})
        state.update({0x116818+i:v for i,v in enumerate(struct.pack('<I',0x610000+eff_animations))})
        state.update({0x116820+i:v for i,v in enumerate(struct.pack('<I',0x500000+animations))})
        seed,ring,draws=struct.unpack('<HBI',before['rng.bin']);state[0x11872c]=ring;state[0x11878c]=4;state[0x118770]=0
        state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))})
        report,_,trace=model.run(counter=frame,seed=seed,state_before=state,entry=0x13780,stack_fill=[0,85,255][frame%3])
        expected={k:bytearray(v)for k,v in before.items()};out=expected['backing.bin']
        out[0x3554:0x3556]=model.load(0x11c2b4,2).to_bytes(2,'little')
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
        callbacks=trace.count(0x118e0);assert callbacks==sum(actual['family8.bin'][i*102+6]!=0 for i in range(16))+trace.count(0x103f0)
        steps+=report['steps'];cases.append(dict(frame=frame,callbacks=callbacks,factory_calls=trace.count(0x101d0),literal_instructions=report['steps']));print(frame,'PASS',flush=True)
    blocked=trajectory/f'frame{boundary}'if trajectory else P/f'private/integration/family8-combined{version}-seed5428-frame{boundary}'
    assert all(f.read_bytes()==(blocked/'after'/f.name).read_bytes()for f in(blocked/'before').glob('*.bin'))
    scope=f'Full13780 at {len(cases)} recorded seeded-world checkpoints. All backing/native pools/players/RNG compared; audio/UI/queue consumers remain explicit ports. '
    if args.corrected:scope+='Corrected actor footstep trajectory; family8 passes only are replayed here. '
    if trajectory:scope+=('Includes state3 return/ascent, complete103F0 unlink/724C0cleanup, count decrement and parent notification. 'if args.return_states else 'Includes state1 travel and positive-timer hovering. State3 return remains outside these fixtures. ')+'Compilation identity comes from the hash-checked isolated trajectory result; this replay does not rebuild it.'
    else:scope+='Includes same-frame children, particle factories, point acceptance and initial target selection'+(' plus state1 travel.'if args.state1 else '.')+' Captures are not rebuilt; current compilation source pins are not claimed. Positive-timer hovering remains outside these fixtures.'
    receipt=dict(status='passed-recorded-native-captures',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=compared,literal_instructions=steps,
        captured_probe_binary_sha256=sha(trajectory/'probe.exe'if trajectory else P/f'private/integration/family8-combined{version}-probe.exe'),fixture_stream_sha256=fixture_digest.hexdigest(),
        interpreter_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+[P/'research/variants/initial-creature-source.py',Path(__file__)]},
        original_execution=False,transactional_boundary=boundary,scope=scope)
    if trajectory:receipt['trajectory_provenance_sha256']=sha(trajectory/'result.json');receipt['compilation_source_pins']=provenance['source_pins']
    (P/('research/integration/family8-corrected-captured-pass-validation.json'if args.corrected else 'research/integration/family8-return-captured-pass-validation.json'if args.return_states else 'research/integration/family8-hover-captured-pass-validation.json'if trajectory else 'research/integration/family8-state1-captured-pass-validation.json' if args.state1 else 'research/integration/family8-onward-captured-pass-validation.json')).write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:receipt[k]for k in ['status','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
