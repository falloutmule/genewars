"""Independent native first-counter plans and scheduler, with linked source witness."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,itertools,json,random,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
WORK=ROOT/'private/review/plant-lifecycle'
def pin(p):b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
BASE=(2,4,4,0,7,1,0,0,29,3,0,3,7,4,0,1,40,-1,1,0)
def model(v):
 active,flags,owner,anim,frame,growth,f2,counter,oldret,delay,neighbors,crowd,threshold,period,delta,cell,count,owncount,gate,cached=v
 if active!=2 or flags&1 or f2&0x9a or counter or growth<=0 or not cell or not threshold or period<=1 or owner>4 or owner<4 and owncount<0:return None
 owncount=owncount if owner<4 else -1;updated=0 if neighbors>crowd else (growth+delta+32768)%65536-32768;updated=min(32,updated)
 if updated>0:
  newanim=max(anim,min(updated//8,2));return [0,2,flags|(4 if newanim!=anim else 0),owner,newanim,frame,updated,f2,1,oldret,cell,count,owncount,0,0,0,0,0]
 anim=(anim+6)%256
 if delay>=4:return [1,2,flags|4,owner,anim,frame,0,f2|128,1,delay-4,cell,count,owncount,0,0,0,0,0]
 return [2,0,flags|4,owner,anim,frame,0,f2|128,1,delay,cell+2 if cell<64 else cell,(count-1)%65536,(owncount-1)%65536 if owner<4 else -1,0,1,int(bool(gate)),int(bool(gate and cached)),1]
def cursor(c):return (c+9)%8 if c+8>512 else c+8
def schedule(c,count,slots,post,autofill=False):
 if c>512 or count>512 or len(slots)>512:return None
 c=cursor(c);remaining=count;calls=[];queued=[];created=0;used=[];resolved=[]
 for i,(active,flags,f2) in enumerate(slots):
  if not remaining:break
  if active not in [0,2]:return None
  if not active or flags&1:continue
  if len(calls)<2 and i==c:
   calls.append(i);matches=[s for j,s in post if j==i]
   if not matches:
    if not autofill:return None
    matches=[(2 if i%3 else 0,(i*17)%256,(i*29)%256)]
   if len(matches)!=1:return None
   active,f2,spawn=matches[0]
   if active not in [0,2]:return None
   resolved.append((i,(active,f2,spawn)));used.append(i);created+=spawn;c=cursor(c)
  if active and not f2&16:queued.append(i)
  remaining-=1
 if remaining or any(i not in used for i,s in post):return None
 return (c,count,created,int(created!=0),calls,queued,resolved)
def main():
 WORK.mkdir(parents=True,exist_ok=True);source=ROOT/'rust/crates/genewars-core/src/plant_lifecycle.rs';sourcepin=pin(source);proof_path=ROOT/'research/ghidra/plant-startup-evidence.json';proof=json.loads(proof_path.read_bytes());pins=[]
 for r in proof['instruction_contracts']:p=ROOT/r['private_path'];actual=pin(p);assert actual['sha256']==r['sha256'];pins.append(actual)
 actual_path=ROOT/'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ001.DAT';data=actual_path.read_bytes();assert pin(actual_path)['sha256']==proof['source_fixture']['source_sha256'];fixture=ROOT/proof['source_fixture']['private_path'];assert pin(fixture)['sha256']==proof['source_fixture']['fixture_sha256']
 layouts=[(243038,300,116),(277838,512,41),(298830,20,140),(301630,256,59),(316734,200,73),(331334,32,41),(332646,32,89),(335494,16,94)];records={(family+1,i):bytearray(data[off+i*stride:off+(i+1)*stride]) for family,(off,count,stride) in enumerate(layouts) for i in range(count)};region=bytearray(data[0x355e:0x3b55e]);plants={i:r for (family,i),r in records.items() if family==2 and r[6]};assert sorted(plants)==list(range(40))
 def ref(b,at):index=struct.unpack_from('<h',b,at)[0];family=b[at+2];return None if index<0 or not family else(family,index)
 def cell(x,y):return((y%128)*128+x%128)*14
 def membership():
  found=[]
  for y in range(128):
   for x in range(128):
    key=ref(region,cell(x,y)+6);seen=set()
    while key is not None:
     assert key not in seen;seen.add(key);r=records[key]
     if key[0]==2 and r[6]:found.append(key[1]);assert ((struct.unpack_from('<h',r,7)[0]//256)%128,(struct.unpack_from('<h',r,9)[0]//256)%128)==(x,y)
     key=ref(r,3)
  return sorted(found)
 assert membership()==list(range(40));memory=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes();actual_cases=[];actual_metadata=[];plantcount=40
 for index in [8,16]:
  r=plants[index];x,y=struct.unpack_from('<hh',r,7);gx,gy=x//256%128,y//256%128
  nearby=[i for i,q in plants.items() if q[6] and struct.unpack_from('<h',q,36)[0]>0 and all(min((a-b)%128,(b-a)%128)<=1 for a,b in zip((gx,gy),(struct.unpack_from('<h',q,7)[0]//256%128,struct.unpack_from('<h',q,9)[0]//256%128)))]
  t=region[cell(gx+(x%256>=128),gy+(y%256>=128))+5]&7
  if not t:t=int(any(region[cell(gx+dx,gy+dy)+5]&7 for dx,dy in itertools.product([-1,0,1],repeat=2)))
  slot=struct.unpack_from('<b',r,35)[0];params=memory[0x4b80+slot*32:0x4ba0+slot*32];delta=struct.unpack_from('<b',params,16+t)[0]
  v=(r[6],r[21],r[24],r[25],r[27],struct.unpack_from('<h',r,36)[0],r[38],r[39],r[40],params[29],len(nearby)%256,params[26],params[27],params[24],delta,region[cell(gx,gy)+13],plantcount,-1,0,0);result=model(v);assert result and result[0]==2;actual_cases.append(v);actual_metadata.append(dict(record=index,neighbors=len(nearby),terrain_class=t,growth_delta=delta,plan=result))
  previous,next=ref(r,0),ref(r,3)
  if next is not None:records[next][:3]=r[:3]
  if previous is not None:records[previous][3:6]=r[3:6]
  else:region[cell(gx,gy)+6:cell(gx,gy)+9]=r[3:6]
  r[6]=0;struct.pack_into('<h',r,36,0);r[38]=result[7];region[cell(gx,gy)+13]=result[10];plantcount=result[11]
 assert membership()==[i for i in range(40) if i not in [8,16]] and plantcount==38
 assert [r['neighbors'] for r in actual_metadata]==[7,5]
 def updates():
  for growth in range(1,32768):
   for delta in [-128,-127,-1,0,1,2,126,127]:
    v=list(BASE);v[1]=growth*2%256;v[3]=growth*13%256;v[5]=growth;v[6]=growth&0x65;v[8]=growth%256;v[14]=delta;yield tuple(v)
  for delay,owner,cellvalue in itertools.product(range(256),range(5),range(1,256)):
   v=list(BASE);v[2]=owner;v[3]=delay;v[9]=delay;v[14]=-1;v[15]=cellvalue;v[16]=(cellvalue*257+delay)%65536;v[17]=-1 if owner==4 else(delay*257+cellvalue)%65536;v[18]=delay%2;v[19]=cellvalue%2;yield tuple(v)
  for neighbors,crowd in itertools.product(range(256),repeat=2):v=list(BASE);v[5]=9;v[10]=neighbors;v[11]=crowd;yield tuple(v)
  for count in range(65536):
   for owner in [0,4]:v=list(BASE);v[2]=owner;v[14]=-1;v[16]=count;v[17]=count if owner==0 else -1;yield tuple(v)
  for index,values in [(0,range(256)),(1,range(256)),(2,range(256)),(6,range(256)),(7,range(256)),(8,range(256)),(12,range(256)),(13,range(256)),(15,range(256)),(5,range(-32768,1))]:
   for value in values:v=list(BASE);v[index]=value;yield tuple(v)
  for owner in range(5):v=list(BASE);v[2]=owner;v[17]=-1;yield tuple(v)
  yield from actual_cases
 with (WORK/'updates.bin').open('wb') as stream:
  for v in updates():stream.write(struct.pack('<20i',*v))
 schedules=[]
 for c in range(65536):schedules.append((c,0,[],[]))
 for c in range(513):
  for mode in range(8):
   slots=[(0 if(i+mode)%7==0 else 2,1 if(i+mode)%11==0 else 4,16 if(i+mode)%13==0 else 0) for i in range(512)];count=sum(bool(a) and not f&1 for a,f,_ in slots);result=schedule(c,count,slots,[],True);assert result;schedules.append((c,count,slots,result[6]))
 schedules.extend([(0,40,[(2,4,0)]*40,[(8,(0,128,0)),(16,(0,128,0))]),(0,41,[(2,4,0)]*40,[(8,(2,0,0)),(16,(2,0,0))]),(0,1,[(3,0,0)],[]),(0,0,[(2,0,0)]*513,[]),(0,513,[],[]),(0,40,[(2,4,0)]*40,[]),(0,40,[(2,4,0)]*40,[(8,(2,0,0)),(8,(2,0,0)),(16,(2,0,0))]),(0,0,[],[(65535,(2,0,0))])])
 rng=random.Random(0x6d0b0)
 for trial in range(4096):
  n=rng.randrange(513);slots=[(rng.choice([0,2]),rng.randrange(256),rng.randrange(256)) for i in range(n)];eligible=sum(bool(a) and not f&1 for a,f,_ in slots);count=rng.randrange(eligible+1);c=rng.randrange(513);result=schedule(c,count,slots,[],True);assert result;post=result[6]
  if trial%8==0:post=post+[(65535,(2,0,0))]
  schedules.append((c,count,slots,post))
 with (WORK/'schedules.bin').open('wb') as stream:
  for c,n,slots,post in schedules:
   stream.write(struct.pack('<4H',c,n,len(slots),len(post)));stream.write(bytes(v for s in slots for v in s))
   for i,s in post:stream.write(struct.pack('<H3B',i,*s))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=WORK/'review.exe';r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace');r=subprocess.run([str(exe),str(WORK)],capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace')
 raw=(WORK/'results.bin').read_bytes();at=0;checked=errors=0;phases=[0]*3
 for v in updates():
  expected=model(v);status=raw[at];at+=1;checked+=1
  if expected is None:assert status==1,(checked,v);errors+=1
  else:assert status==0,(checked,v);observed=list(struct.unpack_from('<18i',raw,at));at+=72;assert observed==expected,(checked,v,observed,expected);phases[observed[0]]+=1
 schedules_bad=0;callbacks=queued=0
 for number,(c,n,slots,post) in enumerate(schedules):
  expected=schedule(c,n,slots,post);status=raw[at];at+=1
  if expected is None:assert status==1,number;schedules_bad+=1;continue
  assert status==0,number;head=struct.unpack_from('<6H',raw,at);at+=12;observed_calls=list(struct.unpack_from('<'+'H'*head[4],raw,at));at+=head[4]*2;observed_queue=list(struct.unpack_from('<'+'H'*head[5],raw,at));at+=head[5]*2;assert(head[:4],observed_calls,observed_queue)==(expected[:4],expected[4],expected[5]),number;callbacks+=head[4];queued+=head[5]
 assert at==len(raw) and pin(source)==sourcepin
 report=dict(status='passed-independent-native-plant-lifecycle-review',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,source=sourcepin,evidence=pin(proof_path),instruction_pins=pins,update_cases=checked,rejected_updates=errors,phase_counts=dict(zip(['grow','delayed_retirement','retired'],phases)),schedule_cases=len(schedules),rejected_schedules=schedules_bad,callback_requests=callbacks,queued_index_receipts=queued,actual_loaded_plants=40,actual_remaining_plants=38,actual_metadata=actual_metadata,source_mission=pin(actual_path),fixture=pin(fixture),native_output_sha256=hashlib.sha256(raw).hexdigest(),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],native_harness=pin(exe),resolved_findings=['Growmustpreserveoldnative30; fixedbyexplicitoldcountdowninput/regression','Solsourcefixtureduplicatehashkey fixedsource_sha256/fixture_sha256'],limits=['Explicit freshcounter/flags/neighbors/resourceparameters only; no wholemissiontick','Otherfamilies/actorcreation/shade-refresh finalmap state external','Cleanupmetadata are requests, not executedworld/UI/audio changes','Oldframecurrentview gate suppliedexplicitly; original narrowsprojectionwords beforebounds','Creationsecondpass pendingwhencreatedcountnonzero; nospawning/RNGimplementation'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:report[k] for k in ['status','update_cases','schedule_cases','actual_remaining_plants']}))
if __name__=='__main__':main()
