"""Independent full viewport assembly from independently reviewed lane models."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
def load(name,file):
 s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
strip=load('own_strip','terrain-strip-review.py');triangle=load('own_triangle','mode5-triangle-review.py');modes=load('own_modes','terrain-modes-review.py');factor=load('own_factor','terrain-factor-review.py');span=triangle.span
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def compose(region,camera,flags,atlas,fade,gla,initial=False):
 frame=bytearray(307200);pairs=prepared=writes=0;counts=[0]*16;dirty=[]
 if initial:flags|=4
 for column in range(4 if initial else 0,20):
  code,plan=strip.model(region,camera,column,0,480,flags)
  if code:return None,'camera'
  pairs+=plan[0]
  for e in plan[1]:
   if e[0]==0:dirty.append((e[3],e[4]));continue
   mode=e[3];counts[mode]+=1;vertices=[e[19+j*5:24+j*5] for j in range(3)]
   if mode in [0,15]:
    if mode==15 and gla is None:return None,'MAIN.GLA'
    code,rows=modes.geometry(vertices,480)
    if code==15:continue
    assert code==0
    for y,left,right in rows:
     for x in range(max(left//65536,0),min(right//65536,640)):
      off=y*640+x;frame[off]=0 if mode==0 else gla[frame[off]*256];writes+=1
   else:
    if mode==5 and e[8]:return None,'selector'
    tile=e[7] if mode==5 else 157+e[34];base=tile//8*8192+tile%8*32
    code,p=triangle.prepare(vertices,480)
    if code==1:continue
    if code:return None,'gradient'
    gradients,rows,kind=p
    for y,left,right,u,v,s in rows:
     case=(1,len(atlas),base,640,640,left,right,u,v,s,*gradients)
     if mode==5:
      head,pixels=span.oracle(case,atlas,fade)
      if head[0]:return None,'sample'
      if head[2]:frame[y*640+head[1]:y*640+head[1]+head[2]]=pixels[4:4+head[2]]
      writes+=head[2]
     else:
      head,pixels=modes.sampler(case,atlas,fade)
      if head[0]:return None,'sample'
      for x,(opaque,value) in enumerate(pixels[4:4+head[2]],head[1]):
       if opaque:frame[y*640+x]=value;writes+=1
   prepared+=1
 return (pairs,prepared,writes,counts,dirty,bytes(frame)),None
def main():
 names=['terrain_view.rs','terrain.rs','terrain_modes.rs','terrain_strip.rs'];source_pins=[record(ROOT/'rust/crates/genewars-render/src'/n) for n in names]
 work=ROOT/'private/review/terrain-view';work.mkdir(parents=True,exist_ok=True)
 corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results'];paths=[ROOT/next(r['private_output'] for r in corpus if r['entry_id']==f'root/2/disc/{i}' and r['status']=='passed') for i in [40,61,107]];atlas,fade,gla=[p.read_bytes() for p in paths]
 panel_path=ROOT/next(r['private_output'] for r in corpus if r['entry_id']=='root/2/disc/86' and r['status']=='passed');panel=panel_path.read_bytes();assert len(panel)==61440
 level=ROOT/'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ001.DAT';assert record(level)['sha256']=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2';region=level.read_bytes()[0x355e:0x3b55e];maps=[region];map_names=['actual-raw']
 data=bytearray(region)
 for _,_,sx,sy,old,value in factor.plan(region,(12345,8317)):data[(sy*128+sx)*14+12]=value
 maps.append(bytes(data));map_names.append('actual-explicit-factor-plan')
 data=bytearray(maps[1])
 for i in range(16384):data[i*14+11]=0x0d
 maps.append(bytes(data));map_names.append('actual-explicit-debug-flags')
 data=bytearray(229376)
 for i in range(16384):struct.pack_into('<h',data,i*14,-8192);data[i*14+11]=13;data[i*14+12]=1;data[i*14+13]=2
 maps.append(bytes(data));map_names.append('synthetic-zero-overlay-gla')
 data=bytearray(maps[3])
 for i in range(16384):data[i*14+4]=255
 maps.append(bytes(data));map_names.append('unsupported-selector-geometry-only')
 data=bytearray(region)
 for i in range(16384):data[i*14+4]=255
 maps.append(bytes(data));map_names.append('unsupported-selector-textured')
 for i,data in enumerate(maps):(work/f'map-{i}.bin').write_bytes(data)
 cases=[(m,cx,cy,flags,1,163840,0) for m in [0,1,2] for cx,cy in [(0,0),(17,29),(127,127)] for flags in ([0,4,68] if m==2 else [0])]
 cases.extend([(3,0,0,68,1,163840,1),(4,127,127,68,1,163840,1),(3,0,0,68,0,163840,1),(3,0,0,4,0,163840,1),(0,0,0,0,0,163840,0),(5,0,0,0,1,163840,0),(0,-1,0,0,1,163840,0),(0,0,128,0,1,163840,0),(0,0,0,0,1,163839,0)])
 cases=[(*c,0) for c in cases]
 cases.extend((m,66,48,flags,1,163840,0,view) for m in [0,1,2] for flags in [0,68] for view in [1,2])
 cases.extend([(3,0,0,0,1,163840,1,1),(3,127,127,255,1,163840,1,2),(3,66,48,68,0,163840,1,1)])
 (work/'cases.bin').write_bytes(b''.join(struct.pack('<8i',*c) for c in cases));rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';lib=work/'libgenewars_assets.rlib';exe=work/'review.exe'
 for args in [['--crate-name','genewars_assets','--crate-type','rlib',str(ROOT/'rust/crates/genewars-assets/src/lib.rs'),'-o',str(lib)],
              [str(Path(__file__).with_suffix('.rs')),'--extern','genewars_assets='+str(lib),'-o',str(exe)]]:
  r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work),str(len(maps)),*[str(p) for p in paths],str(panel_path)],capture_output=True,timeout=120);assert r.returncode==0,r.stderr.decode(errors='replace')
 def integer(s):return struct.unpack('<I',s.read(4))[0]
 frames=[];rejections=[];count_dirty=total_writes=0
 with (work/'results.bin').open('rb') as native:
  for case in cases:
   m,cx,cy,flags,has_gla,extent,res,view=case
   aa,ff,gg=(atlas,fade,gla) if res==0 else (bytes([7])*163840,bytes(i%256 for i in range(16384)),bytes((i//256+3)%256 for i in range(65536)))
   expected,error=(None,'atlas') if extent!=163840 else compose(maps[m],[cx,cy],flags,aa,ff,gg if has_gla else None,view!=0)
   if expected and view:
    assert all(expected[5][y*640+x]==0 for y in range(480) for x in range(128))
    if view==2:
     image=bytearray(expected[5])
     for y in range(480):image[y*640:y*640+128]=panel[y*128:(y+1)*128]
     expected=(*expected[:5],bytes(image))
   status=integer(native)
   if error:
    assert status==1,(case,error);message=native.read(integer(native)).decode();rejections.append(dict(case=case,reason=error,message=message));continue
   assert status==0,case
   header=tuple(integer(native) for _ in range(3));counts=[integer(native) for _ in range(16)];dirty_count=integer(native);dirty=[(integer(native),native.read(1)[0]) for _ in range(dirty_count)];pixels=native.read(307200)
   assert (header,counts,dirty,pixels)==(expected[:3],expected[3],expected[4],expected[5]),case
   if m in [3,4] and has_gla:
    if view==0:assert pixels==bytes([10])*307200
    else:assert all(pixels[y*640+128:(y+1)*640]==bytes([10 if flags&64 else 7])*512 for y in range(480))
   count_dirty+=dirty_count;total_writes+=header[2];frames.append(dict(case=case,map=map_names[m],pairs=header[0],prepared=header[1],writes=header[2],requested_modes=counts,dirty_requests=dirty_count,indices_sha256=hashlib.sha256(pixels).hexdigest()))
  assert not native.read(1)
 assert [record(ROOT/'rust/crates/genewars-render/src'/n) for n in names]==source_pins,'Source changed during review; rerun stable inputs'
 result=dict(status='passed-independent-native-full-and-initial-terrain-view-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,native_cases=len(cases),complete_frames=len(frames),full_index_pixels_compared=len(frames)*307200,pixel_writes_compared=total_writes,ordered_dirty_requests_compared=count_dirty,initial_frames=sum(r['case'][7]!=0 for r in frames),explicit_panel_frames=sum(r['case'][7]==2 for r in frames),panel_invalid_atomic_checks=len(frames)*5,rejections=rejections,frames=frames,sources=source_pins,resources=[record(p) for p in [*paths,panel_path]],harness=record(exe),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],model_dependencies=[record(Path(__file__).with_name(n)) for n in ['terrain-strip-review.py','mode5-triangle-review.py','mode5-span-review.py','terrain-modes-review.py','terrain-factor-review.py']],
  limits=['Full20column diagnostic remains host policy; initial API uses gated source columns4..19 withflagOR4','Factor-modified map and debugflags are explicit synthetic policies; no callbacks or live initialization','MAIN.GLA required at requestedmode15 evenif later geometryculls; absenttable isallowedwhenno15requested','Fresh framebuffer returned onlyoncomplete success; mapandresourceinputs immutable','GENEPANL is explicit diagnostic resource choice; initial selector remains unresolved','Original pitch equivalence assumes inherited initialized640; safe API supplies explicit640'])
 (ROOT/'research/variants/terrain-view-review.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ['frames','sources','resources','review_sources','model_dependencies']}))
if __name__=='__main__':main()
