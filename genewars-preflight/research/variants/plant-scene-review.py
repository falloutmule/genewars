"""Independent explicit scene composition/strict snapshot/native boundary audit."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,itertools,json,random,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
def load(name,file):s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
passes=load('own_passes','plant-passes-review.py');queue=load('own_queue','entity-queue-review.py');pin=passes.pin
WORK=ROOT/'private/review/plant-scene'
def main():
 WORK.mkdir(parents=True,exist_ok=True);source_paths=[ROOT/'rust/crates/genewars-render/src'/n for n in ['plant.rs','entity_shadow.rs','plant_draw.rs','plant_scene.rs','plant_snapshot.rs']]+[ROOT/'rust/crates/genewars-tools/src/plant_scene.rs',ROOT/'rust/crates/genewars-tools/src/terrain_render.rs',ROOT/'rust/crates/genewars-core/src/entity_draw_queue.rs'];sourcepins=[pin(p) for p in source_paths]
 proof=json.loads((ROOT/'research/ghidra/plant-color-evidence.json').read_bytes());corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results'];paths=[];resources=[]
 for r in proof['resources']:p=ROOT/next(v['private_output'] for v in corpus if v['status']=='passed' and v['entry_id']==r['entry_id']);assert pin(p)['sha256']==r['decoded_sha256'];paths.append(p);resources.append(p.read_bytes())
 synthetic=WORK/'synthetic.anb';synthetic.write_bytes(passes.synthetic());paths.append(synthetic);resources.append(synthetic.read_bytes());(WORK/'resources.txt').write_text('\n'.join(str(p) for p in paths));selections=[(i,a,0) for i in range(12) for a in range(9)]+[(12,0,0),(12,0,1),(12,1,0)];models=[passes.frame(resources[r],a,f) for r,a,f in selections];(WORK/'frames.bin').write_bytes(bytes(v for r in selections for v in r))
 tables=[]
 for id,name in [(61,'fade'),(107,'gla')]:p=ROOT/next(v['private_output'] for v in corpus if v['status']=='passed' and v['entry_id']==f'root/2/disc/{id}');tables.append(p.read_bytes());(WORK/f'{name}.bin').write_bytes(tables[-1])
 fades=[tables[0],bytes((r*13+i*7+3)%256 for r in range(64) for i in range(256))];glas=[tables[1],bytes((r*19+i*7+5)%256 for r in range(256) for i in range(256))]
 def rec(m,id=0,x=24,y=24,ctx=0,flags=4,native=0,shade=0,draw=True):return(m,id,x,y,ctx,flags,native,shade,draw)
 def scene(records,w=48,h=36,pitch=53,prefix=7,delta=0,fade=1,table=0,seed=11):return((w,h,pitch,prefix,delta,fade,table,seed,len(records)),records)
 cases=[];categories={}
 def add(name,items):start=len(cases);cases.extend(items);categories[name]=len(cases)-start
 add('all_real_frames_ordered_overlap',(scene([rec(m,7),rec((m+9)%108,7),rec(109,0xffffffff)],table=t) for m in range(108) for t in [0,1]))
 add('order_permutations',(scene(list(order),table=1) for order in itertools.permutations([rec(108,1,x=2,y=3,flags=2),rec(109,2,x=2,y=3,shade=29),rec(110,3,x=2,y=3,flags=2)])))
 add('late_rejection_atomic',(scene([rec(0,1),rec(9,2),rec(108,3,ctx=ctx,flags=f,native=n,shade=s)],fade=fade) for ctx,f,n,s,fade in [(2,0,0,0,1),(0,16,0,0,1),(0,0,2,0,1),(0,0,0,0,0),(0,2,0,0,0)]))
 add('all_dispatch_byte_axes',(scene([rec(108,1,x=2,y=2,flags=2),rec(109,2,x=2,y=2,ctx=c,flags=f,native=n,shade=s,draw=d)],table=1,fade=fd) for v in range(256) for c,f,n,s in [(v,4,0,0),(0,v,0,29),(0,4,v,0),(0,0,0,v)] for d,fd in [(True,True),(True,False),(False,False)]))
 add('empty_skip_and_geometry',(scene(records,w=w,h=h,pitch=p,prefix=pre,delta=delta) for records in [[],[rec(108,ctx=2,flags=255,native=255,draw=False)]] for w,h,p in [(1,1,1),(5,3,8),(512,480,640),(0,1,1),(1,0,1),(641,1,641),(1,481,1),(4,2,3)] for pre in [0,128] for delta in [-1,0,1]))
 rng=random.Random(0x831406c230);randomcases=[]
 for i in range(2048):
  records=[rec(rng.randrange(111),rng.choice([0,7,0xffffffff]),x=rng.choice([-65536,-32768,-1,0,1,24,47,65536,2**31-1]),y=rng.choice([-32768,-1,0,1,24,35,-2**31]),flags=rng.choice([0,2,4,8,16,255]),shade=rng.choice([0,29,30,255]),draw=rng.choice([True,False])) for _ in range(rng.randrange(17))];randomcases.append(scene(records,table=rng.randrange(2),fade=rng.randrange(2)))
 add('mixed_random_ordered_scenes',randomcases)
 with (WORK/'scenes.bin').open('wb') as out:
  out.write(struct.pack('<I',len(cases)))
  for h,records in cases:
   out.write(struct.pack('<9i',*h))
   for r in records:out.write(struct.pack('<9i',r[0],r[1] if r[1]<2**31 else r[1]-2**32,*r[2:]))
 slots=WORK/'slots';slots.mkdir(exist_ok=True)
 for i,b in enumerate(resources[:12]):(slots/f'slot-{i:02}.anb').write_bytes(b)
 snapshots=[];snapshotcases=[]
 def snapshot(name,context,plants):p=WORK/name;p.write_text('GWPLANT1\ncontext '+' '.join(map(str,context))+'\n'+''.join('plant '+' '.join(map(str,r))+'\n' for r in plants),encoding='utf8');snapshots.append(p)
 base=[(i,i%12,0,0,24+(i%3)*3,y,[0,29,30,255][i%4],[0,2,4,8][i%4],0) for i,y in enumerate([-1,0,1,1,-1,480,479,480,519,479,0])]
 for k,context in enumerate([(1,0,128,1,1,0),(1,254,128,1,0,0),(1,256,128,1,255,2),(0,0,128,1,1,0),(1,0,128,0,1,2)]):snapshot(f'snapshot-{k}.txt',context,base);snapshotcases.append((context,base))
 valid=(WORK/'snapshot-0.txt').read_bytes();bad=[b'',b'GWPLANT2\n',b'\xff',valid.replace(b'context 1',b'context 2',1),valid.replace(b'128',b'0',1),valid.replace(b'plant 0 ',b'plant -1 ',1),valid.replace(b'plant 0 0',b'plant 0 12',1),valid.replace(b'plant 1 ',b'plant 0 ',1),valid+b'garbage\n',valid+b'plant 9000 0 0 0 2147483648 0 0 0 0\n',b'GWPLANT1\ncontext 1 257 128 1 1 0\n',b'GWPLANT1\ncontext 1 -1 128 1 1 0\n',bytes(1024*1024+1)]
 for i,b in enumerate(bad):p=WORK/f'invalid-{i}.txt';p.write_bytes(b);snapshots.append(p)
 (WORK/'snapshots.txt').write_text('\n'.join(str(p) for p in snapshots))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=WORK/'review.exe'
 commands=[]
 for crate in ['assets','core','render']:
  args=['--crate-name',f'genewars_{crate}','--crate-type','rlib',str(ROOT/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={WORK}','-o',str(WORK/f'libgenewars_{crate}.rlib')]
  if crate=='render':args+=['--extern',f'genewars_assets={WORK}/libgenewars_assets.rlib']
  commands.append(args)
 commands.append([str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={WORK}',*sum((['--extern',f'genewars_{c}={WORK}/libgenewars_{c}.rlib'] for c in ['assets','core','render']),[]),'-o',str(exe)])
 for args in commands:r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(WORK)],capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace')
 def read(s,n):b=s.read(n);assert len(b)==n;return b
 def u32(s):return struct.unpack('<I',read(s,4))[0]
 pixels=success=errors=recordschecked=0;digest=hashlib.sha256()
 with (WORK/'results.bin').open('rb') as out:
  for number,(h,records) in enumerate(cases):
   w,height,pitch,prefix,delta,hasfade,t,seed,_=h;minimum=(height-1)*pitch+w if w and height else 0;extent=max(0,minimum+delta);initial=bytes((i*37+seed)%256 for i in range(prefix+extent+3));image=initial;error=None;receipts=[];stats=[0]*6
   if not(1<=w<=640 and 1<=height<=480 and pitch>=w and extent>=minimum):error=(1,0)
   else:
    for m,id,x,y,ctx,flags,native,shade,draw in records:
     if not draw:receipts.append((id,False,None));continue
     case=(2,m,x,y,0,0,0,w,height,pitch,prefix,delta,ctx,flags,native,shade,hasfade,seed,t);status,head,new=passes.oracle(case,models,fades[t],glas[t],image)
     if status:error=(2,id);break
     image=new;receipts.append((id,True,(head[0],head[4],head[5],head[1],head[2])))
     for i,v in enumerate([head[4],head[5],head[1],head[2],head[4]+head[1],head[5]+head[2]]):stats[i]+=v
   status=read(out,1)[0]
   if error:assert status==1,number;actualerror=(read(out,1)[0],u32(out));assert actualerror==error,(number,actualerror,error);image=initial;errors+=1
   else:
    assert status==0,number;actualstats=list(struct.unpack('<6Q',read(out,48)));count=u32(out);actual=[]
    for _ in range(count):id=u32(out);draw=bool(read(out,1)[0]);value=struct.unpack('<h4I',read(out,18)) if draw else None;actual.append((id,draw,value))
    assert(actualstats,actual)==(stats,receipts),number;success+=1;recordschecked+=len(actual)
   observed=read(out,u32(out));assert observed==image,number;pixels+=len(image);digest.update(image)
  assert u32(out)==len(snapshotcases) and u32(out)==len(bad) and not out.read(1)
 for k,(context,plants) in enumerate(snapshotcases):
  active,count,margin,gate,mode,ctx=context;queued=queue.chain(active,count,margin,[(r[0],r[4],r[5],r[6],r[7]) for r in plants]);expected=bytearray((i*37+11)%256 for i in range(307200));metadata=json.loads((WORK/f'snapshot-result-{k}.json').read_bytes());assert metadata['queue_order']==[r[0] for r in queued[1]] and metadata['final_count']==queued[0]
  for id,x,drawy,y,bucket,residual,shade,flags in queued[1]:
   p=next(p for p in plants if p[0]==id);draw=bool(gate and(mode or flags&8));row=next(v for v in metadata['records'] if v['id']==id);assert row['origin']==[x,drawy] and row['bucket']==bucket and row['residual']==residual and row['drawn']==draw and row['flags_after']==(flags&243 if draw else flags)
   if draw:
    case=(2,p[1]*9,x,drawy,0,0,0,512,480,640,128,0,ctx,flags,p[8],shade,1,11,0);_,_,pixels_after=passes.oracle(case,models,fades[0],glas[0],bytes(expected)+bytes(3));expected=bytearray(pixels_after[:-3])
  observed=(WORK/f'snapshot-frame-{k}.bin').read_bytes();assert observed==expected,k;pixels+=len(observed);digest.update(observed)
 assert sourcepins==[pin(p) for p in source_paths]
 report=dict(status='passed-independent-native-plant-scene-review',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,source_pins=sourcepins,scene_cases=len(cases),categories=categories,successful_scenes=success,rejected_scenes=errors,record_receipts=recordschecked,compared_destination_bytes=pixels,strict_snapshot_frames=len(snapshotcases),strict_parser_rejections=len(bad),output_aggregate_sha256=digest.hexdigest(),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],native_harness=pin(exe),limits=['Explicit post-admission flags/order/coordinates only; no original first-scene state inferred','Snapshot parser enforces uniqueIDs/margin128 even though purequeue supportsopaque repeatedIDs/margin0','Snapshot loads and validates all admitted frames even when drawpacket skips; stricter resource admission','Terrain_render integration reviewed foroffset128/pitch640 and explicitGENEPANL policy; fullterrainpixels covered by separate integration validators'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:report[k] for k in ['status','scene_cases','compared_destination_bytes','strict_snapshot_frames']}))
if __name__=='__main__':main()
