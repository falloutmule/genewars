"""Literal arrival slice: reserved and unreserved living plants, synthetic inputs."""
from pathlib import Path
import importlib.util,json,struct,hashlib,uuid
P=Path(__file__).resolve().parents[2];f=P/'research/variants/initial-creature-source.py';sp=importlib.util.spec_from_file_location('s',f);m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-mule-callbacks-009/frame-461-slot-1-before';original=(fixture/'creatures.bin').read_bytes()[124:248];packed=(fixture/'backing.bin').read_bytes()[277838:277879]
cases=[]
for flags in [0,1]:
 a=bytearray(original);a[0x60]&=0x7f;struct.pack_into('<H',a,0x49,0x200);struct.pack_into('<I',a,35,0x111fec)
 plant=bytearray(packed[:31]+bytes(8)+packed[31:]);plant[0x2e]=flags;struct.pack_into('<h',plant,0x2c,1);struct.pack_into('<I',plant,35,0x1120ac)
 state={0x15433a+i:v for i,v in enumerate(a)};state.update({0x15d40e+i:v for i,v in enumerate(plant)});state.update({0x3200a6+i:v for i,v in enumerate(struct.pack('<I',0x15d40e))})
 model=m.Source();model.code[0x65b48]=('RET',[])
 r,_,trace=model.run(counter=461,seed=59105,state_before=state,entry=0x65957,registers={'ESI':0x15433a,'EBP':0x320000},instruction_budget=10000)
 after=bytes(model.load(0x15433a+i,1)for i in range(124));pa=bytes(model.load(0x15d40e+i,1)for i in range(49))
 expected=bytearray(a);struct.pack_into('<h',expected,0x3f,612);expected[0x5e]=7;struct.pack_into('<h',expected,0x6c,32767);expected[0x72:0x76]=a[7:11];expected[0x60]=(a[0x60]|8)&0xbf
 ep=bytearray(plant);ep[0x2e]|=1
 assert after==expected,[(hex(i),x,y)for i,(x,y)in enumerate(zip(after,expected))if x!=y]
 assert pa==ep;assert r['draws']==0
 cases.append(dict(input_flags=flags,health=1,output_flags=pa[0x2e],output_state=after[0x5e],steps=r['steps'],compared_bytes=173,status='passed_literal_contract'))
result=dict(status='passed_literal_contract',native_comparison='not_run',scope='Synthetic isolated65957..65B48 arrival; no physics/lifecycle/gameplay acceptance',cases=cases,source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k!='source_pins'},indent=2))
