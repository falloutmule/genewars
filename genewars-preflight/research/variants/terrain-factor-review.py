"""Independent native immutable local-factor plans, order and seam review."""
from pathlib import Path
from datetime import datetime,timezone
import importlib.util,hashlib,json,math,random,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
spec=importlib.util.spec_from_file_location('height_oracle',Path(__file__).with_name('terrain-review.py'));height_oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(height_oracle)
sha=lambda b:hashlib.sha256(b).hexdigest()
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=sha(b))
def word(x):return ((x+32768)%65536)-32768

def plan(data,target):
 x,y=target
 def height(cx,cy):return struct.unpack_from('<h',data,((cy%128)*128+cx%128)*14+2)[0]
 cx,fx=divmod(x%65536,256);cy,fy=divmod(y%65536,256)
 h=height_oracle.weighted([height(cx,cy),height(cx+1,cy),height(cx,cy+1),height(cx+1,cy+1)],fx,fy)
 startx=((x-4096)%65536)//256;starty=((y-4096)%65536)//256;output=[]
 for j in range(33):
  gy=(starty+j)%256;py=word(gy*256);sy=gy%128
  for i in range(33):
   gx=(startx+i)%256;px=word(gx*256);sx=gx%128;old=data[(sy*128+sx)*14+12]
   if old>=63:continue
   # Shortest signed displacement on the32768-period ring, independent of
   # the source's doubled16-bit subtraction/SAR implementation.
   dx=(px-x+16384)%32768-16384;dy=(py-y+16384)%32768-16384
   dz=(2*height(sx,sy)-h)//8;squared=dx*dx+dy*dy+dz*dz
   if squared>=4096**2:continue
   value=63 if squared<3840**2 else (4095-math.isqrt(squared))//4
   if value>old:output.append((px,py,sx,sy,old,value))
 return output

def main():
 proof_path=ROOT/'research/integration/terrain-factor-contract.json';proof=json.loads(proof_path.read_bytes());listings=[]
 for item in proof['instruction_contracts']:
  actual=record(ROOT/item['private_path']);assert actual['sha256']==item['sha256'];listings.append(actual)
 body=(ROOT/'private/ghidra/camera-player-input/instructions-00075b00.txt').read_text()
 for term in ['00075b93 INC AH','00075b95 INC DL','00075c39 CMP EAX,0x3f','00075c8e SAR DX,0x1','00075ca5 SAR AX,0x1','00075cac SAR EDI,0x3','00075cbc JGE 0x00075cd4','00075cd8 JGE 0x00075bff','00075cf9 SAR EAX,0x8','00075cff JBE 0x00075bff']:assert term in body
 work=ROOT/'private/review/terrain-factor';work.mkdir(parents=True,exist_ok=True)
 actual=(ROOT/'private/review/terrain/map-region.bin').read_bytes();assert sha(actual)=='57300db747f5576c3939b95d4b0089d2347c1d525293171b866537935fdb39a6'
 maps=[actual];names=['actual-CONQ001']
 for previous in [0,1,31,62,63,64,255]:
  data=bytearray(229376)
  for i in range(16384):data[i*14+12]=previous
  maps.append(bytes(data));names.append('flat-old-'+str(previous))
 for kind in ['height-checker','mixed-height-factor','constant-min-height','constant-max-height']:
  data=bytearray(229376)
  for i in range(16384):
   x,y=i%128,i//128
   h=(32767 if (x+y)%2 else -32768) if kind=='height-checker' else word(40503*i+97) if kind=='mixed-height-factor' else -32768 if kind=='constant-min-height' else 32767
   struct.pack_into('<h',data,i*14+2,h);data[i*14+12]=[0,1,31,62,63,64,255][i%7] if kind=='mixed-height-factor' else 0
  maps.append(bytes(data));names.append(kind)
 for i,data in enumerate(maps):(work/f'map-{i}.bin').write_bytes(data)
 boundary=[-32768,-32767,-1,0,1,255,32767];targets=[(x,y) for x in boundary for y in boundary]
 rng=random.Random(0x75b00);targets.extend((rng.randrange(-32768,32768),rng.randrange(-32768,32768)) for _ in range(64))
 cases=[(m,*target) for m in range(len(maps)) for target in targets]
 (work/'cases.bin').write_bytes(b''.join(struct.pack('<Ihh',*c) for c in cases))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';library=work/'libgenewars_core.rlib';exe=work/'review.exe'
 for args in [['--crate-name','genewars_core','--crate-type','rlib',str(ROOT/'rust/crates/genewars-core/src/lib.rs'),'-o',str(library)],
              [str(Path(__file__).with_suffix('.rs')),'--extern','genewars_core='+str(library),'-o',str(exe)]]:
  r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work),str(len(maps))],capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
 total=0;empty=0;counts={name:0 for name in names}
 with (work/'results.bin').open('rb') as native:
  for mapid,x,y in cases:
   expected=plan(maps[mapid],(x,y));count=struct.unpack('<I',native.read(4))[0];actual=[struct.unpack('<hhBBBB',native.read(8)) for _ in range(count)]
   assert actual==expected,(names[mapid],x,y,actual[:3],expected[:3]);total+=count;empty+=count==0;counts[names[mapid]]+=count
  assert not native.read(1)
 for name in ['flat-old-63','flat-old-64','flat-old-255']:assert counts[name]==0
 # Deliberate outer-boundary samples cannot be listed, and every plan only
 # increases values on unique sourcecells within this33×33 scan.
 zero=plan(maps[1],(0,0));assert not any(px==4096 and py==0 for px,py,*_ in zero)
 assert all(n>old for px,py,sx,sy,old,n in zero)
 receipt=dict(status='passed-independent-native-terrain-factor-plan-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
  native_plans=len(cases),map_variants=len(maps),targets_per_map=len(targets),scanned_vertices=len(cases)*1089,ordered_changes_compared=total,empty_plans=empty,changes_by_map=counts,
  invalid_extents='Short/long rejected',integer_distance_bound=3*16384**2,evidence=record(proof_path),private_listings=listings,source=record(ROOT/'rust/crates/genewars-core/src/terrain.rs'),harness=record(exe),
  review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['Pure snapshot arithmetic/orderedchangeplan, not whole75B00execution','No mapmutation, callback84B40, virtualentitypredicate orvisibilitylifecycle inferred','Sourceindices wrap128, callbackgridpositions byte-wrap256 andpreserverowmajororder','Explicit radius4096/target only; no initialmissionstate inferred'])
 (ROOT/'research/variants/terrain-factor-review.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:v for k,v in receipt.items() if k not in ['private_listings','source','harness','review_sources']}))

if __name__=='__main__':main()
