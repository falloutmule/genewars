"""Stateful no-RNG callback branches; reproduction is an explicit boundary."""
from pathlib import Path
import hashlib,json
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra';sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
pins=[]
def pin(name,*needles):
 b=(G/name).read_bytes()
 for needle in needles:assert needle in b.decode(),(name,needle)
 pins.append({'private_path':'private/ghidra/'+name,'sha256':sha(b)})
pin('entity-order/instructions-0006ca90.txt','0006caa1 TEST byte ptr [EAX + 0x15],0x1','0006caa7 TEST byte ptr [EDI + 0x2e],0x10','0006cad6 JLE 0x0006d01d','0006cb26 MOV AL,byte ptr [EDI + 0x2f]','0006cb2d INC AL','0006cb37 CMP EAX,EDX','0006cb39 JLE 0x0006cfd8','0006cb4a JLE 0x0006cfd8','0006cc1e MOV word ptr [EDI + 0x2c],0x0','0006ccaa IDIV EBX','0006ccae JNZ 0x0006cfe9','0006ccca CMP EAX,0x200','0006cceb CMP EAX,0x19c','0006cfdc SUB EDX,0x4','0006d043 TEST byte ptr [EDI + 0x2e],0x2')
pin('display-geometry/instructions-0006c780.txt','0006c799 TEST DL,0x88','0006c79c JNZ 0x0006c7b7','0006c7a8 MOV word ptr [ESI + 0x2c],0x0','0006c7b1 MOV byte ptr [ESI + 0x30],AL','0006c7bf JL 0x0006c7ce','0006c7c3 SUB CH,0x4','0006c8d0 ADD CL,0x2','0006c8dd CALL 0x000724c0','0006c8e2 MOV byte ptr [ESI + 0x6],0x0')
def step(state,check=8,period=4,crowding=3,neighbors=0,delta=1,delay=3,cell13=1,plant_count=40):
 s=dict(state);requests=[];before=s['animation']
 assert s['active_type']==2 and 0<=s['owner']<=4 and -32768<=s['growth']<=32767
 assert all(0<=s[k]<=255 for k in ['flags15','flags2e','counter','retirement','animation'])
 assert all(0<=v<=255 for v in [check,period,crowding,neighbors,delay,cell13]) and -128<=delta<=127 and 0<=plant_count<=512
 if s['flags15']&1 or s['flags2e']&16:return {'state':s,'phase':'suppressed','requests':requests,'created_count':0}
 phase='existing-nonpositive'
 if s['growth']>0:
  old=s['counter'];s['counter']=(old+1)&255
  if old>=check or cell13==0:
   s['growth']=((s['growth']-4+32768)&65535)-32768;phase='subtract-four'
  elif neighbors>crowding:
   s['growth']=0;phase='crowded'
  else:
   s['growth']=min(((s['growth']+delta+32768)&65535)-32768,32);phase='delta'
   if not period:raise ValueError('zero-period-source-divide-fault-safely-rejected')
   if s['counter']%period==0 and plant_count<(412 if s['owner']==4 else 512):
    raise ValueError('reproduction-requires-RNG-target-allocation-contract')
  s['animation']=max(s['animation'],min(s['growth']>>3,2))
 if s['growth']<=0:
  if not s['flags2e']&0x88:
   s['animation']=(s['animation']+6)&255;s['growth']=0;s['flags2e']|=128;s['retirement']=delay
  if s['retirement']>=4:
   s['retirement']-=4;phase+=':delayed-retirement'
  else:
   s['active_type']=0;phase+=':retire'
   requests=['unlink-cell-list','old-frame-dirty-check','724C0-reference/UI/audio-cleanup']
   requests.append({'plant_count_delta':-1,'cell13_after':cell13+2 if cell13<64 else cell13,'owner_count_decrement':s['owner'] if s['owner']<4 else None})
 if s['animation']!=before or s['flags2e']&2:s['flags15']|=4
 return {'state':s,'phase':phase,'requests':requests,'created_count':0}

base={'active_type':2,'owner':4,'growth':22,'flags15':4,'flags2e':0,'counter':0,'retirement':77,'animation':0}
vectors=[]
cases=[({'counter':1},{}),({'counter':7},{'period':3}),({'counter':8},{}),({'counter':255},{}),({'counter':1},{'cell13':0}),({'counter':8,'growth':3},{'delay':8}),({'growth':0,'flags2e':128,'retirement':8},{}),({'growth':0,'flags2e':128,'retirement':4},{}),({'growth':-3,'flags2e':128,'retirement':3,'owner':0},{'cell13':63}),({'growth':0,'flags2e':8,'retirement':3},{}),({'counter':1,'flags2e':2},{'period':4}),({'flags15':5},{}),({'flags2e':16},{}),({'counter':3},{'plant_count':412}),({'counter':3,'owner':0},{'plant_count':512}),({'counter':2},{'neighbors':4,'period':0})]
for patch,kwargs in cases:
 s=dict(base,**patch);vectors.append({'state':s,'parameters':kwargs,'result':step(s,**kwargs)})
