"""Pinned Krell contract and literal neutral-zone credit primitive; no game run."""
from pathlib import Path
import hashlib,importlib.util,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra';D=P/'private/reference-run/run-006/drive-c/GAME'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest=json.loads((P/'research/game-files-manifest.json').read_bytes())['entries']
assets={}
for entry_id,name in [('root/2/disc/153','CONQ002.DAT'),('root/2/disc/201','HDRCO2.DAT')]:
 entry=next(e for e in manifest if e['entry_id']==entry_id);p=D/'LEVELS'/name;assert sha(p)==entry['sha256'];assets[name]={'entry_id':entry_id,'sha256':sha(p),'size':p.stat().st_size}
b=(D/'LEVELS/CONQ002.DAT').read_bytes();h=(D/'LEVELS/HDRCO2.DAT').read_bytes();obj=(G/'validated/memory-.object3.bin').read_bytes();code=(G/'validated/memory-.object1.bin').read_bytes();text=(D/'DATA/TEXT.DAT').read_bytes().split(b'\0')
assert h[32:h.index(0,32)]==b'Krell' and len(b)==343958
assert b[0x34b8]==1 and b[0x34b9]==0 and b[0x1ebd]==7 and b[0x52482]==1
zone=b[0x2ff8:0x300b];assert struct.unpack_from('<hh',zone)==(256,24320) and zone[4]==12 and zone[5]==255
assert [struct.unpack_from('<i',zone,o)[0]for o in [6,10,14]]==[1024]*3
assert struct.unpack_from('<H',b,0x1ea6)[0]==0 and struct.unpack_from('<i',b,0x1eaa)[0]==1000
assert obj[0x4a64+8:0x4a64+10]==bytes([4,4]);assert struct.unpack_from('<I',code,0x7a260-0x10000+5*4)[0]==0x7a46a
assert text[139]==b'Genepod' and obj[0x4d88]==1 and obj[0x4e74]==3 and struct.unpack_from('<i',obj,0x4e1c)[0]==600
briefing_index=(0x1165a0-0x115e90)//4+1;briefing=text[briefing_index]
assert all(t in briefing for t in [b'Gene Pod',b'Mules',b'marked zone'])
files=[G/'mission-zone-792c0.txt',G/'mission-zone-score.txt']
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m);x=m.Source();x.code.update(m.m.Model(files,b'').code);order=sorted(x.code);x.next=dict(zip(order,order[1:]));cases=[]
for owner in [0,4]:
 for remaining in [0,1,4,8,16,1024]:
  for weight in [8,16]:
   state={0x11878c:1,0x11873d:0};state.update({0x11bd58+i:v for i,v in enumerate(zone)})
   for at,value in [(0x11bd58+14,remaining),(0x118d74+13,0),(0x118d74+17,0)]:state.update({at+i:v for i,v in enumerate(struct.pack('<i',value))})
   # Null creature is an explicit numeric-credit fixture: audio/marker calls
   # in80780 are outside this primitive proof. Actual793C0 supplies a creature.
   r,_,_=x.run(state_before=state,entry=0x792c0,registers={'EAX':0x11bd58,'EDX':weight,'EBX':owner,'ECX':0})
   credit=min(weight,remaining)>>(2 if owner==4 else 0)
   assert x.load(0x11bd58+14,4)==remaining-credit and x.load(0x118d74+13,4)==credit
   assert bool(x.r['EAX']&255)==(credit>0)
   cases.append({'owner':owner,'remaining':remaining,'weight':weight,'credit':credit,'instructions':r['steps']})
pins={f.relative_to(P).as_posix():sha(f)for f in files+[G/'mission-array-refs/instructions-000793c0.txt',G/'mission-array-refs/instructions-0007a280.txt',G/'font-menu-state/instructions-00072ed0.txt',G/'campaign-level/instructions-0007d090.txt',G/'mission-target-refs/instructions-000801c0.txt']}
groups=[(1,243038,300,116),(2,277838,512,41),(3,298830,20,140),(4,301630,256,59),(5,316734,200,73),(6,331334,32,41),(7,332646,32,89),(8,335494,16,94)]
r={'status':'passed-static-contract-and-zone-credit-primitive','reference_executable_sha256':'4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba','mission':{'progress':1,'number':2,'title':'Krell','assets':assets},'briefing':{'text_index':briefing_index,'sha256':hashlib.sha256(briefing).hexdigest(),'paraphrase':'Build a Gene Pod, create Mules and move them into the marked zone.'},'initial_data':{'players':1,'funds_before_engineer_costs':b[0x1c5c]*100,'target':struct.unpack_from('<hh',b,0x1c7e),'actor_mask':7,'serialized_seed':struct.unpack_from('<H',b,0x52480)[0],'serialized_draw_count':struct.unpack_from('<I',b,0x52478)[0],'active_counts':{str(k):sum(b[o+i*t+6]!=0 for i in range(n))for k,o,n,t in groups}},'zone':{'center':[256,24320],'radius':1024,'selector':12,'species':4,'initial_quota':1024,'scan_every_frames':64,'requires_countdown_zero':True,'mule_primary_credit':16,'mule_secondary_credit':8,'neutral_credit_divisor':4,'credit_is_capped_by_remaining_quota':True},'victory':{'predicate':'Eligible selected-player score at+d >=1000 while terrain_count nonzero; otherwise existing exhausted-zone highest-score fallback may apply. Full72ED0 eligibility/failure rules still apply.','target_count':0},'new_building':{'selector':0,'name':'Genepod','progress_threshold':1,'requires_completed_selector':3,'base_cost':600},'continuity':'7D090 skips7F580 at progress1 when carried blueprint count is nonzero. Actor mask7 selects the previously gated category at progress>=1. Do not reuse fresh-first-mission regeneration or force carried blueprint state.','numeric_credit_cases':cases,'listing_sha256':pins,'unknowns':['Whole793C0 zone callback including effects/audio and creature state gates needs owned-world proof.','Gene Pod production, creature ownership/control and research requirements need source-backed integration.','Prior-world cleanup and carried-state restoration must compose with the independently recovered result transition.','Cross-variant campaign equivalence remains unestablished.'],'acceptance':{'automated':False,'native_interactive':False,'campaign_transition':False},'original_execution':False}
r['variant_comparison']={e['entry_id']:{'size':e['size'],'sha256':e['sha256']}for e in manifest if e['entry_id']in ['root/1/80','root/1/117','root/4/221','root/4/276']}
assert r['variant_comparison']['root/1/80']['sha256']==assets['HDRCO2.DAT']['sha256'] and r['variant_comparison']['root/1/117']['sha256']==assets['CONQ002.DAT']['sha256']
assert r['variant_comparison']['root/4/221']['sha256']!=assets['CONQ002.DAT']['sha256']
(P/'research/campaign/mission-002-contract.json').write_text(json.dumps(r,indent=2)+'\n');print('PASS Krell contract;24 zone-credit fixtures; no mission acceptance')
