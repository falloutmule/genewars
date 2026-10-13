"""Literal6C4B0/1F340/6C690 record initialization; links and sample bodies separate."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,random,struct,subprocess,sys,uuid
from cargo import environment
P=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    work=P/'private/review/plant-initialization'/uuid.uuid4().hex;work.mkdir(parents=True)
    spec=importlib.util.spec_from_file_location('init_source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    class Source(m.Source):
        def get(self,operand):
            if self.pc==0x1f364 and operand=='0x0007b140':
                self.samples.append(self.load(self.r['EAX'],4));self.r['EAX']=self.height&0xffffffff;return 0x2e0000
            if self.pc==0x1f36e and operand=='0x0007b240':
                self.samples.append(self.load(self.r['EAX'],4));self.r['EAX']=self.shade;return 0x2e0000
            if self.pc==0x6c61a and operand=='0x0008ac60':return 0x2e0000
            return super().get(operand)
    source=Source();files=[P/'private/ghidra/mission-array-refs/instructions-0006c4b0.txt',P/'private/ghidra/plant-factory-1f340.txt',P/'private/ghidra/mission-tail-init/instructions-0006c690.txt']
    source.code.update(m.m.Model(files,b'').code);source.code[0x2e0000]=('RET',[]);order=sorted(source.code);source.next=dict(zip(order,order[1:]))
    rand=random.Random(0x6c4b0);inputs=bytearray();expected=bytearray();steps=0
    for case in range(144):
        slot=[0,1,8,40,255,511][case%6];selector=case%12;point=tuple(rand.randrange(-32768,32768)for _ in range(2));height=rand.randrange(-2147483648,2147483648);shade=rand.randrange(256)
        prior=bytearray(rand.randbytes(49));prior[6]=0;prior[31:39]=bytes(8)
        inputs+=prior+struct.pack('<HB2hiB',slot,selector,*point,height,shade)
        address=0x15d40e+slot*49;state={address+i:b for i,b in enumerate(prior)}
        state.update({address+35+i:b for i,b in enumerate(struct.pack('<I',0x1120ac))})
        state.update({0x15d40e+i*49+6:2 for i in range(slot)});state.update({0x11c2b2:40,0x11c2b3:0})
        source.height=height;source.shade=shade;source.samples=[]
        report,_,_=source.run(seed=5428,state_before=state,entry=0x6c4b0,registers=dict(EAX=4,EDX=selector,EBX=int.from_bytes(struct.pack('<2h',*point),'little')),stack_fill=[0,0x55,0xff][case%3]);steps+=report['steps']
        assert source.r['EAX']==address and source.load(0x11c2b2,2)==41 and len(source.samples)==2
        result=bytearray(source.load(address+i,1)for i in range(49));result[31:39]=bytes(8);expected+=result+b''.join(struct.pack('<I',x)for x in source.samples)
    (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
    subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
    target=P/'private/build/rust-target/debug';exe=work/'probe.exe'
    subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/plant_initialization_probe.rs','-L',f'dependency={target}/deps','--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-o',str(exe)],cwd=P,env=environment(),capture_output=True,check=True)
    subprocess.run([str(exe),str(work)],cwd=P,check=True)
    actual=(work/'actual.bin').read_bytes();assert actual==expected,dict(actual=len(actual),expected=len(expected),first_difference=next((i for i,(a,e)in enumerate(zip(actual,expected))if a!=e),None))
    pins=['rust/crates/genewars-runtime/src/plant_spread.rs','scripts/plant_initialization_probe.rs','scripts/validate_plant_initialization.py']
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=144,compared_bytes=len(actual),literal_instructions=steps,listing_pins={f.relative_to(P).as_posix():sha(f)for f in files},implementation_pins={f:sha(P/f)for f in pins},original_execution=False,
        scope='Neutral owner4 record initialization, all selectors, first-free slots0/1/8/40/255/511, exact factory position and both terrain sample coordinates, retained unknown bytes. Height/shade bodies and link insertion replaced by explicit stubs; pointers31..39 normalized and resource selector retained as typed metadata. Does not prove complete creation/world scheduling.')
    (P/'research/integration/plant-initialization-validation.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
if __name__=='__main__':main()
