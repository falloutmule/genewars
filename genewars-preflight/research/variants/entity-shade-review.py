"""Independent weighted integer oracle for entity shade; all vertex operands."""
from pathlib import Path
from datetime import datetime,timezone
import collections,hashlib,json,random,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def expected(products,x,y):
 # Nonnegative triangle weights sum256. Merge both final floor shifts and
 # integer bias into one division by32768 minus14, then clamp0..64.
 weights=(256-y,0,y-x,x) if x<y else (256-x,x-y,0,y)
 total=sum(p*w for p,w in zip(products,weights))
 return min(64,max(0,total//32768-14))
def main():
 source_path=ROOT/'rust/crates/genewars-core/src/terrain.rs';source_pin=record(source_path)
 proof_path=ROOT/'research/integration/entity-shade-contract.json';proof=json.loads(proof_path.read_bytes());listing=ROOT/proof['listing'];assert record(listing)['sha256']==proof['listing_sha256']
 text=listing.read_text()
 for term in ['0007b28a XOR EBX,ECX','0007b294 IMUL EBX,EAX','0007b2a1 SAR EBX,0x6','0007b2e6 JL 0x0007b325','0007b31e SAR EAX,0x7','0007b358 SUB EAX,0x1e00','0007b363 CMP EDX,0x40','0007b366 JL 0x0007b36d','0007b373 XOR EAX,EDX']:assert term in text
 work=ROOT/'private/review/entity-shade';work.mkdir(parents=True,exist_ok=True)
 source=ROOT/'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ001.DAT';assert record(source)['sha256']=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
 region=source.read_bytes()[0x355e:0x3b55e];(work/'map-0.bin').write_bytes(region)
 coordinates=[(x*256,y*256) for y in range(128) for x in range(128)]
 coordinates.extend((x,y) for x in range(65536) for y in [0,1,255,32767,32768,65535])
 coordinates.extend(((i*40503+97)&65535,(i*26317+251)&65535) for i in range(65536))
 (work/'coords.bin').write_bytes(b''.join(struct.pack('<HH',*p) for p in coordinates))
 patterns=[([3840,8192,4097,16384],[64,63,63,63]),([-32768,32767,32767,-32768],[255]*4),([32767,-32768,-32768,32767],[255,255,0,1]),([-1,-63,-64,-65],[1,63,64,255]),([1919,1920,9983,9984],[64]*4)]
 rng=random.Random(0x7b240)
 patterns.extend(([rng.randrange(-32768,32768) for _ in range(4)],[rng.randrange(256) for _ in range(4)]) for _ in range(15))
 for i,(lights,factors) in enumerate(patterns,1):
  data=bytearray(229376)
  for y in range(128):
   for x in range(128):
    j=(y%2)*2+x%2;off=(y*128+x)*14;struct.pack_into('<h',data,off,lights[j]);data[off+12]=factors[j]
  (work/f'map-{i}.bin').write_bytes(data)
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';lib=work/'libgenewars_core.rlib';exe=work/'review.exe'
 for args in [['--crate-name','genewars_core','--crate-type','rlib',str(ROOT/'rust/crates/genewars-core/src/lib.rs'),'-o',str(lib)],
              [str(Path(__file__).with_suffix('.rs')),'--extern','genewars_core='+str(lib),'-o',str(exe)]]:
  r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work),str(len(patterns))],capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 actual=(work/'actual.bin').read_bytes();assert len(actual)==len(coordinates)
 def product(cx,cy):
  off=((cy%128)*128+cx%128)*14;return struct.unpack_from('<h',region,off)[0]*region[off+12]//64
 for observed,(x,y) in zip(actual,coordinates):
  cx,cy=x//256,y//256;p=[product(cx,cy),product(cx+1,cy),product(cx,cy+1),product(cx+1,cy+1)]
  assert observed==expected(p,x%256,y%256)
 actual=(work/'fractions.bin').read_bytes();assert len(actual)==len(patterns)*65536;hist=collections.Counter(actual);index=0
 for lights,factors in patterns:
  p=[l*f//64 for l,f in zip(lights,factors)]
  for y in range(256):
   for x in range(256):assert actual[index]==expected(p,x,y),(lights,factors,x,y,actual[index]);index+=1
 actual=(work/'vertex.bin').read_bytes();assert len(actual)==16777216;hist.update(actual);i=0
 for light in range(-32768,32768):
  for factor in range(256):
   assert actual[i]==min(64,max(0,((light*factor)//64)//128-14));i+=1
 assert set(hist)==set(range(65))
 assert record(source_path)==source_pin,'Source changed during review; rerun against stable input'
 result=dict(status='passed-independent-native-entity-shade-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,actual_coordinates=len(coordinates),synthetic_fraction_cases=index,quad_patterns=len(patterns),all_signed16_unsigned8_vertex_products=i,observed_result_domain=[min(hist),max(hist)],result_histogram=dict(hist),evidence=record(proof_path),listing=record(listing),source=source_pin,harness=record(exe),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['Explicit map/coordinates only; no entity mutation or visibility callbacks','Vertex product floors before interpolation; 64 endpoint intentionally differs from FADE row admission','Coordinate seams wrap128 independently of 16bit fraction extraction','Short/long map regions rejected'])
 (ROOT/'research/variants/entity-shade-review.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ['result_histogram','review_sources']}))
if __name__=='__main__':main()
