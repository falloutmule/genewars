"""Literal74A50 search arithmetic and query order, explicit environment stubs."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,random,struct,subprocess,sys
from cargo import environment
P=Path(__file__).resolve().parents[1];W=P/'private/review/target-resolver'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    W.mkdir(parents=True,exist_ok=True)
    spec=importlib.util.spec_from_file_location('resolver_source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    class BoundedStop(Exception):pass
    class Source(m.Source):
        def get(self,operand):
            if self.pc==0x74ba1 and operand=='0x00074270':
                if len(self.queries)==4096:raise BoundedStop()
                point=struct.unpack('<hh',self.load(self.r['EAX'],4).to_bytes(4,'little'));self.queries.append(point);self.r['EAX']=0;return 0x2e0000
            if self.pc==0x74bfb and operand=='0x0008b520':self.r['EAX']=0;return 0x2e0000
            if self.pc==0x74c49 and operand=='0x0008b130':
                x,y=self.queries[-1];accepted=[True,len(self.queries)>self.limit,False,((x^y)&255)<32][self.mode]
                self.r['EAX']=0 if accepted else 10;return 0x2e0000
            return super().get(operand)
    model=Source();listing=P/'private/ghidra/plant-command/instructions-00074a50.txt'
    assert sha(listing)=='c44b051dc39fd5998975fecccd10798615d09cba01a10c6e909646373358861a'
    extension=m.m.Model([listing],b'');model.code.update(extension.code);model.code[0x2e0000]=('RET',[])
    order=sorted(model.code);model.next=dict(zip(order,order[1:]));randomizer=random.Random(0x74a50)
    vectors=[]
    for requested in [(0,0),(64,0),(65,0),(-65,0),(0,65),(0,-65),(32000,-32000),(-32768,32767)]:
        for mode in range(4):vectors.append(((0,0),requested,mode,3))
    for i in range(128):vectors.append((tuple(randomizer.randrange(-32768,32768)for _ in range(2)),tuple(randomizer.randrange(-32768,32768)for _ in range(2)),i%4,i%33))
    inputs=bytearray();expected=bytearray();steps=0;max_queries=0;bounded_cases=[]
    for index,(current,requested,mode,limit)in enumerate(vectors):
        inputs+=struct.pack('<4hBH',*current,*requested,mode,limit)
        state={0x220000+i:b for i,b in enumerate(struct.pack('<2h',*current))};state[0x230006]=3
        state.update({0x300004+i:0 for i in range(4)})
        model.queries=[];model.mode=mode;model.limit=limit
        try:
            report,_,_=model.run(entry=0x74a50,state_before=state,registers=dict(EAX=0x220000,EDX=int.from_bytes(struct.pack('<2h',*requested),'little'),ECX=0x230000,EBX=0),stack_fill=[0,0x55,0xff][index%3],instruction_budget=2000000)
            point=struct.unpack('<2h',model.r['EAX'].to_bytes(4,'little'));status=0;steps+=report['steps']
        except BoundedStop:
            point=(0,0);status=1;bounded_cases.append(index)
        except AssertionError:
            print(dict(case=index,current=current,requested=requested,mode=mode,first=model.queries[:8],last=model.queries[-3:]),flush=True)
            raise
        expected+=struct.pack('<BH2h',status,len(model.queries),*point)
        for query in model.queries:expected+=struct.pack('<2h',*query)
        max_queries=max(max_queries,len(model.queries))
    (W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected)
    subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
    target=P/'private/build/rust-target/debug';exe=W/'probe.exe'
    subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/target_resolver_probe.rs','-L',f'dependency={target}/deps','--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-o',str(exe)],cwd=P,env=environment(),capture_output=True,check=True)
    subprocess.run([str(exe),str(P)],cwd=P,env=environment(),check=True)
    actual=(W/'actual.bin').read_bytes();assert actual==expected,dict(actual=len(actual),expected=len(expected),first_difference=next((i for i,(a,e)in enumerate(zip(actual,expected))if a!=e),None))
    files=['scripts/validate_target_resolver.py','scripts/target_resolver_probe.rs','rust/crates/genewars-runtime/src/target_resolver.rs','research/variants/initial-creature-source.py']
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=len(vectors),compared_bytes=len(actual),max_queries=max_queries,port_limit_cases=bounded_cases,terminated_case_literal_instruction_steps=steps,
        listing_sha256=sha(listing),source_pins={p:sha(P/p)for p in files},output_sha256=hashlib.sha256(actual).hexdigest(),original_execution=False,
        scope='Literal74A50 numeric search and every queried point, terminal snap, signed wrap and angular boundaries. Obstacle/terrain/blocker calls replaced by explicit deterministic environment fixtures; their bodies and gameplay query admission are separate evidence. 4096-iteration guard is a port bound, never a fabricated success.')
    (P/'research/integration/target-resolver-validation.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
if __name__=='__main__':main()
