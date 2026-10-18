"""Owned-world source-building composition; fixtures, not ordinary gameplay acceptance."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, importlib.util, json, struct, subprocess, sys, uuid
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from cargo import environment
P=Path(__file__).resolve().parents[2]
from validate_building_world import ACTORS, CREATURES, BUILDINGS, PLAYERS, RNG, BINDINGS, LAYOUTS, sha

def main():
    label='source-building-world'
    probe='scripts/desktop/source_building_world_harness.rs'
    work=P/'private/review'/label/uuid.uuid4().hex;work.mkdir(parents=True)
    source_paths=[f for crate in ['assets','core','runtime'] for f in (P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')]+[P/probe,Path(__file__)]
    pins={f.relative_to(P).as_posix():sha(f) for f in source_paths}
    build=subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,text=True)
    (work/'build.log').write_text(build.stdout+build.stderr);build.check_returncode()
    assert all(sha(P/f)==h for f,h in pins.items()),'runtime sources changed during build'
    target=P/'private/build/rust-target/debug';exe=work/'probe.exe'
    cmd=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',probe,'-L',f'dependency={target}/deps','-o',str(exe)]
    for crate in ['assets','runtime']:
        lib=target/f'libgenewars_{crate}.rlib'
        if not lib.exists():lib=max((target/'deps').glob(f'libgenewars_{crate}-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
        cmd+=['--extern',f'genewars_{crate}={lib}']
    result=subprocess.run(cmd,cwd=P,env=environment(),capture_output=True);assert result.returncode==0,result.stderr.decode(errors='replace')
    subprocess.run([str(exe),str(P),str(work)],check=True)
    spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    class Source(m.Source):
        def get(self,operand):
            if operand=='byte ptr [0x0011873d]':return 255
            return super().get(operand)
    model=Source()
    files=[P/'private/ghidra'/f for f in ['mission-array-refs/instructions-00042fa0.txt','mission-array-refs/instructions-00043e30.txt','mission-array-refs/instructions-00043560.txt','mission-tail-init/instructions-00044ff0.txt','terrain-map-refs/instructions-0007b240.txt']]
    files += [P/'private/ghidra'/f for f in ['resource-helpers/instructions-000441c0.txt','resource-helpers/instructions-00043f60.txt','resource-helpers/instructions-00043620.txt','mission-array-refs/instructions-0006a6d0.txt','construction-command/instructions-0005e0a3.txt']]
    files += [P/'private/ghidra/building-update/instructions-000451f0.txt',P/'private/ghidra/landing-contact/instructions-0007ac00.txt']
    model.code.update(m.m.Model(files,b'').code)
    model.code[0x43fd8]=('CALL',['0x2e0000']);model.code[0x2e0000]=('RET',['0x8'])
    for pc in [0x47e21,0x47e34,0x47f34]:model.code[pc]=('CALL',['0x2e0010'])
    model.code[0x2e0010]=('RET',[])
    order=sorted(model.code);model.next=dict(zip(order,order[1:]))
    cases=[];instructions=0;compared=0
    for number,line in enumerate((work/'cases.tsv').read_text().splitlines()):
        name,*values=line.split('\t');directory=work/name
        selector,owner,selected_slot,increment,loading,callback=map(int,values)
        resource=P/f"private/integration/game-runtime/buildings/{ {3:'BPOWE',6:'BSOLA',11:'BMINE'}[selector] }.ANB"
        anb=bytearray(resource.read_bytes());frame_table,animation_table=struct.unpack_from('<II',anb,10)
        for i in range(5):
            for offset in [0,4]:struct.pack_into('<I',anb,animation_table+10*i+offset,0x620000+frame_table+struct.unpack_from('<I',anb,animation_table+10*i+offset)[0])
        before=(directory/'before.bin').read_bytes();actual=(directory/'after.bin').read_bytes()
        assert len(before)==BINDINGS+256
        state={0x118d60+i:b for i,b in enumerate(before[:0x355e])}
        state.update({0x11c2be+i:b for i,b in enumerate(before[0x355e:0x3b55e])})
        for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
            for slot in range(count):
                base={1:CREATURES,3:ACTORS,4:BUILDINGS}.get(kind)
                if base is not None:native=bytearray(before[base+slot*width:base+(slot+1)*width])
                else:
                    raw=before[start+slot*stride:start+(slot+1)*stride];native=bytearray(raw[:31]+bytes(8)+raw[31:])
                if kind==4:
                    struct.pack_into('<I',native,0x23,0x112120)
                    binding=before[BINDINGS+slot]
                    if binding:struct.pack_into('<I',native,0x1f,0x620000+animation_table if binding==selector+1 else 0x500000+(binding-1)*0x100)
                state.update({address+slot*width+j:b for j,b in enumerate(native)})
        for index in range(14):state.update({0x11685c+index*4+j:b for j,b in enumerate(struct.pack('<I',0x500000+index*0x100))})
        state.update({0x118d74+j:b for j,b in enumerate(before[PLAYERS:RNG])})
        seed,ring,draws=struct.unpack_from('<HBI',before,RNG);state[0x11872c]=ring
        state.update({0x118782+j:b for j,b in enumerate(struct.pack('<I',draws))})
        state.update({0x620000+i:b for i,b in enumerate(anb)})
        state[0x11876c]=loading;entry=0x451f0 if callback else 0x441c0
        registers=dict(EAX=0x1645ae+selected_slot*67,EDX=increment,EBX=0)
        report,_,_=model.run(seed=seed,state_before=state,stack_fill=[0,85,255][number%3],entry=entry,registers=registers)
        expected=bytearray(before)
        expected[0x355e:0x3b55e]=bytes(model.load(0x11c2be+j,1)for j in range(229376))
        for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
            for slot in range(count):
                native=bytearray(model.load(address+slot*width+j,1)for j in range(width))
                base={1:CREATURES,3:ACTORS,4:BUILDINGS}.get(kind)
                if base is not None:
                    native[31:39]=before[base+slot*width+31:base+slot*width+39]
                    if kind==4 and native[6]!=0:native[31:35]=bytes(4)
                    expected[base+slot*width:base+(slot+1)*width]=native
                else:expected[start+slot*stride:start+(slot+1)*stride]=native[:31]+native[39:]
        expected[PLAYERS:RNG]=bytes(model.load(0x118d74+j,1)for j in range(7220))
        struct.pack_into('<HBI',expected,RNG,model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
        (directory/'source-after.bin').write_bytes(expected)
        assert expected==actual,(name,[(i,a,b)for i,(a,b)in enumerate(zip(expected,actual))if a!=b][:16])
        compared+=len(actual);instructions+=report['steps'];cases.append(dict(case=name,operation='callback'if callback else 'progress',instructions=report['steps']))
        print(name,'PASS',flush=True)
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=compared,literal_instructions=instructions,implementation_pins=pins,
        listing_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files},private_run=work.relative_to(P).as_posix(),original_execution=False,
        scope='Fixture composition only: selectors3/6/11, all4 owners and bothloadingflags; factory-created owned-world records stepped through exact-resource incomplete callbacks and grade0 completion. Complete backing, native actor/creature/building/player/RNG/binding afterimages compared. UI/audio/queue ports explicit. Mismatchedresource and repeatedcompletion must reject with entire ActorWorld unchanged. Not ordinary gameplay or authentic mission acceptance.')
    (P/f'research/integration/{label}.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=compared,literal_instructions=instructions)))
if __name__=='__main__':main()
