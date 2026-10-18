"""Owned selector1 retirement versus literal103F0, unlink and724C0 cleanup."""
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
    work=P/'private/review/family8-retirement'/uuid.uuid4().hex;work.mkdir(parents=True)
    paths=[f for crate in ['assets','core','runtime']for f in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')]
    paths += [P/'scripts/family8_retirement_probe.rs',Path(__file__)]
    pins={f.relative_to(P).as_posix():sha(f)for f in paths}
    subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
    assert all(sha(P/f)==h for f,h in pins.items()),'runtime sources changed during build'
    target=P/'private/build/rust-target/debug';exe=work/'probe.exe'
    cmd=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/family8_retirement_probe.rs','-L',f'dependency={target}/deps','-o',str(exe)]
    for crate in ['assets','runtime']:
        lib=target/f'libgenewars_{crate}.rlib'
        if not lib.exists():lib=max((target/'deps').glob(f'libgenewars_{crate}-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
        cmd+=['--extern',f'genewars_{crate}={lib}']
    result=subprocess.run(cmd,cwd=P,env=environment(),capture_output=True);assert result.returncode==0,result.stderr.decode(errors='replace')
    assert all(sha(P/f)==h for f,h in pins.items()),'sources changed during compilation'
    subprocess.run([str(exe),str(P),str(work)],check=True)
    spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);model=m.Source()
    files=[P/'private/ghidra'/f for f in ['countdown-expiry/instructions-000103f0.txt','panel-activation-refs/instructions-000724c0.txt']]
    # The native host has no original GUI pointer roots; audio is a typed request.
    for address in [0x1e800,0x1e7c0,0x409f0,0x2c200]:model.code[address]=('RET',[])
    model.code.update(m.m.Model(files,b'').code)
    model.code[0x97830]=('MOV',['AL','0']);model.code[0x97831]=('RET',[])
    order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
    cases=[];instructions=0;compared=0
    for number,line in enumerate((work/'cases.tsv').read_text().splitlines()):
        name,slot_text,selector_text,playing_text=line.split('\t');dead_slot=int(slot_text);selector=int(selector_text);playing=bool(int(playing_text));directory=work/name
        model.code[0x97830]=('MOV',['AL',str(int(playing))])
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
        state.update({0x185ec4+j:0 for j in range(4)})
        state.update({0x118d74+j:b for j,b in enumerate(before[PLAYERS:RNG])})
        seed,ring,draws=struct.unpack_from('<HBI',before,RNG);state[0x11872c]=ring
        state.update({0x118782+j:b for j,b in enumerate(struct.pack('<I',draws))})
        report,_,trace=model.run(seed=seed,state_before=state,stack_fill=[0,85,255][number%3],entry=0x103f0,
            registers=dict(EAX=0x16da36+dead_slot*102,EDX=0xabcdef))
        assert model.r['EDX']==0xabcdef,'saved parent register not restored'
        assert all(trace.count(a)==1 for a in [0x8acc0,0x724c0,0x409f0,0x2c200])
        assert trace.count(0x1e800)==(1 if selector==1 else 2)
        assert trace.count(0x1e7c0)==int(selector==0 and playing)
        assert trace.count(0x10421)==int(selector==0 and playing)
        assert trace.count(0x1042e)==int(selector==0) and trace.count(0x1043b)==int(selector==0)
        assert trace.count(0x1044a)==int(selector==1)
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
        assert model.load(0x16da36+dead_slot*102+6,1)==0
        expected[BINDINGS+dead_slot]=0
        expected[PLAYERS:RNG]=bytes(model.load(0x118d74+j,1)for j in range(7220))
        struct.pack_into('<HBI',expected,RNG,model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
        (directory/'source-after.bin').write_bytes(expected)
        assert expected==actual,(name,[(i,a,b)for i,(a,b)in enumerate(zip(expected,actual))if a!=b][:16])
        compared+=len(actual);instructions+=report['steps'];cases.append(dict(case=name,retired_slot=dead_slot,instructions=report['steps']))
        print(name,'PASS',flush=True)
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=compared,literal_instructions=instructions,implementation_pins=pins,captured_probe_binary_sha256=sha(exe),
        listing_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files},private_run=work.relative_to(P).as_posix(),original_execution=False,
        scope='144 selector0/1 retirements with both explicit audio38 playback-query results across three whole-world fixtures, exercising tail/middle/head unlink, exhausted pools, rejected repeat and slot reuse. All backing, native actor/creature/family8 records, players, RNG and typed bindings compared; machine pointer gaps normalized. Literal103F0/8ACC0/724C0 with explicit audio and native-empty GUI ports. Later parent caller writes and full callbacks are separate.')
    (P/'research/integration/family8-retirement-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=compared,literal_instructions=instructions)))
if __name__=='__main__':main()
