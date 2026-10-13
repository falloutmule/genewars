"""Independent exact source tail accounting and gated target/camera review."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def fill_write_positions(address,count):
 # Literal stores of B6930/B6967 expressed as byte positions. The read-only
 # CMP prefetches are outside this write-extent model.
 stores=[]
 def store(at,n):stores.extend(range(at,at+n))
 while count and address%4:store(address,1);address+=1;count-=1
 dwords=count//4;tail=count%4
 while dwords and address%32:store(address,4);address+=4;dwords-=1
 groups,remainder=divmod(dwords,4)
 if groups:
  groups-=1
  if groups:
   while True:
    store(address,16);groups-=1
    if not groups:address+=16;break
    store(address+16,16);groups-=1;address+=32
    if not groups:break
  store(address,16);address+=16
 for _ in range(remainder):store(address,4);address+=4
 store(address,tail)
 return stores
def main():
 proof_path=ROOT/'research/ghidra/mission-load-evidence.json';proof=json.loads(proof_path.read_bytes());pins=[]
 for p in proof['instruction_contracts']:
  pin=record(ROOT/p['private_path']);assert pin['sha256']==p['sha256'];pins.append(pin)
 helper_names=['000982c0','000b6930','000b6967']
 for name in helper_names:assert any(p['path'].endswith('instructions-'+name+'.txt') for p in pins)
 helper_text={n:(ROOT/'private/ghidra/mission-tail-readers'/('instructions-'+n+'.txt')).read_text() for n in helper_names}
 for n,terms in [('000982c0',['MOV ECX,EBX','CALL 0x000b6930']),('000b6930',['000b693a MOV byte ptr [EAX],DL','000b6944 SHR ECX,0x2','000b6947 CALL 0x000b6967','000b694d AND ECX,0x3']),('000b6967',['000b696b TEST AL,0x1f','000b6978 SHR ECX,0x2','000b69a9 MOV dword ptr [EAX],EDX','000b69b8 AND ECX,0x3'])]:
  for term in terms:assert term in helper_text[n]
 for alignment in range(32):
  for length in range(256):assert fill_write_positions(alignment,length)==list(range(alignment,alignment+length)),(alignment,length)
 assert fill_write_positions(0x94,0x2c)==list(range(0x94,0xc0))
 source=ROOT/'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ001.DAT';data=source.read_bytes();assert record(source)['sha256']==proof['source']['sha256'] and len(data)==343958
 obj3=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes()
 def word(a):return struct.unpack_from('<I',obj3,a-0x100000)[0]
 groups=proof['serialized_groups'];counts=[300,512,20,256,200,32,32,16];native_sizes=[124,49,200,67,81,49,97,102];cursor=0x3b55e;native_cursor=cursor;record_count=active=0;regions=[]
 for g,count,native_size in zip(groups,counts,native_sizes):
  reader=ROOT/('private/ghidra/mission-tail-readers/instructions-'+g['reader'][2:].zfill(8)+'.txt');text=reader.read_text()
  # The only payload MOV ECX follows the shared31-byte reader; this is
  # independently read from each direct listing, not the receipt stride.
  payload=int(re.search(r'CALL 0x0006d490\r?\n[0-9a-f]+ MOV ECX,0x([0-9a-f]+)',text).group(1),16)
  packed=31+payload;end=cursor+count*packed
  assert g['count']==count and g['native_stride']==native_size and g['packed_stride']==packed
  assert g['file_offset']==cursor and g['file_end_exclusive']==end and g['native_state_offset']==native_cursor
  assert word(int(g['constructor_descriptor'],16)+13)==native_size
  assert word(int(g['constructor_descriptor'],16)+1)==int(g['constructor'],16)
  assert word(int(g['vtable'],16)+0x3c)==int(g['reader'],16) and word(int(g['vtable'],16)+0x60)==int(g['active_init'],16)
  end_payload=39+payload;assert end_payload<=native_size
  active_group=0;untouched_ranges=[[31,39]]+([[end_payload,native_size]] if end_payload<native_size else [])
  aggregate=hashlib.sha256()
  for index in range(count):
   raw=data[cursor+index*packed:cursor+(index+1)*packed];assert len(raw)==packed
   reconstructed=bytearray([165])*native_size;reconstructed[:31]=raw[:31];reconstructed[39:end_payload]=raw[31:]
   assert reconstructed[:31]==raw[:31] and reconstructed[39:end_payload]==raw[31:]
   for lo,hi in untouched_ranges:assert reconstructed[lo:hi]==bytes([165])*(hi-lo)
   # Reader changes staging header only AFTER copying the original header.
   staging=bytearray(raw);staging[23]=32;staging[21]&=227
   assert reconstructed[:31]==raw[:31] and (staging[21]&28)==0 and staging[23]==32
   active_group+=raw[6]!=0;aggregate.update(reconstructed)
  assert active_group==g['active_raw_byte_nonzero']
  regions.append(dict(family=g['family'],count=count,packed=packed,native=native_size,source_start=cursor,source_end=end,active=active_group,unserialized_ranges=untouched_ranges,synthetic_reconstruction_sha256=aggregate.hexdigest()))
  cursor=end;native_cursor+=count*native_size;record_count+=count;active+=active_group
 assert cursor==0x52466 and native_cursor==0x55336 and record_count==1368
 globals=data[cursor:cursor+52];assert globals[0]==56 and globals[28]==1 and globals[0]&32
 viewport=cursor+52;extension=viewport+4;tag=struct.unpack_from('<I',data,extension)[0]
 assert tag==0x6be and extension+4+6900==len(data)
 table=struct.unpack_from('<4I',data,0x1c7e);assert table==(0x35005100,0,0,0)
 for player in range(4):
  start=20+player*0x70d;assert data[start+9:start+13]==bytes(4)
  assert start+0x4a1<=0x1c7e and start+0x61a+0xdd<=0x1c7e
 loop=(ROOT/'private/ghidra/terrain-camera-refs/instructions-0007d090.txt').read_text()
 for term in ['0007d551 JNZ 0x0007d55d','0007d57a JMP 0x0007d6af','0007d6bb JLE 0x0007d76a','0007d6e5 MOV EAX,dword ptr [EAX + 0x11a9de]','0007d6eb MOV dword ptr [ECX + 0x9],EAX','0007d75a JZ 0x0007d57f','0007d765 JMP 0x0007d589','0007d599 CMP EAX,EBX','0007d5a2 MOV EDX,dword ptr [ECX + 0x9]','0007d5a5 CALL 0x00082e20']:assert term in loop
 work=ROOT/'private/review/mission-load';work.mkdir(parents=True,exist_ok=True);rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';lib=work/'libgenewars_core.rlib';exe=work/'review.exe'
 for args in [['--crate-name','genewars_core','--crate-type','rlib',str(ROOT/'rust/crates/genewars-core/src/lib.rs'),'-o',str(lib)],
              [str(Path(__file__).with_suffix('.rs')),'--extern','genewars_core='+str(lib),'-o',str(exe)]]:
  r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 native_path=work/'targets.bin';r=subprocess.run([str(exe),str(source),str(native_path)],capture_output=True,timeout=20);assert r.returncode==0,r.stderr.decode(errors='replace')
 native=list(struct.iter_unpack('<iHH',native_path.read_bytes()));assert len(native)==4;targets=[]
 for value,observed in zip(table,native):
  x,y=value&65535,value>>16;cx,cy=x//256,y//256;assert x%256==0 and y%256==0
  height=struct.unpack_from('<h',data,0x355e+((cy%128)*128+cx%128)*14+2)[0];h2=2*height;t=h2//64;q=(-1 if t<0 else 1)*(abs(t)//32)
  expected=(h2,(cx-q-12)%256,(cy-q-2)%256);assert observed==expected
  targets.append(dict(packed=value,coordinate=[x,y],height2=h2,camera=list(observed[1:])))
 assert targets[0]['coordinate']==[20736,13568] and targets[0]['height2']==6272 and targets[0]['camera']==[66,48]
 result=dict(status='passed-independent-source-mission-load-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,source=record(source),proof=record(proof_path),instruction_pins=pins,raw_records=record_count,active_init_records=active,groups=regions,globals_offset=cursor,exact_file_end=len(data),targets=targets,native_helper=record(ROOT/'rust/crates/genewars-core/src/terrain.rs'),harness=record(exe),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],memset_write_extent_checks=8192,initializer_memset_native_extent=[148,192],
  scope='Exactsource structure and gated source-derived player0 camera; not mission execution or complete native-state reconstruction',
  limits=['Successful exact load, activecount1 andselectedplayer0 required; do not claim selection policy or observed original camera','Campaignflag20 bypasses target permutation; loop writes player+9 before the selected-camera call','Native pointers and family2 runtime-only bytes are not serialized; synthetic reconstruction is a copy-layout test only','Transitive982C0/B6930/B6967 fill writes only entity94..C0 for this caller; no complete native-state or allocation reconstruction claimed'])
 (ROOT/'research/variants/mission-load-review.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items() if k not in ['instruction_pins','groups','review_sources']}))
if __name__=='__main__':main()
