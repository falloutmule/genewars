"""Native borrowed mission views: every byte, all truncations and index bounds."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def main():
 source_path=ROOT/'rust/crates/genewars-assets/src/mission_source.rs';source_pin=record(source_path);proof_path=ROOT/'research/ghidra/mission-load-evidence.json';proof=json.loads(proof_path.read_bytes())
 manifest={r['entry_id']:r for r in json.loads((ROOT/'research/game-files-manifest.json').read_bytes())['entries']}
 files=[];samples=[]
 for number,entry in [(1,152),(2,153),(3,154),(24,175),(40,177)]:
  p=ROOT/f'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ{number:03}.DAT';pin=record(p);row=manifest[f'root/2/disc/{entry}'];assert pin['sha256']==row['sha256'] and pin['size']==row['size']==343958
  files.append(p);samples.append(p.read_bytes())
 work=ROOT/'private/review/mission-source';work.mkdir(parents=True,exist_ok=True);rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';lib=work/'libgenewars_assets.rlib';exe=work/'review.exe'
 for args in [['--crate-name','genewars_assets','--crate-type','rlib',str(ROOT/'rust/crates/genewars-assets/src/lib.rs'),'-o',str(lib)],
              [str(Path(__file__).with_suffix('.rs')),'--extern','genewars_assets='+str(lib),'-o',str(exe)]]:
  r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 out=work/'results.bin';r=subprocess.run([str(exe),str(out),*[str(p) for p in files]],capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
 for pattern in [bytes(343958),bytes([255])*343958,bytes((i*151+97)&255 for i in range(343958))]:
  data=bytearray(pattern);struct.pack_into('<I',data,0,0x55336);struct.pack_into('<I',data,337054,0x6be);samples.append(bytes(data))
 def integer(s):return struct.unpack('<Q',s.read(8))[0]
 all_records=all_bytes=0;group_results=[]
 with out.open('rb') as native:
  for sample_index,data in enumerate(samples):
   prefixlen=integer(native);assert prefixlen==243038 and native.read(prefixlen)==data[:prefixlen];all_bytes+=prefixlen
   assert native.read(16)==data[0x1c7e:0x1c8e]
   cursor=243038;groups=[]
   for family,g in enumerate(proof['serialized_groups']):
    observed=[integer(native) for _ in range(5)];count,stride=g['count'],g['packed_stride'];length=count*stride
    assert observed==[family,cursor,count,stride,length],(sample_index,observed)
    active=0
    for index in range(count):
     raw=data[cursor+index*stride:cursor+(index+1)*stride];size=integer(native);byte6=native.read(1)[0];observed=native.read(size)
     assert size==stride and byte6==raw[6] and observed==raw,(sample_index,family,index)
     if family==1:
      assert integer(native)==index
      assert native.read(4)==raw[7:11] and native.read(3)==bytes([raw[0x19],raw[0x1b],raw[0x23]])
     all_records+=1;all_bytes+=size;active+=byte6!=0
    groups.append(dict(family=family,records=count,active_byte6=active));cursor+=length
   assert cursor==336998
   assert native.read(52)==data[cursor:cursor+52];assert native.read(2)==bytes([data[cursor],data[cursor+28]])
   assert native.read(4)==data[337050:337054] and native.read(4)==data[337054:337058]
   length=integer(native);assert length==6900 and native.read(length)==data[337058:]
   all_bytes+=52+4+4+length;assert all_bytes==(sample_index+1)*343958
   group_results.append(groups)
  assert not native.read(1)
 assert record(source_path)==source_pin,'Source changed during review; rerun stable input'
 result=dict(status='passed-independent-native-mission-source-parser-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,real_sources=len(files),opaque_synthetic_sources=3,raw_records_compared=all_records,all_source_bytes_compared=all_bytes,family1_field_records_compared=len(samples)*512,all_truncation_lengths=343958,unsupported_long_lengths=[343959,375830],individual_tag_bitflips=64,index_bounds='All target/group/count/usizeMAX record accesses reject; borrowed slice pointers checked against source offsets',source_immutability=True,source=source_pin,proof=record(proof_path),inputs=[record(p) for p in files],groups_by_sample=group_results,harness=record(exe),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['Exact layout/version extent plus tags, not hash-only admission or generic all-variant parser','Record byte6 and flags/count bytes remain raw; no active-count policy or entity meaning invented','No native runtime gaps/pointers filled and no live-state reconstruction','Other same-name resource sizes are unsupported variants, not corruption claims'])
 (ROOT/'research/variants/mission-source-review.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ['inputs','groups_by_sample','review_sources']}))
if __name__=='__main__':main()
