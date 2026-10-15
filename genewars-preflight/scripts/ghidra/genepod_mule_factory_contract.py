"""Literal null-parent species4 factory fixture; no native factory claim."""
from pathlib import Path
import hashlib,importlib.util,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
vm=m.Source();files=[G/'activation-rng-refs/instructions-0005a210.txt',G/'genepod-mule-initializer.txt',G/'construction-command/instructions-0001ee50.txt',G/'bones-frame/instructions-0005a1c0.txt',G/'player-creation/instructions-0001f480.txt',G/'terrain-map-refs/instructions-0007b240.txt']
vm.code.update(m.m.Model(files,b'').code);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
def put(s,a,n,v):s.update({a+i:b for i,b in enumerate(int(v&((1<<(8*n))-1)).to_bytes(n,'little'))})
cases=[]
for seed in [0,5428,10548,65535]:
 for fill in [0,165]:
  s={0x1542be+i:fill for i in range(300*124)}
  for slot in range(300):put(s,0x1542be+slot*124+6,1,0);put(s,0x1542be+slot*124+35,4,0x111fec)
  # Explicit empty flat synthetic terrain: valid type-zero cell heads.
  terrain=bytearray(229376)
  for c in range(16384):struct.pack_into('<h',terrain,c*14,100);terrain[c*14+12]=128
  s.update({0x11c2be+i:v for i,v in enumerate(terrain)})
  for a,n,v in [(0x11c2b0,2,0),(0x118d74+0x6f9,4,0),(0x180e84+4*4,4,0x400000),(0x300004,4,0),(0x118782,4,19),(0x11872c,1,4)]:put(s,a,n,v)
  r,_,trace=vm.run(seed=seed,state_before=s,entry=0x5a210,registers={'EAX':0,'EDX':4,'EBX':(1024<<16)|1024,'ECX':35})
  b=bytes(vm.load(0x1542be+i,1)for i in range(124))
  assert vm.r['EAX']==0x1542be and b[6]==1 and b[24]==0 and b[0x7a]==4 and b[0x76]==35
  assert b[0x5e]==8 and b[0x5f]==0 and struct.unpack_from('<h',b,0x6c)[0]==0
  assert vm.load(0x11c2b0,2)==1 and vm.load(0x118d74+0x6f9,4)==1
  assert vm.load(0x118782,4)==22
  expected=seed
  for _ in range(3):expected=(expected*0xbb40e62d)&65535
  assert vm.load(0x11878a,2)==expected
  names={0x11:'ground',0x27:'physical_height'}
  fields={name:struct.unpack_from('<i',b,o)[0]for o,name in names.items()}
  fields.update({hex(o):struct.unpack_from('<h',b,o)[0]for o in [0x2b,0x39,0x3b,0x3d,0x62,0x64,0x66,0x68,0x6a,0x6c]})
  fields.update({hex(o):b[o]for o in [0x19,0x53,0x76,0x77,0x78,0x79,0x7b]})
  cases.append({'seed':seed,'prior_fill':fill,'instructions':r['steps'],'draws':3,'rng_after':expected,'fields':fields,'record_sha256':hashlib.sha256(b).hexdigest(),'null_parent_branch':0x5a4c4 in trace})
out={'status':'passed','scope':'Full literal5A210 null-parent owner0/species4/quality35 on explicitly synthetic empty flat terrain; no native factory or campaign acceptance. Prior fills expose retained bytes; no assumed zeroing. Genepod caller ejection overwrite is outside this probe.','cases':cases,'source_pins':{p.relative_to(P).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'original_execution':False}
(P/'research/campaign/krell-genepod-mule-factory.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'status':'passed','cases':len(cases),'draws_per_factory':3,'fields':cases[0]['fields']}))
