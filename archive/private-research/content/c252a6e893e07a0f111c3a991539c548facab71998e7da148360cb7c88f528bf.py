"""Literal plant callback plus scheduler, including RNG, creation and newborn pass.

Private trajectory checkpoints are regenerated from a fresh Rust build. They
are input states, not expected values: the closed disassembly interpreter
independently executes the complete scheduler from each checkpoint.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,struct,subprocess,sys,uuid
from cargo import environment
P=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    work=P/'private/review/plant-reproduction'/uuid.uuid4().hex;work.mkdir(parents=True)
    subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
    target=P/'private/build/rust-target/debug';exe=work/'probe.exe';command=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-O','scripts/plant_reproduction_probe.rs','-L',f'dependency={target}/deps','-o',str(exe)]
    for crate in ['assets','runtime']:
        lib=target/f'libgenewars_{crate}.rlib'
        if not lib.exists():lib=max((target/'deps').glob(f'libgenewars_{crate}-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
        command+=['--extern',f'genewars_{crate}={lib}']
    result=subprocess.run(command,cwd=P,env=environment(),capture_output=True);assert result.returncode==0,result.stderr.decode(errors='replace')
    subprocess.run([str(exe),str(P),str(work)],cwd=P,check=True)
    spec=importlib.util.spec_from_file_location('plant_source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    class Source(m.Source):
        def get(self,operand):
            if operand=='0x00083140':self.plant_queue.append((self.r['EDX']-0x15d40e)//49)
            return super().get(operand)
    model=Source()
    relative=['entity-order/instructions-0006ca90.txt','mission-array-refs/instructions-0006d0b0.txt','mission-array-refs/instructions-0006c4b0.txt','plant-factory-1f340.txt','mission-tail-init/instructions-0006c690.txt','terrain-map-refs/instructions-0007b240.txt','plant-startup/instructions-0008b6f0.txt','plant-startup/instructions-0008b300.txt','terrain-map-refs/instructions-0006c780.txt','panel-activation-refs/instructions-000724c0.txt','entity-plant/instructions-0007af00.txt','display-geometry/instructions-000979e8.txt','plant-startup/instructions-000409f0.txt','plant-startup/audio-shared-tail.txt','entity-order/instructions-0001e760.txt']
    files=[P/'private/ghidra'/f for f in relative];model.code.update(m.m.Model(files,b'').code)
    model.code[0x2c200]=('RET',[]);order=sorted(model.code);model.next=dict(zip(order,order[1:]))
    cases=[];compared=0;instructions=0
    for line in (work/'cases.tsv').read_text().splitlines():
        name,tick,created,count=line.split('\t');directory=work/name;b=(directory/'before.bin').read_bytes();expected=(directory/'after.bin').read_bytes();seed,ring,draws=struct.unpack('<HBI',(directory/'rng-before.bin').read_bytes())
        state={0x118d60+i:v for i,v in enumerate(b[:0x355e])};state.update({0x11c2be+i:v for i,v in enumerate(b[0x355e:0x3b55e])})
        layouts=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
        for start,n,stride,address,native_stride in layouts:
            for i in range(n):
                raw=b[start+i*stride:start+(i+1)*stride];native=raw[:31]+bytes(8)+raw[31:]
                for j,v in enumerate(native):
                    if not 31<=j<39:state[address+i*native_stride+j]=v
        state[0x11872c]=ring
        for i,v in enumerate(struct.pack('<I',draws)):state[0x118782+i]=v
        model.plant_queue=[];report,_,trace=model.run(counter=int(tick),seed=seed,state_before=state,entry=0x6d0b0,instruction_budget=200000)
        after=bytearray(b);after[0x3552:0x3554]=model.load(0x11c2b2,2).to_bytes(2,'little');after[0x355a:0x355c]=model.load(0x11c2ba,2).to_bytes(2,'little')
        after[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
        for start,n,stride,address,native_stride in layouts[:2]:
            for i in range(n):
                raw=bytes(model.load(address+i*native_stride+j,1)for j in range(stride+8));after[start+i*stride:start+(i+1)*stride]=raw[:31]+raw[39:]
        (directory/'source-after.bin').write_bytes(after)
        assert after==expected,(name,'world',[(i,a,e)for i,(a,e)in enumerate(zip(after,expected))if a!=e][:16])
        rng=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4));assert rng==(directory/'rng-after.bin').read_bytes(),(name,'rng')
        queue=struct.pack('<'+'H'*len(model.plant_queue),*model.plant_queue);assert queue==(directory/'queue.bin').read_bytes(),(name,'queue')
        compared+=len(after)+len(rng)+len(queue);instructions+=report['steps'];cases.append(dict(case=name,created=int(created),plants=int(count),instructions=report['steps'],rng_draws=report['draws'],deferred_ui_calls=trace.count(0x2c200)))
        print(name,'PASS',flush=True)
    assert any(c['created']for c in cases)
    pins=['rust/crates/genewars-runtime/src/world.rs','rust/crates/genewars-runtime/src/world/plant_creation.rs','rust/crates/genewars-runtime/src/plant_spread.rs','rust/crates/genewars-runtime/src/actor_world/plant_updates.rs','rust/crates/genewars-runtime/src/actor_world/effect_updates.rs','scripts/plant_reproduction_probe.rs','scripts/validate_plant_reproduction.py']
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=compared,literal_instructions=instructions,implementation_pins={f:sha(P/f)for f in pins},listing_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files},private_run=work.relative_to(P).as_posix(),original_execution=False,scope='Complete6D0B0 family scheduler, literal6CA90 callback, target/environment/factory, newborn pass and exact ordered queue requests. Three seeded plant-only trajectories with other families frozen; shared and packed implementations match per-step reports/RNG and normalized checkpoint backing. UI2C200 and renderer queue bodies remain pending requests; no full game tick claimed.')
    (P/'research/integration/plant-reproduction-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=compared,literal_instructions=instructions)))
if __name__=='__main__':main()
