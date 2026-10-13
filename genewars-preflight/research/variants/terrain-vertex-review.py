"""Independently check native UV/height/parity strip-equivalent primitives."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
sha=lambda b:hashlib.sha256(b).hexdigest()
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=sha(b))

def main():
 work=ROOT/'private/review/terrain-vertex';work.mkdir(parents=True,exist_ok=True)
 proof=json.loads((ROOT/'research/ghidra/terrain-evidence.json').read_bytes())
 projection_path=ROOT/'research/ghidra/terrain-projection-evidence.json'
 projection=json.loads(projection_path.read_bytes())
 assert projection['candidate_sha256']==proof['candidate_sha256']
 for item in projection['instruction_contracts']:
  assert record(ROOT/item['private_path'])['sha256']==item['sha256']
 assert projection['triangle_binding']['indices']==[[0,2,3],[0,1,2]]
 listing=ROOT/'private/ghidra/terrain-map-refs/instructions-0007b7f0.txt';text=listing.read_text()
 for line in ['0007b841 SAR EAX,0x1','0007b843 ADD EDI,EAX','0007b858 SUB EDI,EAX','0007b863 JNZ 0x0007b88a','0007b885 LEA EDI,[EAX + -0x1]','0007b89f SUB EDI,0x2','0007b8fb SAR EAX,0x6','0007b8fe IMUL EDX,EAX','0007b905 SUB EAX,EDX','0007bb8e CALL 0x000e1900','0007bd9d CALL 0x000e1900']:assert line in text
 expected=[];cases=[]
 for column in range(-100,101):
  n=column//2;p=column&1
  for cam in [(0,0),(127,127),(-128,-128),(10000,-20000)]:
   a=(cam[0]+n,cam[1]-n-1-p)
   points=[a,(a[0],a[1]+1),(a[0]+1,a[1]+1),(a[0]+1,a[1]+2)]
   for top in [-100,0,100]:
    for anchor in [(0,0),(320,240)]:
     for h in [-32768,-65,-64,-1,0,63,64,32767]:
      # Independently stepped strip A,B,C,D, not the API delta expression.
      for i,world in enumerate(points):
       sx=32*(column+1 if i in [0,2] else column)
       sy=-16*(1+p)+16*i-top-2*(h//64)
       cases.append((*world,*cam,h,top,*anchor));expected.append((1,sx+anchor[0],sy+anchor[1]))
 boundary=[((0,0,0,0,0,0,2147483647,2147483647),(1,2147483647,2147483647)),
  ((1,0,0,0,0,0,2147483647,0),(0,0,0)),
  ((2147483647,0,-2147483648,0,0,0,0,0),(0,0,0)),
  ((0,0,0,0,-1,-2147483648,0,0),(0,0,0)),
  ((0,0,0,0,0,0,-2147483648,-2147483648),(1,-2147483648,-2147483648)),
  ((0,1,0,0,0,0,-2147483648,0),(0,0,0))]
 for case,result in boundary:cases.append(case);expected.append(result)
 for row in projection['vertex_projection']['vectors']:
  cases.append((*row['world_cell'],*row['camera_cell'],row['height_i16'],row['vertical_top'],*row['caller_anchor']))
  expected.append((1,*row['position']))
 (work/'cases.bin').write_bytes(b''.join(struct.pack('<8i',*x) for x in cases))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=work/'review.exe'
 r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work)],capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
 actual=(work/'positions.bin').read_bytes();assert len(actual)==len(cases)*9
 for i,wanted in enumerate(expected):assert struct.unpack_from('<Bii',actual,i*9)==wanted,(cases[i],wanted,struct.unpack_from('<Bii',actual,i*9))
 actual=(work/'displacements.bin').read_bytes();assert len(actual)==65536*4
 for i,h in enumerate(range(-32768,32768)):assert struct.unpack_from('<i',actual,i*4)[0]==2*(h//64)
 mapped=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes();uv=mapped[0x60ec:0x60ec+8*32];assert len(uv)==256
 for row in range(8):assert list(struct.unpack_from('<8I',uv,row*32))==proof['uv_orientation_table']['corners_u16_16'][row]
 actual=(work/'uv-flags.bin').read_bytes();assert len(actual)==256*32
 for flags in range(256):assert actual[flags*32:(flags+1)*32]==uv[((flags&0x70)>>4)*32:(((flags&0x70)>>4)+1)*32]
 receipt=dict(status='passed-independent-native-terrain-vertex-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
  height_values=65536,cell_flag_values=256,orientation_admission_values=256,strip_vertex_positions=len(cases)-len(boundary)-len(projection['vertex_projection']['vectors']),overflow_boundary_cases=len(boundary),projection_receipt_vectors=len(projection['vertex_projection']['vectors']),
  formula_scope='Equivalent to7B7F0strip A/B/C/D for explicit unwrappedcells/camera/placement; not7AF00pointprojection',
  proof=record(ROOT/'research/ghidra/terrain-evidence.json'),projection_proof=record(projection_path),strip_listing=record(listing),source=record(ROOT/'rust/crates/genewars-render/src/terrain.rs'),harness=record(exe),
  review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['Checked intermediateoverflow rejects deliberate safeport inputs instead of original32bitwrap','No initialcamera or fullraster/clip/shade/opacity behavior inferred','Triangles are firstA,C,B forcellA andsecondB,C,D foradjacentcellB; not two successivecalls ononefixedcell'])
 (ROOT/'research/variants/terrain-vertex-review.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:v for k,v in receipt.items() if k not in ['proof','strip_listing','source','harness','review_sources']}))

if __name__=='__main__':main()
