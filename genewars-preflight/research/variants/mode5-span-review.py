"""Independent 72-bit carry model versus actual native prepared-span API."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,subprocess,sys,random,collections
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
sha=lambda b:hashlib.sha256(b).hexdigest()
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=sha(b))
def signed(value):return ((value+2**31)%2**32)-2**31

def oracle(case,source,fade):
 mode,length,base,capacity,width,left,right,u,v,s,du,dv,ds=case
 untouched=bytes([165])*(capacity+8)
 if width<0:return (1,0,0),untouched
 start=max(left//65536,0);end=min(right//65536,width);count=max(0,end-start)
 if not count:return (0,start,0),untouched
 if capacity<count:return (2,0,0),untouched
 skip=max(0,-(left//65536));u=(u+skip*du)%2**32;v=(v+skip*dv)%2**32;s=(s+skip*ds)%2**32
 du%=2**32;dv%=2**32;ds%=2**32
 # Concatenate C32,D32,V8 into one72-bit little-endian accumulator.
 # This single integer addition independently expresses ADD/ADC/ADC.
 c=((u&65535)<<16)|((s//256)&65535)
 d=((v&65535)<<16)|((u//65536)&255)
 vb=(v//65536)&255
 gc=((du&65535)<<16)|((ds//256)&65535)
 gd=((dv&65535)<<16)|((dv//65536)&65280)|((du//65536)&255)
 gv=(dv//65536)&255
 state=c+(d<<32)+(vb<<64);gradient=gc+(gd<<32)+(gv<<64)
 pixels=bytearray()
 for i in range(count):
  tx=(state>>32)&255;ty=(state>>64)&255;shade=(state>>8)&255
  if shade>=64:return (3,0,0),untouched
  address=base+ty*256+tx
  if address>2**64-1:return (4,0,0),untouched
  if address>=length:return (5,0,0),untouched
  pixels.append(fade[shade*256+source[address]])
  state=(state+gradient)%(2**72)
 return (0,start,count),bytes([165])*4+pixels+bytes([165])*(capacity-count+4)

def main():
 proof_path=ROOT/'research/ghidra/mode5-span-evidence.json';proof=json.loads(proof_path.read_bytes());listing=ROOT/'private/ghidra/terrain-mode5.txt'
 assert record(listing)['sha256']==proof['instruction_contracts'][0]['sha256']
 text=listing.read_text()
 for term in ['000e3fae JNC 0x000e3fb1','000e3fb0 DEC ECX','000e3fdb SAR EAX,0x10','000e3fde SAR EBP,0x10','000e4039 JMP 0x000e406e','000e406e XOR DH,DH','000e40d4 ADD ECX,dword ptr [ESP + 0x4c]','000e40db ADC EDX,dword ptr [ESP + 0x50]','000e40e5 ADC BH,byte ptr [ESP + 0x64]']:assert term in text
 mapped=(ROOT/'private/ghidra/validated/memory-.object1.bin').read_bytes();assert mapped[0xe3fab-0x10000:0xe3fb1-0x10000]==bytes.fromhex('83fd00730149')
 corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results']
 ar=next(r for r in corpus if r['entry_id']=='root/2/disc/40' and r['status']=='passed');fr=next(r for r in corpus if r['entry_id']=='root/2/disc/61' and r['status']=='passed')
 ap,fp=ROOT/ar['private_output'],ROOT/fr['private_output'];actual_source,actual_fade=ap.read_bytes(),fp.read_bytes()
 assert sha(actual_source)==proof['atlas_decoded_sha256'] and sha(actual_fade)==proof['fade_decoded_sha256']
 source=bytes((3*x+5*y)&255 for y in range(256) for x in range(256));fade=bytes((i+7*s)&255 for s in range(64) for i in range(256))
 cases=[];labels=[]
 def add(label,width,left,right,attrs,steps,*,mode=0,length=65536,base=0,capacity=96):
  cases.append((mode,length,base,capacity,width,left,right,*attrs,*steps));labels.append(label)
 for row in proof['synthetic_vectors']:
  a=row['input'];add('receipt-'+row['name'],a['clip_width'],a['left_16_16'],a['right_16_16'],a['attributes_u_v_s'],a['gradients_du_dv_ds'],base=a['texture_base'])
  head,pixels=oracle(cases[-1],source,fade);assert head==(0,row['output']['start_x'],len(row['output']['pixels'])) and pixels[4:4+head[2]]==bytes(row['output']['pixels'])
 for row in proof['actual_resource_fixtures']:
  add('actual-'+str(row['tile']),row['clip_width'],row['left_16_16'],row['right_16_16'],row['attributes_u_v_s'],row['gradients_du_dv_ds'],mode=1,length=len(actual_source),base=row['texture_base'])
  head,pixels=oracle(cases[-1],actual_source,actual_fade);assert sha(pixels[4:4+head[2]])==row['output_sha256']
 # Every unroll width, while output prefix/tail sentinels detect misplaced writes.
 for width in range(1,641):add('unroll',width,0,width<<16,[0x2ffff,0x44000,0xa8000],[0x8000,-0x4000,0],capacity=width+3)
 # Extreme horizontal coordinates, negative fractions, full32 clipping wrap.
 for left in [-2**31,-196609,-65537,-65536,-1,0,1,65535,65536,2**31-1]:
  for right in [-2**31,-1,0,1,65535,65536,196609,2**31-1]:
   for width in [0,1,640,2**31-1]:add('coverage-extreme',width,left,right,[-2**31,2**31-1,32<<16],[2**31-1,-2**31,0],capacity=max(0,min(right//65536,width)-max(left//65536,0))+2)
 rng=random.Random(0xe40d0)
 for i in range(7000):
  attrs=[signed(rng.getrandbits(32)),signed(rng.getrandbits(32)),rng.randrange(64)<<16|rng.randrange(65536)]
  steps=[signed(rng.getrandbits(32)),signed(rng.getrandbits(32)),0 if i<3500 else signed(rng.getrandbits(32))]
  left=rng.randrange(-10,20)*65536+rng.randrange(65536);right=rng.randrange(-5,90)*65536+rng.randrange(65536);width=rng.randrange(0,80)
  count=max(0,min(right//65536,width)-max(left//65536,0));capacity=count+rng.randrange(4)
  length=65536 if i<3500 else rng.randrange(65537);base=0 if i<3500 else rng.randrange(65536)
  add('random-valid' if i<3500 else 'random-bounds',width,left,right,attrs,steps,length=length,base=base,capacity=capacity)
 for name,width,left,right,attrs,steps,length,base,capacity in [
  ('late-shade',4,0,4<<16,[0,0,63<<16],[0,0,1<<16],65536,0,4),
  ('late-source',4,0,4<<16,[0,0,0],[1<<16,0,0],2,0,4),
  ('address-overflow',4,0,4<<16,[1<<16,0,0],[0,0,0],65536,2**64-1,4),
  ('short-output',4,0,4<<16,[0,0,0],[0,0,0],65536,0,3),
  ('negative-width',-1,0,1<<16,[0,0,0],[0,0,0],65536,0,3),
  ('zero-source-emits',1,0,1<<16,[0,0,0],[0,0,0],65536,0,1),
  ('zero-width-safe-deviation',0,-1,1<<16,[0,0,0],[0,0,0],0,2**64-1,0)]:
  add(name,width,left,right,attrs,steps,length=length,base=base,capacity=capacity)
 work=ROOT/'private/review/mode5-span';work.mkdir(parents=True,exist_ok=True)
 (work/'cases.bin').write_bytes(b''.join(struct.pack('<IIQI9i',*c) for c in cases))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=work/'review.exe'
 r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work),str(ap),str(fp)],capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
 histogram=collections.Counter();pixels_checked=0
 with (work/'outputs.bin').open('rb') as native:
  for label,c in zip(labels,cases):
   head,expected=oracle(c,source if c[0]==0 else actual_source,fade if c[0]==0 else actual_fade)
   observed=struct.unpack('<BiI',native.read(9));output=native.read(c[3]+8)
   assert observed==head and output==expected,(label,c,head,observed)
   histogram[head[0]]+=1;pixels_checked+=head[2]
  assert not native.read(1)
 receipt=dict(status='passed-independent-native-mode5-span-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
  cases=len(cases),written_pixels_compared=pixels_checked,case_groups=dict(collections.Counter(labels)),status_counts=dict(histogram),
  independent_model='One72-bit accumulator C|D<<32|v<<64 and one72-bit gradient addition; full32 clipping happens before packing',
  atomic_output='Entireprefix/output/tail buffer checked for everycase; rejectedlate-shade/source andshort-output remainunmodified',
  zero_width='Safe empty behavior differs from originalnegative-left zero-width unroll; original equivalence restricted topositiveclipwidth',
  evidence=record(proof_path),listing=record(listing),inputs=[record(ap),record(fp)],production_source=record(ROOT/'rust/crates/genewars-render/src/terrain.rs'),harness=record(exe),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['Preparedhorizontalspan only; no triangleedge/gradientsetup/verticalcoverage/culling','Texturecoordinates arebytes into fullpitchedallocation, not compact32x32tilebounds','Original unchecked source/shadebounds are safelyrejected; no clamping','No originalrender/frame/cadence equivalence claimed'])
 (ROOT/'research/variants/mode5-span-review.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:v for k,v in receipt.items() if k not in ['case_groups','listing','inputs','production_source','harness','review_sources']}))

if __name__=='__main__':main()
