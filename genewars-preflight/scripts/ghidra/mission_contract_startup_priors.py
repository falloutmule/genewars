"""Literal fresh static constructor footprints, before any campaign mission."""
from pathlib import Path
import importlib.util,hashlib,json,struct
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Source(m.Source):
 def store(self,a,n,v):
  if getattr(self,'record_writes',False):self.writes.update(range(a,a+n))
  super().store(a,n,v)
 def get(self,o):
  if getattr(self,'pc',0)==self.entry:self.record_writes=True
  return super().get(o)
files=[P/'private/ghidra'/n for n in ['entity-shadow-panel/instructions-0002b7e0.txt','panel-activation-extra/instructions-00021940.txt','panel-activation-extra/instructions-000252b0.txt']]
files +=[P/'private/ghidra/mission-tail-constructors'/f'instructions-000{entry:x}.txt'for entry in [0x1e390,0x1e3e0,0x1e400,0x1e3f0,0x1e450,0x1e4b0,0x1e500]]
x=Source();x.code.update(m.m.Model(files,b'').code);order=sorted(x.code);x.next=dict(zip(order,order[1:]));obj=(P/'private/ghidra/validated/memory-.object3.bin').read_bytes();rows=[]
for kind,base,stride,count,entry in [(1,0x1542be,124,300,0x1e390),(2,0x15d40e,49,512,0x1e3e0),(3,0x16360e,200,20,0x1e400),(4,0x1645ae,67,256,0x1e3f0),(5,0x1688ae,81,200,0x1e450),(7,0x16ce16,97,32,0x1e4b0),(8,0x16da36,102,16,0x1e500)]:
 assert not any(obj[base-0x100000:base-0x100000+stride*count]);x.entry=entry;x.record_writes=False;x.writes=set()
 # Explicit zero mapped prior overrides the interpreter's unrelated creature fixture.
 state={base+i:0 for i in range(stride)};report,_,_=x.run(state_before=state,entry=entry,registers={'EAX':base})
 after=bytearray(x.load(base+i,1)for i in range(stride));vtable=struct.unpack_from('<I',after,35)[0];after[35:39]=bytes(4);assert not any(after),(kind,after.hex())
 rows.append({'kind':kind,'base':hex(base),'stride':stride,'count':count,'constructor':hex(entry),'vtable':hex(vtable),'nonpointer_bytes_zero':True,'instructions':report['steps']})
x.entry=0x2b7e0;x.record_writes=False;x.writes=set();report,_,_=x.run(state_before={},entry=x.entry,registers={'EAX':0x185dfc,'EDX':0,'EBX':0x106894});assert x.load(0x185ec4,4)==0
assert not any(a in x.writes for a in range(0x185ec4,0x185ec8))
r={'status':'passed-fresh-constructor-footprints','native_constructors':rows,'main_panel':{'base':'185DFC','panel_field':'185EC4','mapped_prior':0,'after_constructor':0,'constructor_instructions':report['steps'],'field_written':False},'source_pins':{f.relative_to(P).as_posix():hashlib.sha256(f.read_bytes()).hexdigest()for f in files},'object3_sha256':hashlib.sha256(obj).hexdigest(),'limits':['Fresh process mapped state only; does not authorize zero priors after an earlier mission.','Later1D770 pool resets set IDs for some families;71B70 serialized loaders overwrite those numeric fields.','MAINPANL field may change through prior panel activation; fresh menu route still needs module path audit.'],'original_execution':False}
(P/'research/campaign/startup-constructor-priors.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
