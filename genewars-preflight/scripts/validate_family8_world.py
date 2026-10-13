"""Full owned-world factory afterimages versus literal source101D0 execution."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, importlib.util, json, struct, subprocess, sys, uuid
from cargo import environment
P=Path(__file__).resolve().parents[1]
ACTORS=343958
CREATURES=ACTORS+4000
FAMILY8=CREATURES+37200
PLAYERS=FAMILY8+1632
RNG=PLAYERS+7220
BINDINGS=RNG+7
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),
         (298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),
         (316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),
         (332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    work=P/'private/review/family8-world'/uuid.uuid4().hex;work.mkdir(parents=True)
    pins={f.relative_to(P).as_posix():sha(f) for f in (P/'rust/crates/genewars-runtime/src').rglob('*.rs')}
    subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
    assert all(sha(P/f)==h for f,h in pins.items()),'runtime sources changed during build'
    target=P/'private/build/rust-target/debug';exe=work/'probe.exe'
    cmd=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/family8_world_probe.rs','-L',f'dependency={target}/deps','-o',str(exe)]
    for crate in ['assets','runtime']:
        lib=target/f'libgenewars_{crate}.rlib'
        if not lib.exists():lib=max((target/'deps').glob(f'libgenewars_{crate}-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
        cmd+=['--extern',f'genewars_{crate}={lib}']
    result=subprocess.run(cmd,cwd=P,env=environment(),capture_output=True);assert result.returncode==0,result.stderr.decode(errors='replace')
    subprocess.run([str(exe),str(P),str(work)],check=True)
    spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);model=m.Source()
    files=[P/'private/ghidra'/f for f in ['mission-array-refs/instructions-000101d0.txt','countdown-expiry/instructions-0001ee80.txt','countdown-expiry/instructions-0001ea90.txt','mission-tail-init/instructions-00010060.txt','terrain-map-refs/instructions-0007b240.txt']]
    model.code.update(m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]))
    cases=[];instructions=0;compared=0
    for number,line in enumerate((work/'cases.tsv').read_text().splitlines()):
        name,*values=line.split('\t');selector,owner,x,y,height=map(int,values);directory=work/name
        before=(directory/'before.bin').read_bytes();actual=(directory/'after.bin').read_bytes()
        assert len(before)==BINDINGS+16
        state={0x118d60+i:b for i,b in enumerate(before[:0x355e])}
        state.update({0x11c2be+i:b for i,b in enumerate(before[0x355e:0x3b55e])})
        for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
            for slot in range(count):
                base={1:CREATURES,3:ACTORS,8:FAMILY8}.get(kind)
                if base is not None:native=bytearray(before[base+slot*width:base+(slot+1)*width])
                else:
                    raw=before[start+slot*stride:start+(slot+1)*stride];native=bytearray(raw[:31]+bytes(8)+raw[31:])
                if kind==8:
                    struct.pack_into('<I',native,0x23,0x11242c)
                    binding=before[BINDINGS+slot]
                    struct.pack_into('<I',native,0x1f,0x500000+(0x50 if binding==2 else 0))
                state.update({address+slot*width+j:b for j,b in enumerate(native)})
        state.update({0x116820+j:b for j,b in enumerate(struct.pack('<I',0x500000))})
        seed,ring,draws=struct.unpack_from('<HBI',before,RNG);state[0x11872c]=ring
        state.update({0x118782+j:b for j,b in enumerate(struct.pack('<I',draws))})
        report,_,_=model.run(seed=seed,state_before=state,stack_fill=[0,85,255][number%3],entry=0x101d0,
            registers=dict(EAX=selector,ECX=owner,EDX=int.from_bytes(struct.pack('<hh',x,y),'little'),EBX=height&0xffffffff))
        expected=bytearray(before);expected[0x3559]=model.load(0x11c2b9,1)
        expected[0x355e:0x3b55e]=bytes(model.load(0x11c2be+j,1)for j in range(229376))
        for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
            for slot in range(count):
                native=bytearray(model.load(address+slot*width+j,1)for j in range(width))
                base={1:CREATURES,3:ACTORS,8:FAMILY8}.get(kind)
                if base is not None:
                    native[31:39]=before[base+slot*width+31:base+slot*width+39]
                    expected[base+slot*width:base+(slot+1)*width]=native
                else:expected[start+slot*stride:start+(slot+1)*stride]=native[:31]+native[39:]
        returned=model.r['EAX']
        if returned:
            slot=(returned-0x16da36)//102
            offset=model.load(returned+0x1f,4)-0x500000;assert offset in [0,80]
            expected[BINDINGS+slot]=1 if offset==0 else 2
        struct.pack_into('<HBI',expected,RNG,model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
        (directory/'source-after.bin').write_bytes(expected)
        assert expected==actual,(name,[(i,a,b)for i,(a,b)in enumerate(zip(expected,actual))if a!=b][:16])
        compared+=len(actual);instructions+=report['steps'];cases.append(dict(case=name,created=bool(returned),instructions=report['steps']))
        print(name,'PASS',flush=True)
    pins.update({f:sha(P/f)for f in ['scripts/family8_world_probe.rs','scripts/validate_family8_world.py']})
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=compared,literal_instructions=instructions,implementation_pins=pins,
        listing_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files},private_run=work.relative_to(P).as_posix(),original_execution=False,
        scope='Three complete16-allocation trajectories and full-pool NULL attempts. All backing, native actor/creature/family8 records, players, RNG and typed bindings compared; normalized machine pointer bytes. Existing plant/Mule/family8 heads and wrapped positions. No callback or countdown scheduler claimed.')
    (P/'research/integration/family8-world-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=compared,literal_instructions=instructions)))
if __name__=='__main__':main()