gate_cases=0
for old in range(256):
 for threshold in range(256):
  for cell13 in [0,1]:
   expected=(old>=threshold or cell13==0)
   # Check old-versus-new distinction and wrap independently of lifecycle.
   dl=old;al=(old+1)&255;ecx=threshold
   branch=(ecx<=dl) or not cell13
   assert expected==branch and al==(old+1)%256;gate_cases+=1
retirement_cases=0
for delay in range(256):
 s=dict(base,growth=0,flags2e=0,retirement=99);calls=0
 while s['active_type']:
  out=step(s,delay=delay);s=out['state'];calls+=1
  assert calls<=64
 assert calls==delay//4+1;retirement_cases+=1
 # Existing88 flags do not recopy delay or add another animation6.
 for flags in [8,128,136]:
  s=dict(base,growth=-3,flags2e=flags,retirement=delay,animation=7)
  out=step(s,delay=(delay+1)&255)
  assert out['state']['growth']==-3 and out['state']['animation']==7
  assert out['state']['retirement']==(delay-4 if delay>=4 else delay)
  retirement_cases+=1
for kwargs,message in [({'period':0},'zero-period'),({'period':1},'reproduction')]:
 try:step(base,**kwargs)
 except ValueError as e:assert message in str(e)
 else:raise AssertionError('unsupported branch accepted')
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,
 'scope':'Complete entity-field plan only for explicitly admitted no-RNG callback paths; not whole growth/reproduction/world tick.',
 'input':'Active numeric-family1 type2; owner0..4; signed16 growth; u8 animation/flags15/flags2E/counter2F/retirement30; explicit parameter bytes, nine-cell neighbor sum, terrain delta, cell13 and liveplantcount0..512.',
 'early_suppression':'flags15mask01 orflags2Emask10 ->AL0 and noentityfieldchanges; no finaldirtyflag logic.',
 'counter':'Only positive starting growth increments counter2F withbytewrap. Check OLDcounter unsigned againstparameter1B. Old>=threshold orcell13==0 goesdirectsignedwordsubtract4 andskipsneighbors/period/RNG. Negative/zero startinggrowth preservescounter.',
 'checked_growth':'Whenold<threshold andcell13>0, neighborbyte>crowdlimit1A sets growth0 and skipsperiod/RNG. Otherwise addsignedterrain delta withi16wrap/capupper32, then tests NEWcounter modulo unsignedperiod18. Period0 is originaldividefault and saferejected.',
 'reproduction_boundary':'IfperioddividesNEWcounter and liveplantcount <412 forowner4 or<512 forowner0..3, thisplan rejects asneedsreproduction; do notsilentlysuppressspawn/RNG. At/aboveowner-specificcap, originalskipsRNG andthe parentfieldplan isadmitted. Reproductioncanbeattemptedbeforethe finalnonpositivegrowthretirementcheck.',
 'animation':'Afterpositive-growth branch, stage=min(arithmetic_shift(growth,3),2); animationchanges onlyif stage>unsignedoldanimation. No lowerclamp; negativestagecannotincreaseu8animation. Finalflags15OR04 ifanimationchanged orflags2Emask02set, evenwhenretirementclearsactive.',
 'retirement':'Fornonpositivegrowth,6C780 initializes onlywhen(flags2E&88)==0: adds6toanimationwithu8wrap,growth0,flags2EOR80,retirement30=param1D. Otherwisepreservesanimation/growth/flags/current30, includingnegativegrowth. Ifcurrent30>=4 subtract4 andremainactive; elseclearactive,requestunlink/dirty/724C0,plantcountDECword,cell13+2 onlyifold<64,owner0..3DECwordrequest,owner4none. Numberofcallsfromfreshdelay isfloor(delay/4)+1. Paramdelayread-only.',
 'state_mapping_receipt':'research/ghidra/plant-state-evidence.json','cleanup_receipt':'research/ghidra/plant-startup-evidence.json',
 'vectors':vectors,'old_counter_gate_cases':gate_cases,'retirement_cases':retirement_cases,
 'limits':['No reproduction RNG, targetselection, allocation, transitive cleanup or fullscheduleexecution in thisplan.','Explicitliveworld/map inputs stillmustaccountforprecedingfamilyupdates.','Safeerrorsdo notapplypartialnativewrites; originalwouldalreadyhaveincrementedcounter/growth beforefault/pendingreproduction.']}
(P/'research/ghidra/plant-step-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS2pins;{gate_cases}oldcounter/cellgates;{retirement_cases}retirementcases;{len(vectors)}statefulvectors;explicitreproduction/divideboundaries.')
