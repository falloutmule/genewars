"""Independent native modes0/6/15: arithmetic samples, masks and full frames."""
from pathlib import Path
from datetime import datetime,timezone
import collections,hashlib,importlib.util,json,random,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
spec=importlib.util.spec_from_file_location('triangle_review',Path(__file__).with_name('mode5-triangle-review.py'));triangle=importlib.util.module_from_spec(spec);spec.loader.exec_module(triangle)
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def sampler(case,source,fade):
 mode,length,base,capacity,width,left,right,*operands=case
 untouched=[(1,165)]*(capacity+8);l,r=left//65536,right//65536
 if not 1<=width<=640:return (1,0,0,0),untouched
 if not -4096<=l<=4096 or not -4096<=r<=4096:return (2,0,0,0),untouched
 if r<l:return (3,0,0,0),untouched
 start=max(l,0);count=max(0,min(r,width)-start)
 if not count:return (0,start,0,0),untouched
 if count>capacity:return (4,0,0,0),untouched
 attrs,steps=operands[:3],operands[3:];pixels=[];opaque=0
 for i in range(count):
  # Direct expression at sample numberi, rather than native iterative adds.
  u,v,s=[((a+(max(-l,0)+i)*d)%2**24)//65536 for a,d in zip(attrs,steps)]
  address=base+v*256+u
  if address>=2**64:return (7,0,0,0),untouched
  if address>=length:return (6,0,0,0),untouched
  texel=source[address]
  if not texel:pixels.append((0,0));continue
  if s>=64:return (5,0,0,0),untouched
  pixels.append((1,fade[s*256+texel]));opaque+=1
 return (0,start,count,opaque),[(1,165)]*4+pixels+[(1,165)]*(capacity-count+4)
def geometry(v,h):
 if not 1<=h<=480:return 8,None
 if any(not -4096<=a<=4096 for p in v for a in p[:2]):return 9,None
 c=triangle.dispatch(v)
 if c is None:return 15,None
 kind,t,m,b=c
 if b[1]<=0 or t[1]>=h:return 15,None
 left=t[0]<<16;right=(m[0] if kind=='top' else t[0])<<16
 if kind in ['top','bottom']:
  sl=triangle.slope(t,b if kind=='top' else m);sr=triangle.slope(m,b) if kind=='top' else triangle.slope(t,b);middle=b[1];ll,lr=sl,sr
 else:
  long,upper,lower=triangle.slope(t,b),triangle.slope(t,m),triangle.slope(m,b)
  if (kind=='right' and upper<=long) or (kind=='left' and long<=upper):return 15,None
  sl,sr,ll,lr=(long,upper,long,lower) if kind=='right' else (upper,long,lower,long);middle=m[1]
 rows=[]
 for y in range(t[1],min(b[1],h)):
  if y==middle:
   sl,sr=ll,lr
   if kind=='right':right=m[0]<<16
   if kind=='left':left=m[0]<<16
  if y>=0:rows.append([y,left,right])
  left,right=triangle.wrap(left+sl),triangle.wrap(right+sr)
 return 0,rows
def frame(case,rows,mode,gla,source,fade,prepared=None):
 h,width,pitch,color,mutation,*_=case;rows=[r[:] for r in rows];length=1 if mutation==9 else pitch*h;original=bytes((i*17+3)&255 for i in range(length+8))
 if rows:
  r=rows[-1]
  if mutation==1:r[0]=h
  if mutation==3:r[1]=2**31-1
  if mutation==4:r[1:3]=[2<<16,1<<16]
  if mutation==5:r[1:3]=[4096<<16,4096<<16]
  if mutation==6:r[1:3]=[-4096<<16,-1]
 if not 1<=width<=640:return 1,0,original
 if mutation==7:return 8,0,original
 if pitch<width:return 11,0,original
 if length<pitch*h:return 12,0,original
 output=bytearray(original);written=0
 for index,(y,left,right) in enumerate(rows):
  if y<0 or y>=h:return 10,0,original
  l,r=left//65536,right//65536
  if not -4096<=l<=4096 or not -4096<=r<=4096:return 2,0,original
  if r<l:return 3,0,original
  start,end=max(l,0),min(r,width)
  if mode==6:
   gradients,p,kind=prepared;attrs=p[index][3:]
   if mutation==2 and index==len(rows)-1:left,right,attrs=0,65536,[65536,0,64<<16]
   code,pixels=sampler((0,len(source),0,width,width,left,right,*attrs,*gradients),source,fade)
   if code[0]:return code[0],0,original
   start=code[1]
   for n,(opaque,value) in enumerate(pixels[4:4+code[2]]):
    if opaque:output[4+y*pitch+start+n]=value;written+=1
  else:
   for x in range(start,end):
    off=4+y*pitch+x;output[off]=color if mode==0 else gla[output[off]*256+color];written+=1
 return 0,written,bytes(output)
def main():
 source_path=ROOT/'rust/crates/genewars-render/src/terrain_modes.rs';source_pin=record(source_path)
 proof_path=ROOT/'research/ghidra/terrain-modes-evidence.json';proof=json.loads(proof_path.read_bytes())
 for p in proof['instruction_contracts']:assert record(ROOT/p['private_path'])['sha256']==p['sha256']
 work=ROOT/'private/review/terrain-modes';work.mkdir(parents=True,exist_ok=True)
 corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results']
 paths=[ROOT/next(r['private_output'] for r in corpus if r['entry_id']==f'root/2/disc/{i}' and r['status']=='passed') for i in [40,61,107]]
 atlas,actual_fade,gla=[p.read_bytes() for p in paths];assert len(atlas)==163840 and len(actual_fade)==16384 and len(gla)==65536
 source=bytes((3*x+5*y)&255 for y in range(256) for x in range(256));fade=bytes((i+7*s)&255 for s in range(64) for i in range(256));sources=[source,atlas,bytes(65536),bytes([1])*65536]
 cases=[];labels=[]
 def add(label,width,left,right,attrs,steps,mode=0,length=65536,base=0,capacity=640):cases.append((mode,length,base,capacity,width,left,right,*attrs,*steps));labels.append(label)
 for v in proof['mode6_synthetic_vectors']:
  add('receipt-'+v['name'],v['clip_width'],v['left_fixed'],v['right_fixed'],v['attrs_u_v_s'],v['steps_du_dv_ds'])
  h,p=sampler(cases[-1],source,fade);assert h[0]==0 and h[1]==v['output']['start'] and [n[0] for n in p[4:4+h[2]]]==v['output']['mask'] and [n[1] for n in p[4:4+h[2]]]==v['output']['pixels']
 for tile in [157,158,159]:
  base=tile//8*8192+(tile%8)*32
  for y in range(32):add('actual-tile-'+str(tile),32,0,32<<16,[0,y<<16,32<<16],[65536,0,0],1,len(atlas),base,32)
 for frac in range(65536):add('every-shade-fraction',3,0,3<<16,[1<<16,0,(32<<16)|frac],[0,0,257],capacity=3)
 rng=random.Random(0xe42aa)
 for i in range(5000):
  left=rng.randrange(-4096,4097);right=rng.randrange(left,4097);width=rng.randrange(1,641)
  attrs=[triangle.wrap(rng.getrandbits(32)),triangle.wrap(rng.getrandbits(32)),rng.randrange(64)<<16|rng.randrange(65536)]
  steps=[triangle.wrap(rng.getrandbits(32)),triangle.wrap(rng.getrandbits(32)),0 if i<2500 else triangle.wrap(rng.getrandbits(32))]
  add('random-carry',width,(left<<16)+rng.randrange(65536),(right<<16)+rng.randrange(65536),attrs,steps,mode=0 if i<2500 else 2 if i%2 else 3,length=65536 if i<2500 else rng.randrange(65537))
 for width in range(1,641):add('every-span-width',width,0,width<<16,[0x2ffff,0x4ffff,0x20ffff],[-32768,16384,-1],capacity=width)
 specials=[('transparent-invalid-shade',1,0,65536,[0,0,255<<16],[0,0,0],2,65536,0,1),('opaque-mapped-zero',1,0,65536,[83<<16,0,65536],[0,0,0],0,65536,0,1),('late-invalid-shade',4,0,4<<16,[1<<16,0,63<<16],[0,0,65536],3,65536,0,4),('late-source',4,0,4<<16,[0,0,0],[65536,0,0],2,2,0,4),('address-overflow',1,0,65536,[65536,0,0],[0,0,0],2,65536,2**64-1,1),('output-short',4,0,4<<16,[0,0,0],[0,0,0],2,65536,0,3),('empty-right',640,4096<<16,4096<<16,[0,0,0],[0,0,0],2,0,2**64-1,0),('inverted',4,65536,0,[0,0,0],[0,0,0],2,0,0,0),('edge-low',4,-4097<<16,0,[0,0,0],[0,0,0],2,0,0,0),('edge-high',4,0,4097<<16,[0,0,0],[0,0,0],2,0,0,0)]
 for row in specials:add(*row)
 for width in [0,-1,641]:add('width-admission',width,0,65536,[0,0,0],[0,0,0])
 (work/'spans.bin').write_bytes(b''.join(struct.pack('<IIQI9i',*c) for c in cases))
 geocases=[];geolabels=[]
 def geo(label,v,h=20,width=32,pitch=35,color=97,mutation=0):geocases.append((h,width,pitch,color,mutation,*[n for p in v for n in p]));geolabels.append(label)
 for v in json.loads((ROOT/'research/ghidra/mode5-triangle-evidence.json').read_bytes())['vectors']:geo('triangle-receipt',v['vertices_x_y_u_v_s'],v['clip_height'])
 zero=proof['zero_gradient_width_geometry_vector'];geo('geometry-only-zero-width',zero['vertices'],zero['clip_height'])
 for i in range(1200):
  v=[[rng.randrange(-35,65),rng.randrange(-35,65),rng.randrange(32)<<16,rng.randrange(32)<<16,32<<16] for _ in range(3)]
  geo('random-geometry',v,rng.choice([1,4,20,480]),rng.choice([1,32,640]),643,color=rng.randrange(256))
 v=[[1,0,0,0,32<<16],[8,0,0,0,32<<16],[4,5,0,0,32<<16]]
 for mutation in range(10):geo('public-row-'+str(mutation),v,mutation=mutation)
 for width,pitch in [(0,35),(-1,35),(641,643),(32,31)]:geo('frame-admission',v,width=width,pitch=pitch)
 geo('fully-clipped-right',[[1000,0,0,0,0],[1010,0,0,0,0],[1005,3,0,0,0]],3,640,640)
 for h in [0,-1,481]:geo('prepare-admission',v,h)
 for n in [-4097,4097]:geo('coordinate-admission',[[n,0,0,0,0],v[1],v[2]])
 (work/'geometry.bin').write_bytes(b''.join(struct.pack('<20i',*c) for c in geocases))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=work/'review.exe'
 r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work),*[str(p) for p in paths]],capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 hist=collections.Counter();opaque=sampled=0;tile_outputs={tile:[bytearray(),bytearray()] for tile in [157,158,159]}
 with (work/'spans-out.bin').open('rb') as native:
  for case,label in zip(cases,labels):
   expected,pixels=sampler(case,sources[case[0]],actual_fade if case[0]==1 else fade);head=struct.unpack('<4i',native.read(16));output=list(struct.iter_unpack('<BB',native.read((case[3]+8)*2)))
   assert head==expected and output==pixels,(label,case,head,expected);hist[head[0]]+=1;sampled+=head[2];opaque+=head[3]
   if label.startswith('actual-tile-'):
    tile=int(label.rsplit('-',1)[1]);tile_outputs[tile][0].extend(p[1] for p in pixels[4:4+head[2]]);tile_outputs[tile][1].extend(p[0] for p in pixels[4:4+head[2]])
  assert not native.read(1)
 for tile,(pixels,mask) in tile_outputs.items():
  reference=next(r for r in proof['mode6_actual_fixtures'] if r['tile']==tile)
  assert hashlib.sha256(pixels).hexdigest()==reference['pixels_sha256'] and hashlib.sha256(mask).hexdigest()==reference['mask_sha256']
 assert (work/'gla-out.bin').read_bytes()==gla
 def integer(stream):return struct.unpack('<i',stream.read(4))[0]
 geo_hist=collections.Counter();frame_hist=collections.Counter();rows_total=frames_total=pixels_total=0
 with (work/'geometry-out.bin').open('rb') as native:
  for case,label in zip(geocases,geolabels):
   v=[list(case[5+j*5:10+j*5]) for j in range(3)];code,rows=geometry(v,case[0]);observed=integer(native);assert code==observed,(label,case,code,observed);geo_hist[code]+=1
   if code:continue
   count=integer(native);observed_rows=[list(struct.unpack('<3i',native.read(12))) for _ in range(count)];assert observed_rows==rows;rows_total+=count
   for mode in [0,15]:
    status,written,length=integer(native),integer(native),integer(native);output=native.read(length);expected=frame(case,rows,mode,gla,source,fade)
    assert (status,written,output)==expected,(label,case,mode,status,written,expected[:2]);frame_hist[str(mode)+'/'+str(status)]+=1;frames_total+=1;pixels_total+=written
   c,p=triangle.prepare(v,case[0]);expected_code={0:0,1:15,2:14,3:9,4:8}[c];observed=integer(native);assert observed==expected_code
   if c:continue
   status,written,length=integer(native),integer(native),integer(native);output=native.read(length);expected=frame(case,rows,6,gla,source,fade,p)
   assert (status,written,output)==expected,(label,case,6,status,written,expected[:2]);frame_hist['6/'+str(status)]+=1;frames_total+=1;pixels_total+=written
  assert not native.read(1)
 assert record(source_path)==source_pin,'Source changed during review; rerun against stable input'
 result=dict(status='passed-independent-native-terrain-modes-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,native_mode6_spans=len(cases),mode6_samples_compared=sampled,opaque_mode6_samples=opaque,span_status_counts=dict(hist),all_65536_shade_fractions=True,actual_overlay_tiles=[157,158,159],all_main_gla_lookup_pairs=65536,geometry_cases=len(geocases),geometry_status_counts=dict(geo_hist),rows_compared=rows_total,complete_frames_compared=frames_total,frame_status_counts=dict(frame_hist),written_frame_pixels=pixels_total,
  evidence=record(proof_path),source=source_pin,harness=record(exe),private_resources=[record(p) for p in paths],review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],limits=['Explicit snapshots, operands, colors and placements only; no initial lighting or complete scene inferred','Mode6uses independent24bit sample expressions; sourcezero skipsFADE but stillchecks source bounds','Geometry0/15 skips attribute-gradient width calculation and admits its roundedzero case','All caller outputs, fullframes and sentinels stay unmodified on late validation error'])
 (ROOT/'research/variants/terrain-modes-review.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ['private_resources','review_sources']}))
if __name__=='__main__':main()
