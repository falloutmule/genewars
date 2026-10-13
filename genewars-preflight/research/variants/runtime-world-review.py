"""Independent real-source owned world trajectory and headless adapter audit."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,subprocess,sys,uuid
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
WORK=ROOT/'private/review/runtime-world';MAP=0x355e;PLANT=277838;STRIDE=41
LAYOUTS=[(243038,300,116),(PLANT,512,41),(298830,20,140),(301630,256,59),(316734,200,73),(331334,32,41),(332646,32,89),(335494,16,94)]
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def word(b,o):return struct.unpack_from('<h',b,o)[0]
def uword(b,o):return struct.unpack_from('<H',b,o)[0]
def setword(b,o,v):struct.pack_into('<H',b,o,v&65535)
def signed(v):return(v+32768)%65536-32768
def at(i):return PLANT+i*STRIDE
def cell(x,y):return MAP+((y%128)*128+x%128)*14
def position(b,i):return word(b,at(i)+7),word(b,at(i)+9)
def cursor(c):return c+8 if c+8<=512 else(c+9)&7
def terrain(b,x,y):
 gx,gy=x//256%128,y//256%128;rx,ry=gx+(x%256>=128),gy+(y%256>=128);value=b[cell(rx,ry)+5]&7
 return value or int(any(b[cell(gx+dx,gy+dy)+5]&7 for dy in [-1,0,1]for dx in [-1,0,1]))
def ref(raw):
 index=word(raw,0);return None if index<0 or raw[2]==0 else(raw[2],index)
def record(reference):return LAYOUTS[reference[0]-1][0]+reference[1]*LAYOUTS[reference[0]-1][2]
def memberships(b):
 found=[]
 for gy in range(128):
  for gx in range(128):
   link=ref(b[cell(gx,gy)+6:cell(gx,gy)+9]);previous=None;seen=set()
   while link:
    assert link not in seen;seen.add(link);base=record(link);assert ref(b[base:base+3])==previous
    if link[0]==2:assert b[base+6]==2;assert(position(b,link[1])[0]//256%128,position(b,link[1])[1]//256%128)==(gx,gy);found.append(link[1])
    previous=link;link=ref(b[base+3:base+6])
 assert sorted(found)==[i for i in range(512)if b[at(i)+6]];assert len(found)==uword(b,0x3552)
def callback(b,i,parameters):
 start=at(i);f=b[start+21];g=b[start+38]
 if f&1 or g&16:return
 oldanim=anim=b[start+25];owner=b[start+24];growth=word(b,start+36);counter=b[start+39];delay=b[start+40];x,y=position(b,i);gx,gy=x//256%128,y//256%128;c=cell(gx,gy);overlay=b[c+13];row=parameters[b[start+35]*32:(b[start+35]+1)*32]
 assert owner==4
 if growth>0:
  oldcounter=counter;counter=(counter+1)%256
  if oldcounter>=row[27]or overlay==0:growth=signed(growth-4)
  else:
   # Independent geometric count, not the production linked traversal.
   nearby=0
   for j in range(512):
    if not b[at(j)+6]or word(b,at(j)+36)<=0:continue
    xx,yy=position(b,j);dx=(xx//256-gx)%128;dy=(yy//256-gy)%128
    if dx in [0,1,127]and dy in [0,1,127]:nearby=(nearby+1)%256
   if nearby>row[26]:growth=0
   else:
    delta=row[16+terrain(b,x,y)];delta=delta if delta<128 else delta-256;growth=min(signed(growth+delta),32)
    if row[24]==0:raise ValueError('zero period')
    if counter%row[24]==0 and uword(b,0x3552)<412:raise ValueError('reproduction')
  stage=min(growth//8,2)
  if stage>anim:anim=stage
 if growth<=0:
  if not g&0x88:anim=(anim+6)%256;growth=0;g|=128;delay=row[29]
  if delay>=4:delay-=4
  else:
   prevraw=b[start:start+3];nextraw=b[start+3:start+6];previous=ref(prevraw);following=ref(nextraw)
   if following:b[record(following):record(following)+3]=prevraw
   if previous:b[record(previous)+3:record(previous)+6]=nextraw
   elif following:b[c+6:c+9]=nextraw
   else:b[c+8]=0 # 8ADAA singleton: keep index bytes; only clear kind.
   b[start+6]=0;b[c+13]=overlay+2 if overlay<64 else overlay;setword(b,0x3552,uword(b,0x3552)-1)
 if anim!=oldanim or g&2:f|=4
 b[start+21]=f;b[start+25]=anim;setword(b,start+36,growth);b[start+38]=g;b[start+39]=counter;b[start+40]=delay
def step(original,parameters):
 b=bytearray(original);remaining=uword(b,0x3552);c=cursor(uword(b,0x355a));calls=[];queued=[]
 for i in range(512):
  if remaining==0:break
  start=at(i)
  if b[start+6]==0 or b[start+21]&1:continue
  if len(calls)<2 and i==c:
   calls.append(i)
   try:callback(b,i,parameters)
   except ValueError as error:return None,str(error),calls
   c=cursor(c)
  if b[start+6]and not b[start+38]&16:queued.append(i)
  remaining-=1
 assert remaining==0;setword(b,0x355a,c);memberships(b);return b,None,(calls,queued)
def main():
 WORK.mkdir(parents=True,exist_ok=True);sources=[ROOT/'rust/crates/genewars-runtime/src'/name for name in ['world.rs','identity.rs']]+[ROOT/'rust/crates/genewars-desktop/src/bin/support'/name for name in ['runtime_resources.rs','runtime_scene.rs','menu_art.rs']];sourcepins=[pin(p)for p in sources]
 mission=ROOT/'private/integration/game-runtime/terrain/CONQ001.DAT';parameterpath=ROOT/'private/integration/game-runtime/plant-parameters.bin';data=mission.read_bytes();parameters=parameterpath.read_bytes();assert pin(mission)['sha256']=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2';assert pin(parameterpath)['sha256']=='297a1da0e9c30d6d4cb7c88addb8234be260e0e93b7d4b2b1c4c3ad8b7df2b7c';memberships(data)
 (WORK/'source.bin').write_bytes(data);(WORK/'parameters.bin').write_bytes(parameters)
 proofpins=[]
 for name in ['plant-state-evidence.json','plant-step-evidence.json','plant-startup-evidence.json']:
  p=ROOT/'research/ghidra'/name;proofpins.append(pin(p));receipt=json.loads(p.read_bytes())
  for r in receipt['instruction_contracts']:assert pin(ROOT/r['private_path'])['sha256']==r['sha256']
 unlink=ROOT/'private/ghidra/terrain-map-refs/instructions-0008acc0.txt';assert '0008adaa MOV byte ptr [EBX + 0x8],AL'in unlink.read_text()
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=WORK/'review.exe'
 for crate in ['assets','core','runtime']:
  args=[str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(ROOT/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={WORK}','-o',str(WORK/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dependency in ['assets','core']:args+=['--extern',f'genewars_{dependency}={WORK}/libgenewars_{dependency}.rlib']
  result=subprocess.run(args,env=environment(),capture_output=True,timeout=90);assert result.returncode==0,result.stderr.decode(errors='replace')
 args=[str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={WORK}','-o',str(exe)]
 for dependency in ['assets','core','runtime']:args+=['--extern',f'genewars_{dependency}={WORK}/libgenewars_{dependency}.rlib']
 result=subprocess.run(args,env=environment(),capture_output=True,timeout=90);assert result.returncode==0,result.stderr.decode(errors='replace');result=subprocess.run([str(exe),str(WORK)],capture_output=True,timeout=90);assert result.returncode==0,result.stderr.decode(errors='replace')
 states=[data];trajectory=[];blocked=None;callback_total=0
 with (WORK/'states.bin').open('rb')as native:
  for tick in range(1,2049):
   expected,reason,details=step(states[-1],parameters);status=native.read(1)[0]
   if reason:
    assert status==1;length=struct.unpack('<I',native.read(4))[0];error=native.read(length).decode();assert native.read(len(data))==states[-1];assert reason=='reproduction'and 'reproduction'in error;blocked=dict(attempted_tick=tick,reason=reason,native_error=error,callbacks_before_reject=details);break
   assert status==0,tick;number=struct.unpack('<Q',native.read(8))[0];c,n,callcount,queuecount=struct.unpack('<4H',native.read(8));calls=list(struct.unpack('<'+'H'*callcount,native.read(callcount*2)));queued=list(struct.unpack('<'+'H'*queuecount,native.read(queuecount*2)));actual=native.read(len(data));assert actual==expected,(tick,[i for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]);assert(number,c,n,calls,queued)==(tick,uword(expected,0x355a),uword(expected,0x3552),*details);states.append(bytes(expected));callback_total+=len(calls);trajectory.append(dict(tick=tick,cursor=c,plants=n,callbacks=calls,world_sha256=hashlib.sha256(actual).hexdigest()))
  assert not native.read(1)
 app=ROOT/'private/build/rust-target/debug/genewars-game.exe';apppin=pin(app);runroot=WORK/uuid.uuid4().hex;runroot.mkdir();adapter=[]
 for ticks in [0,1,min(128,len(states)-1)]:
  script=runroot/f'ticks-{ticks}.txt';script.write_text('click 312 140\nload\n'+(f'tick {ticks}\n'if ticks else''));report=runroot/f'ticks-{ticks}.json';world=runroot/f'ticks-{ticks}.bin';result=subprocess.run([str(app),str(ROOT/'private/integration/game-runtime'),'--headless',str(script),'--report',str(report),'--world-output',str(world)],capture_output=True,timeout=90);assert result.returncode==0,result.stderr.decode(errors='replace');r=json.loads(report.read_bytes());assert world.read_bytes()==states[ticks];assert r['scene_history']==['Menu','Loading','Mission']and r['mission']['steps']==ticks and not r['full_world_tick']and not r['gameplay'];assert r['mission']['world_sha256']==hashlib.sha256(states[ticks]).hexdigest();adapter.append(dict(steps=ticks,report=pin(report),world=pin(world),rgb_sha256=r['mission']['rgb_sha256']))
 script=runroot/'extreme-pointer.txt';script.write_text('click -2147483648 -2147483648\nclick 2147483647 2147483647\n');report=runroot/'extreme-pointer.json';result=subprocess.run([str(app),str(ROOT/'private/integration/game-runtime'),'--headless',str(script),'--report',str(report)],capture_output=True,timeout=90);assert result.returncode==0,result.stderr.decode(errors='replace');r=json.loads(report.read_bytes());assert r['scene']=='Menu'and r['mission']is None
 assert apppin==pin(app)and sourcepins==[pin(p)for p in sources]
 result=dict(status='passed-independent-native-runtime-world-review',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,gui_execution=False,source_pins=sourcepins,proof_pins=proofpins,unlink_pin=pin(unlink),source_mission=pin(mission),parameters=pin(parameterpath),successful_steps=len(trajectory),callback_count=callback_total,full_state_bytes_compared=len(data)*len(trajectory),trajectory=trajectory,atomic_reproduction_boundary=blocked,headless_adapter_cases=adapter,extreme_pointer_case=pin(report),app_binary=apppin,harness=pin(exe),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],resolved_findings=['Retired ownlinks preserved; singleton headkind only cleared; raw NULL indices preserved','Zero-class fallback uses original floorcell','i32 headless hit subtraction safely rejects extreme coordinates'],limits=['Exact source-owned plant subset only; otherfamily callbacks and cleanup remain suspended','Geometric independent neighbor model applies to validated map/list membership; no invented RNG/spawn behavior','World bytes compared through first unsupported reproduction boundary with rollback; no full originaltick or hostcadence equivalence','Headless adapter rendering bound by hashes/source review; no new GUI/raw original framebuffer comparison'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','successful_steps','callback_count','full_state_bytes_compared','atomic_reproduction_boundary']}))
if __name__=='__main__':main()
