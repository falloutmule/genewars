"""Independent literal branch/slot oracle; no original execution or live-state inference."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, itertools, json, random, struct, subprocess, sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
WORK=ROOT/'private/review/player-creation'
def pin(path):
 b=path.read_bytes();return dict(path=path.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def signed(v):return v if v<128 else v-256
def gate(category,mask,progress,campaign):
 # Literal jump-table branch predicates; quota bits are private to selection.
 if category==0:return bool(mask&1)
 if category==1:return bool(mask&2)and(progress>=1 or not campaign)
 if category==2:return bool(mask&4)
 if category==3:return bool(mask&8)and(progress>=4 or not campaign)
 return False
def oracle(b):
 mask,rawprogress,campaign,player,rawcount=b[:5];progress=signed(rawprogress);count=signed(rawcount);slots=list(b[5:10]);out=bytearray(64)
 if player>3:out[0]=1;return out
 if not 0<=count<=5:out[0]=2;return out
 # Model original CH/CL quotas rather than an implementation category-bitset.
 ch=cl=attempts=created=0
 for index in range(32):
  active,category=b[10+index*2:12+index*2]
  if not active or not gate(category,mask,progress,bool(campaign)):continue
  if category==0:
   if ch>=1:continue
   ch+=1
  else:
   bit={1:2,2:8,3:4}[category]
   if cl&bit:continue
   cl|=bit
  at=10+13*attempts;out[at:at+3]=bytes((1,index,category));attempts+=1
  # Address-order scan through exactly five 200-byte native records.
  pointer=0
  while pointer<1000 and slots[pointer//200]!=0:pointer+=200
  if pointer==1000:continue
  if count>=5:return bytearray([3])+bytearray(63)
  local=pointer//200;global_id=(1000*player+pointer)//200
  out[at+3:at+13]=bytes((1,local,global_id,player,3,4+count,1,index,0x80,2))
  slots[local]=3;count+=1;created+=1
 out[1:10]=bytes((attempts,created,count,*slots,player));return out
def main():
 WORK.mkdir(parents=True,exist_ok=True)
 source=ROOT/'rust/crates/genewars-core/src/player_creation.rs';evidence=ROOT/'research/ghidra/player-creation-evidence.json';sourcepin=pin(source);evidencepin=pin(evidence);proof=json.loads(evidence.read_bytes())
 instructionpins=[]
 for p in proof['instruction_contracts']:
  actual=pin(ROOT/p['private_path']);assert actual['sha256']==p['sha256'];instructionpins.append(actual)
 # Bind numeric branches and partition/address arithmetic to actual listings.
 checks={
 'mission-target-refs/instructions-000801c0.txt':['000801fb INC CH','000801fd CALL 0x00081310','00080237 OR CL,0x2','00080269 OR CL,0x4','00080286 OR CL,0x8','0008021d MOVSX EDX,byte ptr [EBX + 0x6e7]','000802be MOV dword ptr [EBX + 0x9],EDX'],
 'mission-array-refs/instructions-00081300.txt':['000802f9 ADD ECX,0xc8','00080304 IMUL EAX,EAX,0x3e8','00080318 CMP byte ptr [ECX + 0x6],0x0','00080323 OR byte ptr [EBX + 0x20],0x1','0008033e MOVSX EDX,AL','00080349 MOV byte ptr [EDX + ESI*0x1 + 0x4],AL'],
 'player-creation/instructions-0004fd30.txt':['0004fd7b MOV byte ptr [ECX + 0x6],0x3','0004fd83 MOV byte ptr [ECX + 0x18],AL','0004fd88 MOV word ptr [ECX + 0xf],BX','0004fef4 MOV word ptr [ECX + 0x15],0x280','0004ff0e MOV word ptr [ECX + 0x70],0x0'],
 'player-creation/instructions-00057a10.txt':['00057a1e TEST byte ptr [EAX + 0x70],0x1','00057a24 JMP 0x0005876f','0005876f XOR AL,AL']}
 for name,lines in checks.items():
  text=(ROOT/'private/ghidra'/name).read_text();assert all(line in text for line in lines),name
 code=(ROOT/'private/ghidra/validated/memory-.object1.bin').read_bytes();assert struct.unpack_from('<4I',code,0x801a4-0x10000)==(0x801dc,0x80207,0x80273,0x80241)
 gates=bytearray();expectedgates=bytearray()
 for cat,mask,raw,campaign in itertools.product(range(4),range(256),range(256),range(2)):
  gates.extend((cat,mask,raw,campaign));expectedgates.append(gate(cat,mask,signed(raw),bool(campaign)))
 for cat,mask,campaign in itertools.product(range(4,256),range(256),range(2)):
  gates.extend((cat,mask,127 if campaign else 128,campaign));expectedgates.append(0)
 (WORK/'gates.bin').write_bytes(gates);categories={};count=0
 def record(categories,actives=None):return bytes(v for i,c in enumerate(categories)for v in ((actives[i]if actives else 1),c))+bytes(64-2*len(categories))
 generated=record([i//5 for i in range(20)])
 def request(records=generated,mask=15,progress=4,campaign=1,player=0,slots=(0,0,0,0,0),stored=0):return bytes((mask,progress&255,campaign,player,stored&255,*slots))+records
 expected=hashlib.sha256();rejected=successful=attemptsum=createdsum=0
 def batch(name,items):
  nonlocal count,rejected,successful,attemptsum,createdsum
  first=count
  for item in items:
   assert len(item)==74;plans.write(item);answer=oracle(item);answers.write(answer);expected.update(answer);count+=1
   if answer[0]:rejected+=1
   else:successful+=1;attemptsum+=answer[1];createdsum+=answer[2]
  categories[name]=count-first
 with (WORK/'plans.bin').open('wb')as plans,(WORK/'expected.bin').open('wb')as answers:
  batch('all_active_and_category_bytes',(request(record([cat],[active]))for active,cat in itertools.product(range(256),repeat=2)))
  batch('all_slot_occupancy_player_blueprint_positions',(request(record([0]*index+[2]),player=player,slots=tuple((17+i if bits>>i&1 else 0)for i in range(5)),stored=bits.bit_count())for bits,player,index in itertools.product(range(32),range(4),range(32))))
  batch('all_slot_patterns_and_signed_count_bytes',(request(slots=tuple((255 if bits>>i&1 else 0)for i in range(5)),stored=signed(c))for bits,c in itertools.product(range(32),range(256))))
  batch('all_player_bytes_error_precedence',(request(player=player,stored=count)for player,count in itertools.product(range(256),[-128,-1,0,5,6,127])))
  batch('every_mask_progress_boundary_campaign',(request(mask=mask,progress=progress,campaign=campaign)for mask,progress,campaign in itertools.product(range(256),[-128,-1,0,1,3,4,5,127],range(2))))
  batch('category_order_duplicate_quota_and_holes',(request(record(list(order)*8),player=player,slots=tuple(7 if bits>>i&1 else 0 for i in range(5)),stored=stored)for order,bits,player,stored in itertools.product(itertools.permutations(range(4)),range(32),range(4),range(6))))
  batch('one_free_slot_failure_consumes_later_quotas',(request(record([0,0,1,1,2,2,3,3]),slots=tuple(0 if i==free else 250 for i in range(5)),stored=stored)for free,stored in itertools.product(range(5),range(6))))
  batch('all_occupied_bytes_preserved',(request(slots=(v,v,v,v,v),stored=5)for v in range(1,256)))
  batch('empty_scan_target_reset',(request(bytes(64),player=player,stored=count,slots=(0,3,255,0,7))for player,count in itertools.product(range(4),range(6))))
  rng=random.Random(0x801c081310)
  batch('random_raw_blueprint_scan',(request(bytes(rng.randrange(256)for _ in range(64)),mask=rng.randrange(256),progress=rng.randrange(-128,128),campaign=rng.randrange(2),player=rng.randrange(4),slots=tuple(rng.choice([0,0,1,3,255])for _ in range(5)),stored=rng.randrange(6))for _ in range(10000)))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=WORK/'review.exe'
 build=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert build.returncode==0,build.stderr.decode(errors='replace')
 run=subprocess.run([str(exe),str(WORK)],capture_output=True,timeout=90);assert run.returncode==0,run.stderr.decode(errors='replace')
 with (WORK/'results.bin').open('rb')as actual,(WORK/'expected.bin').open('rb')as answers:
  assert actual.read(len(expectedgates))==expectedgates
  assert actual.read(256)==bytes(flag%2==0 for flag in range(256))
  digest=hashlib.sha256()
  for i in range(count):
   a=actual.read(64);e=answers.read(64);assert a==e,(i,a.hex(),e.hex());digest.update(a)
  assert not actual.read(1)and not answers.read(1);assert digest.hexdigest()==expected.hexdigest()
 assert pin(source)==sourcepin and pin(evidence)==evidencepin
 mission=ROOT/'private/ghidra/campaign-resources/152.dat';assert pin(mission)['sha256']==proof['source']['sha256'];assert mission.read_bytes()[0x1ebd]==5
 example=oracle(request(mask=5,progress=0));assert example[1:4]==bytes((2,2,2))and example[11]==0 and example[24]==10
 report=dict(status='passed-independent-native-player-creation-review',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,source=sourcepin,evidence=evidencepin,instruction_pins=instructionpins,jump_table_sha256=hashlib.sha256(code[0x701a4:0x701b4]).hexdigest(),source_mission=pin(mission),category_gate_cases=len(expectedgates),ordinary_update_gate_cases=256,plan_cases=count,categories=categories,successful_plans=successful,rejected_plans=rejected,attempt_receipts=attemptsum,allocation_receipts=createdsum,native_plan_output_sha256=digest.hexdigest(),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],native_harness=pin(exe),limits=['Explicit metadata, slot, count, player and campaign inputs; no generated random blueprint or retained live-player inference','Count0..5 and per-success append capacity enforced; count is not required to equal occupied slots','Requests describe selected writes only; initializer resource pointers, placement, reveal, audio and target-table copy remain caller operations','Clear flags70bit0 suppression only; family0 AI/playerbookkeeping/fullmissiontick unknown'])
 out=Path(__file__).with_suffix('.json');temp=out.with_suffix('.json.tmp');temp.write_text(json.dumps(report,indent=2)+'\n');temp.replace(out)
 print(json.dumps({k:report[k]for k in ['status','category_gate_cases','plan_cases','attempt_receipts','allocation_receipts']}))
if __name__=='__main__':main()
