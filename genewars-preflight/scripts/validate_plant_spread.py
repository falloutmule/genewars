"""Literal6CCF6 target planning; environment queries and allocation are explicit boundaries."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,random,struct,subprocess,sys,uuid
from cargo import environment
P=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    work=P/'private/review/plant-spread'/uuid.uuid4().hex;work.mkdir(parents=True)
    spec=importlib.util.spec_from_file_location('spread_source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    class Stop(Exception):pass
    class Source(m.Source):
        def get(self,operand):
            if self.pc==0x6cfe9 and operand=='0x2e0010':raise Stop()
            if self.pc==0x6ce08 and operand=='0x0008b520':
                self.walked=struct.unpack('<2h',self.r['EDX'].to_bytes(4,'little'));self.queries.append((0,*self.walked));self.r['EAX']=int(self.mode==1);return 0x2e0000
            if self.pc==0x6ce23 and operand=='0x0008b6f0':
                self.cell=tuple((x>>8)&127 for x in self.walked);self.queries.append((1,*self.cell));self.r['EAX']=(1 if self.mode==2 else -1 if self.mode==6 else 0)&0xffffffff;return 0x2e0000
            if self.pc==0x6ce31 and operand=='byte ptr [EBX + 0xb]':
                self.queries.append((2,*self.cell));return int(self.mode==3)
            if self.pc==0x6cfa3 and operand=='0x0007b140':
                point=struct.unpack('<2h',self.load(self.r['EAX'],4).to_bytes(4,'little'));self.queries.append((3,*point));self.r['EAX']=(self.threshold+(512 if self.mode==4 else 511 if self.mode==5 else 513))&0xffffffff;return 0x2e0000
            if self.pc==0x6cfc5 and operand=='0x0006c4b0':
                self.factory=struct.unpack('<2h',self.r['EBX'].to_bytes(4,'little'));raise Stop()
            return super().get(operand)
    source=Source();listing=P/'private/ghidra/entity-order/instructions-0006ca90.txt'
    assert sha(listing)=='056c0c9a3cfab3cc4cfbfb15851895551fa672bb546f1e5f459f7a258315769b'
    source.code.update(m.m.Model([listing],b'').code);source.code[0x6cfe9]=('JMP',['0x2e0010']);source.code[0x2e0000]=('RET',[])
    order=sorted(source.code);source.next=dict(zip(order,order[1:]))
    rand=random.Random(0x6ccf6);inputs=bytearray();expected=bytearray();vectors=[]
    for steps in [0,1,3,8,31,255]:
      for seed in [0,5428,65535]:
       for mode in range(7):
        point=tuple(rand.randrange(-32768,32768)for _ in range(2));ring=rand.randrange(32);count=rand.choice([0,31,0xffffffff]);threshold=rand.choice([-32768,-512,0,816,32767]);deltas=[0,-1,2,-128,127,1,0,-2]
        vectors.append((point,steps,mode,seed,ring,count,threshold,deltas))
    for index,(point,steps,mode,seed,ring,count,threshold,deltas)in enumerate(vectors):
        inputs+=struct.pack('<2hBBHBIh8b',*point,steps,mode,seed,ring,count,threshold,*deltas)
        state={0x220000+i:0 for i in range(49)};state.update({0x220007+i:b for i,b in enumerate(struct.pack('<2h',*point))})
        state.update({0x230010+i:v&255 for i,v in enumerate(deltas)});state[0x230019]=steps;state[0x11872c]=ring
        state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',count))});state.update({0x11a9c3+i:b for i,b in enumerate(struct.pack('<h',threshold))})
        source.mode=mode;source.threshold=threshold;source.queries=[];source.factory=None
        try:source.run(seed=seed,state_before=state,entry=0x6ccf6,registers=dict(EDI=0x220000,EBP=0x230000),stack_fill=[0,0x55,0xff][index%3],instruction_budget=100000)
        except Stop:pass
        else:raise AssertionError('missing explicit source boundary')
        factory=source.factory or (0,0);rejection=0 if source.factory else 1 if mode==1 else 2 if mode in (2,6) else 3 if mode==3 else 4
        expected+=struct.pack('<BB4hHBI B',source.factory is not None,rejection,*source.walked,*factory,source.load(0x11878a,2),source.load(0x11872c,1),source.load(0x118782,4),len(source.queries))
        for query in source.queries:expected+=struct.pack('<B2h',*query)
    (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
    subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
    target=P/'private/build/rust-target/debug';exe=work/'probe.exe'
    subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/plant_spread_probe.rs','-L',f'dependency={target}/deps','--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-o',str(exe)],cwd=P,env=environment(),capture_output=True,check=True)
    subprocess.run([str(exe),str(P),str(work)],cwd=P,check=True)
    actual=(work/'actual.bin').read_bytes();assert actual==expected,dict(actual=len(actual),expected=len(expected),first_difference=next((i for i,(a,e)in enumerate(zip(actual,expected))if a!=e),None))
    files=['rust/crates/genewars-runtime/src/plant_spread.rs','scripts/plant_spread_probe.rs','scripts/validate_plant_spread.py']
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=len(vectors),compared_bytes=len(actual),listing_sha256=sha(listing),source_pins={f:sha(P/f)for f in files},output_sha256=hashlib.sha256(actual).hexdigest(),original_execution=False,
        scope='Literal6CCF6..6CFC5 arithmetic, exact ordered environment queries, all RNG seed/ring/count afterimages and every rejection path. Caller reproduction gates, environment bodies and plant factory are separate contracts; no reproduction-enabled world claimed.')
    (P/'research/integration/plant-spread-validation.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
if __name__=='__main__':main()
