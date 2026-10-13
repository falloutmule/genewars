"""Independent full-cell/tile and barycentric terrain-height native review."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
sha=lambda b:hashlib.sha256(b).hexdigest()
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=sha(b))

def weighted(heights,x,y):
 a,b,c,d=heights
 if x<y:weights=(256-y,0,y-x,x)
 else:weights=(256-x,x-y,0,y)
 assert min(weights)>=0 and sum(weights)==256
 return sum(h*w for h,w in zip(heights,weights))//128

def main():
 proof_path=ROOT/'research/ghidra/terrain-evidence.json';proof=json.loads(proof_path.read_bytes());listings=[]
 for item in proof['instruction_contracts']:
  actual=record(ROOT/item['private_path']);assert actual['sha256']==item['sha256'];listings.append(actual)
 assert sha((ROOT/'private/ghidra/GW-CD-candidate.exe').read_bytes())==proof['candidate_sha256']
 text=(ROOT/'private/ghidra/terrain-map-refs/instructions-0007b140.txt').read_text()
 for term in ['0007b152 SAR EBP,0x8','0007b155 SAR ESI,0x8','0007b18a XOR AH,AH','0007b1d7 XOR DH,DH','0007b1e8 JL 0x0007b209','0007b1fe IMUL EAX,EBX','0007b204 IMUL EDX,ESI','0007b220 IMUL EAX,EBX','0007b225 SAR EAX,0x7','0007b228 ADD EAX,EBP']:assert term in text
 work=ROOT/'private/review/terrain';work.mkdir(parents=True,exist_ok=True)
 source=ROOT/'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ001.DAT';level=source.read_bytes()
 assert sha(level)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
 region=level[0x355e:0x3b55e];assert len(region)==229376 and sha(region)==proof['map']['region_sha256']
 map_path=work/'map-region.bin';map_path.write_bytes(region)
 corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results'];entry=next(r for r in corpus if r['entry_id']=='root/2/disc/40' and r['status']=='passed')
 atlas_path=ROOT/entry['private_output'];atlas=atlas_path.read_bytes();assert len(atlas)==163840 and sha(atlas)==entry['output_sha256']
 coordinates=[]
 # All real cell vertices, all signed16 x values at six y boundaries, plus
 # deterministic mixed-axis samples to exercise non-axis-aligned triangles.
 coordinates.extend((x*256,y*256) for y in range(128) for x in range(128))
 coordinates.extend((x,y) for x in range(65536) for y in [0,1,255,32767,32768,65535])
 coordinates.extend(((i*40503+97)&65535,(i*26317+251)&65535) for i in range(65536))
 (work/'coordinates.bin').write_bytes(b''.join(struct.pack('<HH',*c) for c in coordinates))
 env=environment();rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=work/'terrain-review.exe'
 for name in ['assets','core']:
  result=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name','genewars_'+name,'--crate-type','rlib',str(ROOT/f'rust/crates/genewars-{name}/src/lib.rs'),'-o',str(work/f'libgenewars_{name}.rlib')],capture_output=True,env=env,timeout=90)
  assert result.returncode==0,result.stderr.decode(errors='replace')
 result=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'--extern','genewars_assets='+str(work/'libgenewars_assets.rlib'),'--extern','genewars_core='+str(work/'libgenewars_core.rlib'),'-L','dependency='+str(work),'-o',str(exe)],capture_output=True,env=env,timeout=90)
 assert result.returncode==0,result.stderr.decode(errors='replace')
 result=subprocess.run([str(exe),str(work),str(map_path),str(atlas_path)],capture_output=True,timeout=60);assert result.returncode==0,result.stderr.decode(errors='replace')
 native=(work/'cells.bin').read_bytes();assert len(native)==16384*17
 for i in range(16384):
  h,selector=struct.unpack_from('<hB',native,i*17);raw=native[i*17+3:(i+1)*17]
  assert raw==region[i*14:(i+1)*14] and h==struct.unpack_from('<h',raw,2)[0] and selector==raw[4]
 tiles=(work/'tiles.bin').read_bytes();assert len(tiles)==len(atlas);coverage=bytearray(len(atlas));tile_hashes=[]
 for t in range(160):
  expected=bytearray()
  for v in range(32):
   for u in range(32):
    address=((t//8)*32+v)*256+(t%8)*32+u;coverage[address]+=1;expected.append(atlas[address])
  assert tiles[t*1024:(t+1)*1024]==expected
  tile_hashes.append(dict(tile=t,sha256=sha(expected)))
 assert min(coverage)==max(coverage)==1
 def height(x,y):return struct.unpack_from('<h',region,((y%128)*128+(x%128))*14+2)[0]
 native=(work/'heights.bin').read_bytes();assert len(native)==4*len(coordinates)
 for i,(x,y) in enumerate(coordinates):
  cx,cy=x//256%128,y//256%128;h=[height(cx,cy),height(cx+1,cy),height(cx,cy+1),height(cx+1,cy+1)]
  expected=weighted(h,x%256,y%256);assert struct.unpack_from('<i',native,i*4)[0]==expected,(x,y,h,expected)
 sets=[(0,0,0,0),(7,7,7,7),(0,256,512,768),(-32768,32767,-32768,32767),(32767,-32768,32767,-32768),(-5,8,-13,21),(-32768,32767,32767,-32768),(32767,-32768,-32768,32767)]
 native=(work/'synthetic-heights.bin').read_bytes();assert len(native)==len(sets)*65536*4;index=0
 for h in sets:
  for x in range(256):
   for y in range(256):
    expected=weighted(h,x,y);assert -65536<=expected<=65534 and struct.unpack_from('<i',native,index*4)[0]==expected;index+=1
 receipt=dict(status='passed-independent-native-terrain-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
  real_cells=16384,raw_cell_bytes=229376,atlas_tiles=160,atlas_indices=163840,atlas_source_coverage='each byte exactly once',
  real_height_samples=len(coordinates),synthetic_height_samples=index,extrema_height_sets=len(sets),tile_hashes=tile_hashes,
  invalid_cases=['Maplength short/long','Cellcoordinate128/usizeMAX','Atlaslength short/long','Tile160/usizeMAX','Interpolationregion short/long'],
  integer_bound=dict(max_abs_difference=65535,max_fraction=255,max_abs_two_product_sum=2*65535*255,result_min=-65536,result_max=65534),
  evidence=record(proof_path),private_listings=listings,inputs=[record(source),record(atlas_path)],harness=record(exe),
  sources=[record(ROOT/'rust/crates/genewars-assets/src/map_atlas.rs'),record(ROOT/'rust/crates/genewars-core/src/terrain.rs'),record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['No complete triangle rasterization/shading/camera/frame parity inferred','Map exposes initial serialized bytes; later live-state mutations are not modeled','Atlas raw indices assign no transparency; mode5 zero-write remains explicit static evidence','Doubled interpolation result differs from separately rounded vertex projection'])
 (ROOT/'research/variants/terrain-review.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:v for k,v in receipt.items() if k not in ['tile_hashes','private_listings','sources','inputs','harness']}))

if __name__=='__main__':main()
