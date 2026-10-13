"""Independent ring-distance/weighted-height native entity point review."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
spec=importlib.util.spec_from_file_location('height_model',Path(__file__).with_name('terrain-review.py'));height_model=importlib.util.module_from_spec(spec);spec.loader.exec_module(height_model)
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def word(v):return (v+32768)%65536-32768
def ring(value,origin):return (value-origin*256+16384)%32768-16384
def main():
 source_path=ROOT/'rust/crates/genewars-core/src/terrain.rs';source_pin=record(source_path);proof_path=ROOT/'research/ghidra/entity-point-evidence.json';proof=json.loads(proof_path.read_bytes());pins=[]
 for p in proof['instruction_contracts']:
  pin=record(ROOT/p['private_path']);assert pin['sha256']==p['sha256'];pins.append(pin)
 text=(ROOT/proof['instruction_contracts'][0]['private_path']).read_text()
 for term in ['0007b070 ADD EBX,EBX','0007b072 SHL ECX,0x9','0007b075 SUB EBX,ECX','0007b083 SAR BX,0x1','0007b08b SAR AX,0x1','0007b0a8 XOR ESI,ESI','0007b0ad MOV SI,word ptr [0x0016e0c5]','0007b0c6 MOV dword ptr [ESI],EDI','0007b0c8 MOV dword ptr [EDX],ECX']:assert term in text
 work=ROOT/'private/review/entity-point';work.mkdir(parents=True,exist_ok=True);level=ROOT/'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ001.DAT';data=level.read_bytes();assert record(level)['sha256']==proof['actual_fixture']['source_sha256'];maps=[data[0x355e:0x3b55e]]
 for kind in ['flat','minimum','maximum','checker']:
  region=bytearray(229376)
  for y in range(128):
   for x in range(128):
    h=0 if kind=='flat' else -32768 if kind=='minimum' else 32767 if kind=='maximum' else (-32768 if (x+y)%2 else 32767)
    struct.pack_into('<h',region,(y*128+x)*14+2,h)
  maps.append(bytes(region))
 for i,b in enumerate(maps):(work/f'map-{i}.bin').write_bytes(b)
 points=[]
 for i in range(512):
  raw=data[0x43d4e+i*41:0x43d4e+(i+1)*41]
  if raw[6]:points.append(struct.unpack_from('<hh',raw,7))
 assert len(points)==40
 def cases():
  for y in range(128):
   for x in range(128):yield 0,word(x*256),word(y*256),66,48,128
  for x in range(65536):
   for y in [0,1,255,32767,32768,65535]:yield 0,word(x),word(y),66,48,128
  for i in range(65536):yield 0,word(i*40503+97),word(i*26317+251),66,48,128
  for x in range(65536):
   for camera in [0,1,63,64,127,128,255,256,32767,32768,65535]:
    for j,margin in enumerate([0,128,65535]):yield 1+(x+j)%4,word(x),word(x*26317+251),camera,(camera*40503)&65535,margin
  for camera in range(65536):
   for x in [-32768,-16384,-1,0,1,16383,16384,32767]:
    for margin in [0,128,65535]:yield 1+camera%4,x,word(x*40503+camera*97),camera,(camera*26317+251)&65535,margin
  for y in range(256):
   for x in range(256):yield 4,x,y,63,64,[0,128,65535][(x+y)%3]
  for x,y in points:yield 0,x,y,66,48,128
 with (work/'cases.bin').open('wb') as out:
  for case in cases():out.write(struct.pack('<HhhHHH',*case))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';lib=work/'libgenewars_core.rlib';exe=work/'review.exe'
 for args in [['--crate-name','genewars_core','--crate-type','rlib',str(ROOT/'rust/crates/genewars-core/src/lib.rs'),'-o',str(lib)],
              [str(Path(__file__).with_suffix('.rs')),'--extern','genewars_core='+str(lib),'-o',str(exe)]]:
  r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work)],capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
 count=0;bounds=[[2**31,-2**31],[2**31,-2**31]];last=[]
 native=(work/'results.bin').read_bytes()
 for count,case in enumerate(cases(),1):
  m,x,y,cx,cy,margin=case;region=maps[m];ix,iy=(x//256)%128,(y//256)%128
  heights=[struct.unpack_from('<h',region,(((iy+dy)%128)*128+(ix+dx)%128)*14+2)[0] for dx,dy in [(0,0),(1,0),(0,1),(1,1)]]
  h2=height_model.weighted(heights,x%256,y%256);dx,dy=ring(x,cx),ring(y,cy)
  expected=((dx-dy)//8-margin,(dx+dy)//16-h2//64)
  observed=struct.unpack_from('<ii',native,(count-1)*8);assert observed==expected,(case,h2,observed,expected)
  for i,n in enumerate(observed):bounds[i][0]=min(bounds[i][0],n);bounds[i][1]=max(bounds[i][1],n)
  last.append(observed)
  if len(last)>40:last.pop(0)
 assert len(native)==count*8 and bounds[0][0]<-65535
 fixture_path=ROOT/proof['actual_fixture']['private_path'];assert record(fixture_path)['sha256']==proof['actual_fixture']['sha256'];fixture=json.loads(fixture_path.read_bytes());assert [list(p) for p in last]==[p['local_point'] for p in fixture]
 assert record(source_path)==source_pin,'Source changed during review; rerun stable source'
 result=dict(status='passed-independent-native-entity-point-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,native_points=count,real_coordinate_cases=475136,active_family1_points=40,all_camera_word_values=True,all_position_word_values=True,all_fraction_pairs_on_checker=65536,explicit_margins=[0,128,65535],observed_output_bounds=bounds,invalid_map_lengths=[0,1,229375,229377],proof=record(proof_path),instruction_pins=pins,source=source_pin,harness=record(exe),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['Fixedheightfactor2/horizontalscale32/verticalscale16 source preconditions','Modulo32768 ring deltas and nonnegative barycentric height weights independent of native doubleword implementation','Unsignedmargin is not signed16; output fullsigned32 with no final narrowing','No caller visibility, sort, word-storage narrowing, entity timing or initial camera policy inferred'])
 (ROOT/'research/variants/entity-point-review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['instruction_pins','review_sources']}))
if __name__=='__main__':main()
