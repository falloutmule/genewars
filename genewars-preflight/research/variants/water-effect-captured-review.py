"""Replay captured owned-world13780 passes through the private listing interpreter.

The recorded native trajectory is an input/output fixture, not the oracle.
This gate does not rebuild that trajectory or claim its current source pins.
"""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,importlib.util,json,struct,uuid
P=Path(__file__).resolve().parents[2]
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
OVERLAYS={1:'creatures.bin',3:'actors.bin',5:'effects.bin',8:'family8.bin'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--trajectory',type=Path,default=P/'private/review/family8-trajectory/76d5f278b3234c69ac0f562782193e51',help='Pinned corrected-footstep trajectory')
    args=parser.parse_args()
    args.state1=False;args.return_states=True;args.corrected=False
    assert not args.corrected or args.trajectory
    assert not args.return_states or args.trajectory
    trajectory=args.trajectory.resolve()if args.trajectory else None
    if trajectory:
        assert trajectory.is_relative_to((P/'private').resolve())
        provenance=json.loads((trajectory/'result.json').read_text(encoding='utf-8'))
        assert provenance['seed']==65535 and provenance['completed_frames']>=5862
        assert sha(trajectory/'probe.exe')==provenance['binary_sha256']
    version='011' if args.state1 else '010'
    frames_to_check=([6144,6145,6146,6150,6160,6180,6200,6210,6220,6230,6232,6233] if args.state1 else [6046,6047,6048,6049,6050,6051,6064,6079,6080,6100,6120,6142,6143])
    boundary=6234 if args.state1 else 6144
    if trajectory:frames_to_check=[6234,6240,6250,6260,6270,6280,6287,6288,6289];boundary=6290
    if args.return_states:
        assert provenance['completed_frames']>=5862
        frames_to_check=[5859,5860,5861,5862];boundary=None
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
    extra += [f'countdown-expiry/instructions-000{x}.txt'for x in ['12c29','12c59','12c64','12dae']]
    extra += ['mission-array-refs/instructions-0004e950.txt','terrain-map-refs/instructions-00049d30.txt','plant-command/instructions-0004b630.txt','mission-array-refs/instructions-0004b3b0.txt','actor-activation/instructions-0004b090.txt','landing-contact/instructions-0001e600.txt']
    files=[P/'private/ghidra'/f for f in extra];model.code.update(m.m.Model(files,b'').code)
    if trajectory:
        model.code={address:value for address,value in model.code.items()if not 0x11720<=address<=0x1188d}
        model.code.update(m.m.Model([P/'private/ghidra/countdown-expiry/instructions-00011720.txt'],b'').code)
    # Ghidra omits the explicit memory width in the A3 accumulator store.
    # The operand-size prefix and AX prove a16-bit store; generic text
    # parsing otherwise defaults32 and overwrites the first map bytes.
    object1=P/'private/ghidra/validated/memory-.object1.bin'
    assert object1.read_bytes()[0x39a4:0x39aa]==bytes.fromhex('66a3bcc21100')
    model.code[0x139a4]=('MOV',['word ptr [0x0011c2bc]','AX'])
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
        before={f.name:f.read_bytes()for f in(fixture/'before-effects').glob('*.bin')};actual={f.name:f.read_bytes()for f in(fixture/'after-effects').glob('*.bin')}
        if trajectory:
            for role,values in [('before-effects',before),('after-effects',actual)]:
                for name,value in values.items():assert hashlib.sha256(value).hexdigest()==provenance['captures'][f'frame{frame}/{role}/{name}']
        sizes={'actors.bin':4000,'backing.bin':343958,'creatures.bin':37200,'effects.bin':16200,'family8.bin':1632,'players.bin':7220,'rng.bin':7}
        assert {name:len(value)for name,value in before.items()}==sizes,(frame,'before fixture shape')
        assert {name:len(value)for name,value in actual.items()}==sizes,(frame,'after fixture shape')
        for role,values in [('before-effects',before),('after-effects',actual)]:
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
        report,_,trace=model.run(counter=frame,seed=seed,state_before=state,entry=0x4e950,stack_fill=[0,85,255][frame%3])
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
        callbacks=trace.count(0x4b630);assert callbacks>=1
        steps+=report['steps'];cases.append(dict(frame=frame,callbacks=callbacks,factory_calls=trace.count(0x101d0),literal_instructions=report['steps']));print(frame,'PASS',flush=True)
    assert provenance['completed_frames']>=5862
    assert all(case['frame'] in frames_to_check for case in cases)
    scope=f'Full13780 at {len(cases)} recorded seeded-world checkpoints. All backing/native pools/players/RNG compared; audio/UI/queue consumers remain explicit ports. '
    if args.corrected:scope+='Corrected actor footstep trajectory; family8 passes only are replayed here. '
    if trajectory:scope+=('Includes state3 return/ascent, complete103F0 unlink/724C0cleanup, count decrement and parent notification. 'if args.return_states else 'Includes state1 travel and positive-timer hovering. State3 return remains outside these fixtures. ')+'Compilation identity comes from the hash-checked isolated trajectory result; this replay does not rebuild it.'
    else:scope+='Includes same-frame children, particle factories, point acceptance and initial target selection'+(' plus state1 travel.'if args.state1 else '.')+' Captures are not rebuilt; current compilation source pins are not claimed. Positive-timer hovering remains outside these fixtures.'
    receipt=dict(status='passed-recorded-native-captures',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=compared,literal_instructions=steps,
        captured_probe_binary_sha256=sha(trajectory/'probe.exe'if trajectory else P/f'private/integration/family8-combined{version}-probe.exe'),fixture_stream_sha256=fixture_digest.hexdigest(),
        interpreter_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+[P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py',object1,P/'private/ghidra/validated/memory-.object3.bin',anbpath,P/'private/integration/game-runtime/effects/EFF.ANB',Path(__file__)]},
        original_execution=False,transactional_boundary=boundary,scope=scope)
    if trajectory:receipt['trajectory_provenance_sha256']=sha(trajectory/'result.json');receipt['compilation_source_pins']=provenance['source_pins']
    receipt['scope']='Seed65535 recorded full4E950 effects passes5859-5862. Full backing/native pools/players/RNG compared, including6DF30,code15 child22 factory, sameframe newborncallback,49D30 cleanup and nextframe childretirement. Palette/UI/queue consumers are explicit ports; otherfamilycallbacks are captureinputs.'
    receipt['adapted_from']={'path':'scripts/validate_family8_onward_captured_pass.py','sha256':sha(P/'scripts/validate_family8_onward_captured_pass.py')}
    (P/'research/variants/water-effect-captured-review.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:receipt[k]for k in ['status','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
