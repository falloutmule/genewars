"""Bounded literal6A2BD hunger/stage tail and5A990 speed parameter checks."""
from pathlib import Path
import importlib.util,struct,json,hashlib
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('m',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
model=m.Source();model.code[0x6a492]=('RET',[]);base=0x1542be;bp=0x2fff00
fixture=P/'private/review/krell-first-tick-001/creatures.bin';record=bytearray(fixture.read_bytes()[:124]);cases=[];steps=0
for skill in [25,30,50,90]:
 percent=30+min(skill,100)*70//100;life=19000*percent//100;reserve=life>>3;threshold=life-reserve;divisor=threshold//3
 for energy in [life,threshold+1,threshold,reserve+1,3]:
  r=record.copy();r[0x76]=skill;struct.pack_into('<h',r,0x66,energy+1);r[0x79]=0
  state={base+i:v for i,v in enumerate(r)};state.update({bp-0x7a+i:v for i,v in enumerate(struct.pack('<I',base))})
  report,_,trace=model.run(entry=0x6a2bd,state_before=state,registers={'EBP':bp},seed=47117)
  after=bytes(model.load(base+i,1)for i in range(124));want=r.copy();struct.pack_into('<h',want,0x66,energy)
  if energy<=threshold:
   want[0x77]=255;stage=m.trunc(energy-reserve,divisor)+1
   if stage!=abs(m.signed(want[0x79],8)):want[0x79]=stage&255
  else:
   hunger=max(16,m.trunc((life-energy)*255,reserve))&255;want[0x77]=hunger
   struct.pack_into('<h',want,0x62,((160*(40+min(skill,100)*60//100)//100)*(hunger+1)>>8)+32)
   struct.pack_into('<h',want,0x64,((80*percent//100)*(hunger+1)>>8)+16)
  assert after==want,(skill,energy,[(i,a,b)for i,(a,b)in enumerate(zip(after,want))if a!=b]);steps+=report['steps'];cases.append({'skill':skill,'energy_after':energy,'hunger':after[0x77],'stage':after[0x79],'life':life,'reserve':reserve,'stage_divisor':divisor})
 for warmth in [0,127,255]:
  r=record.copy();r[0x76]=skill;r[0x7b]=warmth;report,_,_=model.run(entry=0x5a990,state_before={base+i:v for i,v in enumerate(r)},registers={'EAX':base},seed=47117)
  expected=(704*(75+min(skill,100)*25//100)//100)*(warmth+1)>>8;assert model.r['EAX']==expected,(skill,warmth,model.r['EAX'],expected);steps+=report['steps']
paths=[fixture,P/'private/ghidra/player-creation/instructions-0005ed20.txt',P/'private/ghidra/initial-creature/instructions-0006b150.txt',P/'private/ghidra/initial-creature/instructions-0005a990.txt',P/'private/ghidra/initial-creature/instructions-00023820.txt',P/'private/ghidra/initial-creature/instructions-0005d360.txt',P/'private/ghidra/validated/memory-.object3.bin',Path(__file__)]
r={'status':'passed','hunger_cases':20,'speed_cases':12,'literal_instructions':steps,'cases':cases,'sha256':{str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},'scope':'Numericformulas versusliteral tail/speed; not fullcallbackorproductioncomparison','ports':['entry6A2BD callerstacklocal explicitlybound; stopbefore6A492','5A990 directentry; originalmappedselector4parameters']}
Path(__file__).with_suffix('.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:r[k]for k in ['status','hunger_cases','speed_cases','literal_instructions']}))
