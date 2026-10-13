"""Exact mission, complete neutral6C4B0 factory versus owned plant/map/Mule afterimages."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,random,struct,subprocess,sys,uuid
from cargo import environment
P=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    work=P/'private/review/plant-factory'/uuid.uuid4().hex;work.mkdir(parents=True)
    spec=importlib.util.spec_from_file_location('factory_source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);model=m.Source()
    files=[P/'private/ghidra/mission-array-refs/instructions-0006c4b0.txt',P/'private/ghidra/plant-factory-1f340.txt',P/'private/ghidra/mission-tail-init/instructions-0006c690.txt',P/'private/ghidra/terrain-map-refs/instructions-0007b240.txt']
    model.code.update(m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]))
    mission=(P/'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes();assert hashlib.sha256(mission).hexdigest()=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    rand=random.Random(0x6c4b0);inputs=bytearray();expected=bytearray();steps=0
    points=[(0,0),(32767,-32768),(16384,15872),struct.unpack_from('<2h',mission,277838+7)]
    for case in range(24):
        selector=case%12;point=points[case]if case<4 else tuple(rand.randrange(-32768,32768)for _ in range(2));inputs+=struct.pack('<B2h',selector,*point)
        report,creature,_=model.run(seed=5428,state_before={0x11c2b2:40,0x11c2b3:0,0x11872c:0,**{0x118782+i:0 for i in range(4)}},entry=0x6c4b0,registers=dict(EAX=4,EDX=selector,EBX=int.from_bytes(struct.pack('<2h',*point),'little')),stack_fill=[0,0x55,0xff][case%3]);steps+=report['steps']
        slot=(model.r['EAX']-0x15d40e)//49;assert 0<=slot<512 and model.load(0x11c2b2,2)==41
        after=bytearray(mission);after[0x3552:0x3554]=struct.pack('<H',41)
        after[0x355e:0x355e+229376]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
        for i in range(512):
            native=bytes(model.load(0x15d40e+i*49+j,1)for j in range(49));after[277838+i*41:277838+(i+1)*41]=native[:31]+native[39:]
        native=bytearray(creature);native[31:39]=bytes(8);expected+=struct.pack('<H',slot)+after+native
    (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
    subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
    target=P/'private/build/rust-target/debug';exe=work/'probe.exe';command=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/plant_factory_probe.rs','-L',f'dependency={target}/deps','-o',str(exe)]
    for crate in ['assets','runtime']:
        lib=target/f'libgenewars_{crate}.rlib'
        if not lib.exists():lib=max((target/'deps').glob(f'libgenewars_{crate}-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
        command+=['--extern',f'genewars_{crate}={lib}']
    subprocess.run(command,cwd=P,env=environment(),capture_output=True,check=True);subprocess.run([str(exe),str(P),str(work)],cwd=P,check=True)
    actual=(work/'actual.bin').read_bytes();assert actual==expected,dict(actual=len(actual),expected=len(expected),first_difference=next((i for i,(a,e)in enumerate(zip(actual,expected))if a!=e),None))
    pins=['rust/crates/genewars-runtime/src/plant_spread.rs','rust/crates/genewars-runtime/src/world/plant_creation.rs','rust/crates/genewars-runtime/src/actor_world/plant_updates.rs','scripts/plant_factory_probe.rs','scripts/validate_plant_factory.py']
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=24,compared_bytes=len(actual),literal_instructions=steps,listing_pins={f.relative_to(P).as_posix():sha(f)for f in files},implementation_pins={f:sha(P/f)for f in pins},original_execution=False,
        scope='Complete neutral owner4 creation on exact mission: first inactive slot, quantized position, literal terrain height/shade calls, all512 packed plants, entire map/backing, source count and native Mule common links. Twelve selectors, signed boundaries, empty/plant/Mule occupied cells. Pointer gaps normalized. Reproduction caller/newborn scheduler and owner0..3 accounting remain separate.')
    (P/'research/integration/plant-factory-validation.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
if __name__=='__main__':main()
