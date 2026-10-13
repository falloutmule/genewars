"""Independent cached strip evolution and ordered native event comparison."""
from pathlib import Path
from datetime import datetime,timezone
import collections,hashlib,json,random,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def uv(corner,flags):
 u,v=[(0,0),(1,0),(1,1),(0,1)][corner]
 u^=(flags>>4)&1;v^=(flags>>5)&1
 if flags&64:u,v=v,u
 return u*0x1fffff,v*0x1fffff
def model(region,camera,column,top,bottom,flags):
 if any(n<0 or n>127 for n in camera):return 1,None
 if column<0 or column>19:return 2,None
 if top<0 or top>=bottom or bottom>480:return 3,None
 x=camera[0]+column//2;y=camera[1]-column//2-1-(column&1);baseline=-16*(1+(column&1))-top
 def vertex(x,y,screen_x,base):
  index=(y%128)*128+x%128;off=index*14;light,height=struct.unpack_from('<hh',region,off)
  return dict(world=[x,y],index=index,screen=[screen_x,base-2*(height//64)],shade=4*light*region[off+12]+32768,tex=region[off+4],orient=region[off+5],flags=region[off+11],overlay=region[off+13])
 a=vertex(x,y,32*(column+1),baseline);baseline+=16;b=vertex(x,y+1,32*column,baseline)
 events=[];culled_dirty=0;culled=0
 for pair in range(48):
  x+=1;y+=1;baseline+=16;c=vertex(x,y,32*(column+1),baseline);baseline+=16;d=vertex(x,y+1,32*column,baseline)
  for half,owner,points,corners,mask in [(0,a,[a,c,b],[0,2,3],251),(1,b,[b,c,d],[0,1,2],247)]:
   if flags&4:events.append([0,*owner['world'],owner['index'],mask]+[0]*30)
   if all(p['screen'][1]<0 for p in points):
    culled+=1;culled_dirty+=bool(flags&4);continue
   mode=5 if any(p['shade'] for p in points) else 0
   world=[n for p in points for n in p['world']]
   vertices=[n for p,corner in zip(points,corners) for n in [*p['screen'],*uv(corner,owner['orient']),p['shade']]]
   def event(mode,overlay=-1):return [1,pair,half,mode,*owner['world'],owner['index'],owner['tex'],int(owner['tex']>=160),(owner['orient']>>4)&7,*corners,*world,*vertices,overlay]
   events.append(event(mode))
   if owner['overlay']>1:
    # Threshold chain expresses the source's index, independently of division.
    n=owner['overlay'];table=0 if n<=32 else 1 if n<=64 else 2
    events.append(event(6,table))
   if flags&64 and owner['flags']&1 and not b['flags']&32:events.append(event(15))
  a,b=c,d
  if a['screen'][1]>=bottom-top and b['screen'][1]>=bottom-top:return 0,(pair+1,events,culled,culled_dirty)
 return 4,None
def main():
 source_path=ROOT/'rust/crates/genewars-render/src/terrain_strip.rs';source_pin=record(source_path)
 proof_path=ROOT/'research/ghidra/terrain-strip-evidence.json';proof=json.loads(proof_path.read_bytes())
 for p in proof['instruction_contracts']:assert record(ROOT/p['private_path'])['sha256']==p['sha256']
 work=ROOT/'private/review/terrain-strip';work.mkdir(parents=True,exist_ok=True)
 level=ROOT/'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ001.DAT';assert record(level)['sha256']==proof['source_sha256']
 maps=[level.read_bytes()[0x355e:0x3b55e]];names=['actual-conq001']
 for kind in ['mixed-flags','maximum-height','minimum-height','zero-shade']:
  data=bytearray(229376)
  for i in range(16384):
   light=(i%19-9)*2048;height=(i%17-8)*64
   if kind=='maximum-height':height=32767
   if kind=='minimum-height':height=-32768
   if kind=='zero-shade':light=-8192
   struct.pack_into('<hh',data,i*14,light,height)
   data[i*14+4]=(i*73)%256;data[i*14+5]=(i*151)%256;data[i*14+11]=(i*97)%256
   data[i*14+12]=1 if kind=='zero-shade' else i%256;data[i*14+13]=(i*179)%256
  maps.append(bytes(data));names.append(kind)
 for i,data in enumerate(maps):(work/f'map-{i}.bin').write_bytes(data)
 cases=[];labels=[]
 def add(label,m,camera,column,top,bottom,flags):cases.append((m,*camera,column,top,bottom,flags));labels.append(label)
 for m in range(len(maps)):
  for camera in [(0,0),(17,29),(127,127)]:
   for column in range(20):
    for top,bottom in [(0,1),(0,480),(127,128),(64,320)]:
     for flags in [0,4,64,68]:add(names[m],m,camera,column,top,bottom,flags)
  for flags in range(256):
   for column in [0,19]:add('all-context-flags',m,(127,0),column,0,480,flags)
 rng=random.Random(0x7b7f0)
 for i in range(500):
  top=rng.randrange(480);add('random-context',rng.randrange(len(maps)),(rng.randrange(128),rng.randrange(128)),rng.randrange(20),top,rng.randrange(top+1,481),rng.randrange(256))
 for camera,column,top,bottom in [((-1,0),0,0,480),((0,128),0,0,480),((0,0),-1,0,480),((0,0),20,0,480),((0,0),0,-1,480),((0,0),0,0,481),((0,0),0,0,0),((0,0),0,480,480)]:add('admission',0,camera,column,top,bottom,68)
 (work/'cases.bin').write_bytes(b''.join(struct.pack('<7i',*c) for c in cases))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';lib=work/'libgenewars_assets.rlib';exe=work/'review.exe'
 for args in [['--crate-name','genewars_assets','--crate-type','rlib',str(ROOT/'rust/crates/genewars-assets/src/lib.rs'),'-o',str(lib)],
              [str(Path(__file__).with_suffix('.rs')),'--extern','genewars_assets='+str(lib),'-o',str(exe)]]:
  r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work),str(len(maps))],capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 def integer(stream):return struct.unpack('<i',stream.read(4))[0]
 modes=collections.Counter();sources=collections.Counter();status_counts=collections.Counter();paircounts=collections.Counter();total=dirty=culled=culled_dirty=0;orientations=set();selectors=set();overlay_indices=set()
 with (work/'results.bin').open('rb') as native:
  for case,label in zip(cases,labels):
   m,cx,cy,column,top,bottom,flags=case;code,expected=model(maps[m],[cx,cy],column,top,bottom,flags);observed=integer(native)
   assert code==observed,(label,case,code,observed);status_counts[code]+=1
   if code:continue
   pairs,count=integer(native),integer(native);events=[list(struct.unpack('<35i',native.read(140))) for _ in range(count)]
   assert pairs==expected[0] and events==expected[1],(label,case,pairs,expected[0],next(((a,b) for a,b in zip(events,expected[1]) if a!=b),None))
   paircounts[pairs]+=1;total+=count;culled+=expected[2];culled_dirty+=expected[3]
   for e in events:
    if e[0]==0:dirty+=1
    else:modes[e[3]]+=1;sources[e[8]]+=1;orientations.add(e[9]);selectors.add(e[7]);overlay_indices.add(e[-1])
  assert not native.read(1)
 assert max(paircounts)==48 and orientations==set(range(8)) and selectors==set(range(256)) and set(modes)=={0,5,6,15} and overlay_indices=={-1,0,1,2}
 assert record(source_path)==source_pin,'Source changed during review; rerun against stable input'
 result=dict(status='passed-independent-native-terrain-strip-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,native_cases=len(cases),case_groups=dict(collections.Counter(labels)),status_counts=dict(status_counts),pairs_histogram=dict(paircounts),ordered_events_compared=total,dirty_events=dirty,culled_halves=culled,dirty_events_before_culled_halves=culled_dirty,triangle_mode_counts=dict(modes),texture_source_counts=dict(sources),all_256_selectors=True,all_8_orientations=True,all_256_context_flags=True,
  evidence=record(proof_path),instruction_contracts=proof['instruction_contracts'],source=source_pin,assets_source=record(ROOT/'rust/crates/genewars-assets/src/map_atlas.rs'),harness=record(exe),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['One explicit strip; no initial camera, viewport interval or live factor chosen','Modes0/6/15 remain visible requests, not implemented pixels','Selectors160..255 are surfaced as OutOfRangeSelector, independent of unreachable rawpointer16E098 fallback','Immutable snapshot planner emits ordered dirtyAND requests without mutating map'])
 (ROOT/'research/variants/terrain-strip-review.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ['case_groups','instruction_contracts','review_sources','pairs_histogram']}))
if __name__=='__main__':main()
