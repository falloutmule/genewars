"""Literal814A0 versus independent pure Rust command, no original execution."""
from pathlib import Path
import importlib.util,struct,json,random,subprocess,sys,uuid,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
s=importlib.util.spec_from_file_location('source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def put(state,a,n,v):state.update({a+i:(v>>(i*8))&255 for i in range(n)})
def main():
 w=P/'private/review/genepod-command'/uuid.uuid4().hex;w.mkdir(parents=True);listing=P/'private/ghidra/building-loader-refs/instructions-000814a0.txt';model=m.Source();model.mem={};model.pc=0;model.code.update(m.m.Model([listing],b'').code);model.code[0x21d70]=('RET',[]);order=sorted(model.code);model.next=dict(zip(order,order[1:]));assert model.load(0x10461c+4*45,4)==75
 rng=random.Random(814);inputs=bytearray();expected=bytearray();steps=0;cases=[]
 for index in range(240):
  a=bytearray(rng.randbytes(67));p=bytearray(rng.randbytes(1805));slot=index%256;grade=index%3;start=index%2==0;typ=(index//2)%2;refresh=index%4==0;caller=index%4;funds_id=(caller+1)%4
  a[6]=4;a[24]=caller;a[45]=0;a[46]=grade;a[49]|=1;struct.pack_into('<h',a,15,slot);p[1]=caller;p[2]=(index//4)%2;p[0x4a1]=funds_id
  primary=[0,1,2,1000,-3,2147483647][index%6];secondary=[0,1,500,1000,2000,-1][(index//6)%6];cap=[0,500,1000,2147483647][(index//12)%4];work=[0,1,18,75,100,-2][index%6];total=[0,18,37,75,-1,32767][(index//3)%6]
  for at,value in [(0x4a3,primary),(0x4a7,secondary),(0x5e6,cap)]:struct.pack_into('<i',p,at,value)
  struct.pack_into('<i',a,54,work);struct.pack_into('<h',a,52,total)
  base=0x1645ae+slot*67;player=0x118d74+caller*1805;payload=0x2e0000;state={base+i:x for i,x in enumerate(a)};state.update({player+i:x for i,x in enumerate(p)})
  put(state,0x118d74+funds_id*1805+2,1,typ);put(state,payload,2,4);put(state,payload+2,2,slot);put(state,payload+4,2,int(start));put(state,0x116974,4,0x1863a2 if refresh else 0);put(state,0x118740,4,base);put(state,0x185ede,4,0x2d0000);put(state,0x2d0004,4,0);put(state,0x185ec4,4,0x185ede);put(state,0x118782,4,123);put(state,0x11872c,1,7)
  r,_,trace=model.run(entry=0x814a0,registers={'EAX':player,'EDX':payload},state_before=state,seed=47117);steps+=r['steps'];assert r['draws']==0 and r['rng_seed_after']==47117 and r['rng_ring_after']==7
  after=bytes(model.load(base+i,1)for i in range(67))+bytes(model.load(player+i,1)for i in range(1805))+bytes([int(0x21d70 in trace)]);expected+=after;inputs+=a+p+struct.pack('<hhBBB',4,slot,start,typ,refresh)
  cases.append({'grade':grade,'start':start,'caller_type':p[2],'funds_identity_type':typ,'primary':primary,'secondary':secondary,'cap':cap,'work':work,'initial_work_word':total})
 (w/'input.bin').write_bytes(inputs);(w/'expected.bin').write_bytes(expected)
 exe=w/'review.exe';subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],cwd=P,env=environment(),check=True,capture_output=True)
 subprocess.run([str(exe),str(w)],cwd=P,env=environment(),check=True,capture_output=True);actual=(w/'actual.bin').read_bytes();diff=next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None)
 result={'status':'passed' if actual==expected else 'error','cases':len(cases),'compared_bytes':len(expected),'literal_instructions':steps,'unsupported_admission_checks':len(cases)*5,'first_difference':diff,'output':str(w),'scope':'814A0 handler numeric and explicit panel request only; upstream eligibility/UI producer unresolved; synthetic funds edge cases are source arithmetic evidence, not gameplay authorization','pins':{str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [listing,Path(__file__),Path(__file__).with_suffix('.rs'),P/'rust/crates/genewars-runtime/src/genepod_command.rs']}}
 (w/'result.json').write_text(json.dumps(result,indent=2)+'\n');Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
