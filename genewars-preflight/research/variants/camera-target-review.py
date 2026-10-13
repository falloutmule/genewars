"""Actual native explicit-target camera words; no startup-state inference."""
from pathlib import Path
from datetime import datetime,timezone
import importlib.util,hashlib,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
spec=importlib.util.spec_from_file_location('height_oracle',Path(__file__).with_name('terrain-review.py'));height_oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(height_oracle)
sha=lambda b:hashlib.sha256(b).hexdigest()
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=sha(b))
def expected(x,y,doubled):
 t=doubled//64
 # Explicit instruction sign/carry correction: negative signmask<<5=-32
 # with carry1; SBB adds31 before arithmetic shift5.
 q=(t+31 if t<0 else t)//32
 xhigh=(x&65535)//256;yhigh=(y&65535)//256
 # Model byte ADD then byte SUB; final AX/DX stores zero-extended bytes.
 return (xhigh-((q+12)&255))&255,(yhigh-((q+2)&255))&255

def main():
 proof_path=ROOT/'research/integration/camera-target-contract.json';proof=json.loads(proof_path.read_bytes());listings=[]
 for item in proof['instruction_contracts']:
  actual=record(ROOT/item['private_path']);assert actual['sha256']==item['sha256'];listings.append(actual)
 text=(ROOT/'private/ghidra/terrain-camera-refs/instructions-00082e20.txt').read_text()
 for term in ['00082e32 SAR EAX,0x6','00082e3a SHL EDX,0x5','00082e3d SBB EAX,EDX','00082e3f SAR EAX,0x5','00082e4b ADD AH,0xc','00082e55 SUB BL,AH','00082e5a ADD AL,0x2','00082e5c SUB DL,AL','00082e60 XOR EDX,EDX','00082e62 XOR AH,AH']:assert term in text
 work=ROOT/'private/review/camera-target';work.mkdir(parents=True,exist_ok=True)
 map_path=ROOT/'private/review/terrain/map-region.bin';region=map_path.read_bytes();assert sha(region)=='57300db747f5576c3939b95d4b0089d2347c1d525293171b866537935fdb39a6'
 coordinates=[(x*256,y*256) for y in range(128) for x in range(128)]
 coordinates.extend((x,y) for x in range(65536) for y in [0,1,255,32767,32768,65535])
 coordinates.extend(((i*40503+97)&65535,(i*26317+251)&65535) for i in range(65536))
 (work/'coords.bin').write_bytes(b''.join(struct.pack('<HH',*p) for p in coordinates))
 exe=work/'review.exe';rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe'
 library=work/'libgenewars_core.rlib'
 r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name','genewars_core','--crate-type','rlib',str(ROOT/'rust/crates/genewars-core/src/lib.rs'),'-o',str(library)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'--extern','genewars_core='+str(library),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work),str(map_path)],capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
 def height(cx,cy):return struct.unpack_from('<h',region,((cy%128)*128+cx%128)*14+2)[0]
 actual=(work/'actual-cameras.bin').read_bytes();assert len(actual)==len(coordinates)*4
 for i,(x,y) in enumerate(coordinates):
  cx,cy=x//256%128,y//256%128;heights=[height(cx,cy),height(cx+1,cy),height(cx,cy+1),height(cx+1,cy+1)]
  doubled=height_oracle.weighted(heights,x%256,y%256)
  assert struct.unpack_from('<HH',actual,i*4)==expected(x,y,doubled)
 levels=[-32768,-1025,-1024,-1023,-993,-992,-991,-1,0,991,992,1023,1024,32767]
 actual=(work/'synthetic-cameras.bin').read_bytes();assert len(actual)==len(levels)*65536*4;index=0
 for h in levels:
  for x in range(256):
   for y in range(256):
    assert struct.unpack_from('<HH',actual,index*4)==expected(x*256+0x55,y*256+0xaa,2*h);index+=1
 receipt=dict(status='passed-independent-native-explicit-camera-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
  real_target_coordinates=len(coordinates),synthetic_target_coordinates=index,constant_height_sets=len(levels),all_coordinate_high_byte_pairs_per_set=65536,
  bounds='Short/long map extents rejected',evidence=record(proof_path),private_listings=listings,source=record(ROOT/'rust/crates/genewars-core/src/terrain.rs'),harness=record(exe),
  review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['Explicit targets only; no startupcamera orrawfileplayer targetassumed','Resultwords zeroextendbytewrapping, range0..255; maplookups separaterepeatmod128','Viewport,minimap,sound/redraw sideeffects excluded','Negative correction truncatestowardzero afterarithmeticheightshift; no directfloor divide substitution'])
 (ROOT/'research/variants/camera-target-review.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:v for k,v in receipt.items() if k not in ['private_listings','source','harness','review_sources']}))

if __name__=='__main__':main()
