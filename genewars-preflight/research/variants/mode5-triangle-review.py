"""Independent branch-directed classification and repeated ADD row oracle.

Only independently authored native Rust code runs; original machine code does
not run. Span samples use the separately reviewed 72-bit accumulator model.
"""
from pathlib import Path
from datetime import datetime,timezone
import collections,hashlib,importlib.util,json,random,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
spec=importlib.util.spec_from_file_location('span_review',Path(__file__).with_name('mode5-span-review.py'));span=importlib.util.module_from_spec(spec);spec.loader.exec_module(span)
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def wrap(n):return (n+2**31)%2**32-2**31
def div(n,d):assert d>0;return (-1 if n<0 else 1)*(abs(n)//d)
def delta(a,b):return wrap(b-a)
def slope(a,b):return div(wrap((b[0]-a[0])<<16),b[1]-a[1])
def dispatch(v):
 # Direct conditional tree from E1913..E199E, including pointer exchanges.
 # This deliberately differs from rotating to a minimum-y vertex.
 s,d,c=v;a,b,z=s[1],d[1],c[1]
 if a==b:
  if a==z:return None
  if a<z:return ('top',s,d,c) if d[0]>s[0] else None
  return ('bottom',c,d,s) if s[0]>d[0] else None
 if a<b:
  if a==z:return ('top',c,s,d) if s[0]>c[0] else None
  if a>z:return 'right',c,s,d
  if b==z:return ('bottom',s,c,d) if d[0]>c[0] else None
  return ('left',s,c,d) if b>z else ('right',s,d,c)
 if a==z:return ('bottom',d,s,c) if c[0]>s[0] else None
 if a<z:return 'left',d,s,c
 if b==z:return ('top',d,c,s) if c[0]>d[0] else None
 return ('right',d,c,s) if b<z else ('left',c,d,s)
def prepare(v,clip):
 if not 1<=clip<=480:return 4,None
 if any(not -4096<=n<=4096 for p in v for n in p[:2]):return 3,None
 classified=dispatch(v)
 if classified is None:return 1,None
 kind,t,m,b=classified
 if b[1]<=0 or t[1]>=clip:return 1,None
 H=b[1]-t[1];attrs=list(t[2:]);left=t[0]<<16;right=(m[0] if kind=='top' else t[0])<<16
 if kind in ['top','bottom']:
  sl=slope(t,b if kind=='top' else m);sr=slope(m,b) if kind=='top' else slope(t,b)
  aa,bb=(t,m) if kind=='top' else (m,b)
  gradients=[div(delta(x,y),bb[0]-aa[0]) for x,y in zip(aa[2:],bb[2:])]
  target=b if kind=='top' else m;dy=[div(delta(x,y),H) for x,y in zip(t[2:],target[2:])]
  middle=b[1];ll,lr,ddy=sl,sr,dy
 else:
  h=m[1]-t[1];r=(h<<16)//H;long=slope(t,b);upper=slope(t,m);lower=slope(m,b)
  if (kind=='right' and upper<=long) or (kind=='left' and long<=upper):return 1,None
  # Emulate IMUL low32/SAR for X; SHRD of signed64 product for attributes.
  ca,cb=(t,b) if kind=='right' else (b,t)
  rounded=wrap(wrap((ca[0]-cb[0])*r))>>16
  width=wrap((m[0]-t[0] if kind=='right' else t[0]-m[0])+rounded)
  if width<0:return 1,None
  if width==0:return 2,None
  gradients=[]
  for i in range(2,5):
   shifted=wrap((wrap(ca[i]-cb[i])*r)>>16)
   numerator=wrap(shifted+(m[i]-t[i] if kind=='right' else t[i]-m[i]))
   gradients.append(div(numerator,width+1))
  middle=m[1]
  if kind=='right':
   sl,sr,ll,lr=long,upper,long,lower
   dy=[div(delta(x,y),H) for x,y in zip(t[2:],b[2:])];ddy=dy
  else:
   sl,sr,ll,lr=upper,long,lower,long
   dy=[div(delta(x,y),h) for x,y in zip(t[2:],m[2:])]
   ddy=[div(delta(x,y),H-h) for x,y in zip(m[2:],b[2:])]
 rows=[]
 # Repeated ADD through every omitted row, then reset short X exactly.
 for y in range(t[1],min(b[1],clip)):
  if y==middle:
   sl,sr,dy=ll,lr,ddy
   if kind=='right':right=m[0]<<16
   if kind=='left':left=m[0]<<16
  if y>=0:rows.append([y,left,right,*attrs])
  left,right=wrap(left+sl),wrap(right+sr);attrs=[wrap(x+d) for x,d in zip(attrs,dy)]
 return 0,(gradients,rows,kind)
def draw(case,prepared,source,fade):
 h,width,pitch,length,mutation,*_=case;gradients,original_rows,_=prepared
 rows=[r[:] for r in original_rows];frame_len=1 if mutation==10 else pitch*h
 untouched=bytes([165])*(frame_len+8)
 if rows:
  last=rows[-1]
  if mutation==1:last[1:3]=[0,65536];last[5]=64<<16
  if mutation==2:last[1:]=[0,65536,255<<16,255<<16,0]
  if mutation==3:last[0]=-1
  if mutation==4:last[0]=h
  if mutation==5:last[:3]=[h-1,2**31-1,2**31-1]
  if mutation==6:last[1:3]=[-2**31,-1]
 if not 1<=width<=640:return 8,0,untouched
 if mutation in [7,8]:return 4,0,untouched
 if pitch<width:return 9,0,untouched
 if frame_len<pitch*h:return 10,0,untouched
 staged=bytearray(untouched);count=0
 for y,left,right,u,v,s in rows:
  if y<0 or y>=h:return 7,0,untouched
  code,pixels=span.oracle((0,length,0,width,width,left,right,u,v,s,*gradients),source,fade)
  if code[0]:return {3:5,5:6}[code[0]],0,untouched
  n=code[2]
  if n:
   start=4+y*pitch+code[1];staged[start:start+n]=pixels[4:4+n]
  count+=n
 return 0,count,bytes(staged)
def main():
 source_path=ROOT/'rust/crates/genewars-render/src/terrain.rs';source_pin=record(source_path)
 proof_path=ROOT/'research/ghidra/mode5-triangle-evidence.json';proof=json.loads(proof_path.read_bytes())
 for p in proof['instruction_contracts']:assert record(ROOT/p['private_path'])['sha256']==p['sha256']
 cases=[];labels=[]
 def add(label,v,h=32,w=32,pitch=35,length=65536,mutation=0):
  cases.append((h,w,pitch,length,mutation,*[n for point in v for n in point]));labels.append(label)
 for vector in proof['vectors']:
  v=vector['vertices_x_y_u_v_s'];h=vector['clip_height'];code,p=prepare(v,h)
  if 'rows_y_left_right_u_v_s' in vector:
   assert code==0 and p[0]==vector['gradients_du_dv_ds'] and p[1]==vector['rows_y_left_right_u_v_s']
  add('receipt-'+vector['name'],v,h)
 # All6 permutations preserve/reverse winding; middle lies above/at/below
 # the clip start and end. Include fully clipped X and maximum admissions.
 import itertools
 base_shapes=[[(2,-4),(20,-4),(12,18)],[(12,-4),(20,18),(2,18)],[(8,-9),(22,-2),(10,20)],[(10,-9),(12,20),(1,-2)],[(1000,0),(1010,0),(1005,3)]]
 for shape in base_shapes:
  for offset in [-4090,-20,-1,0,1,4,9,20,480]:
   v=[[x,y+offset,0x1fffff*(j&1),0x1fffff*(j>>1),0x208000] for j,(x,y) in enumerate(shape)]
   for perm in itertools.permutations(v):
    for h in [1,3,7,32,480]:add('permutation-clip',perm,h)
 rng=random.Random(0xe1900)
 for i in range(4500):
  coords=[rng.randrange(-60,95) for _ in range(6)] if i<3500 else [rng.randrange(-4096,4097) for _ in range(6)]
  attrs=[[wrap(rng.getrandbits(32)),wrap(rng.getrandbits(32)),rng.randrange(64)<<16|rng.randrange(65536)] for _ in range(3)]
  if i%3==0:
   for a in attrs:a[2]=32<<16
  v=[[coords[2*j],coords[2*j+1],*attrs[j]] for j in range(3)]
  add('random-attributes',v,rng.choice([1,3,17,32,480]),rng.choice([1,17,32,640]),640 if i%2 else 643)
 # Public constructible prepared rows: late faults, empty spans, all-frame
 # commit, pitch/tail checks. These are safe API checks, not source admissions.
 v=[[1,0,0,0,0],[8,0,0,0,0],[4,5,0,0,0]]
 for mutation in range(11):
  add('public-row-mutation-'+str(mutation),v,6,16,19,65535 if mutation==2 else 65536,mutation)
 for w,p in [(0,16),(-1,16),(641,641),(16,15)]:add('draw-admission',v,6,w,p)
 for h in [0,-1,481]:add('prepare-admission',v,h)
 for n in [-4097,4097,2**31-1,-2**31]:add('coordinate-admission',[[n,0,0,0,0],v[1],v[2]])
 work=ROOT/'private/review/mode5-triangle';work.mkdir(parents=True,exist_ok=True)
 (work/'cases.bin').write_bytes(b''.join(struct.pack('<20i',*c) for c in cases))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=work/'review.exe'
 r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work)],capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 source=bytes((3*x+5*y)&255 for y in range(256) for x in range(256));fade=bytes((i+7*s)&255 for s in range(64) for i in range(256))
 def integer(stream):return struct.unpack('<i',stream.read(4))[0]
 hist=collections.Counter();drawhist=collections.Counter();rows_total=pixels_total=frames=0
 with (work/'results.bin').open('rb') as native:
  for case,label in zip(cases,labels):
   h,*_=case;v=[list(case[5+j*5:10+j*5]) for j in range(3)];status,expected=prepare(v,h);observed=integer(native)
   assert observed==status,(label,case,status,observed);hist[status]+=1
   if status:continue
   gradients=[integer(native) for _ in range(3)];count=integer(native);rows=[list(struct.unpack('<6i',native.read(24))) for _ in range(count)]
   assert (gradients,rows)==expected[:2],(label,case,expected,(gradients,rows));rows_total+=count
   code,written,length=integer(native),integer(native),integer(native);image=native.read(length)
   expected_draw=draw(case,expected,source,fade)
   assert (code,written,image)==expected_draw,(label,case,code,written,expected_draw[:2]);drawhist[code]+=1;pixels_total+=written;frames+=1
  assert not native.read(1)
 assert record(source_path)==source_pin,'Source changed during review; rerun against stable input'
 result=dict(status='passed-independent-native-mode5-triangle-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
  native_cases=len(cases),case_groups=dict(collections.Counter(labels)),prepared_status_counts=dict(hist),draw_status_counts=dict(drawhist),rows_compared=rows_total,entire_frames_compared=frames,written_pixels_compared=pixels_total,
  evidence=record(proof_path),instruction_contracts=proof['instruction_contracts'],source=source_pin,harness=record(exe),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  model='Direct E1913..E199E conditional dispatch; repeated signed32 row additions through clipped rows, exact short-X reset, separate72-bit ADD/ADC span model',
  limits=['Safe coordinate/clip admissions and zero-general-width rejection are deliberate restrictions','No initial camera, scene ordering, culling producer or live lighting inferred','Whole frame including pitch padding and external sentinels compared; failed rows must leave all bytes unchanged'])
 (ROOT/'research/variants/mode5-triangle-review.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ['instruction_contracts','case_groups','review_sources']}))
if __name__=='__main__':main()
