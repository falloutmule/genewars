"""Exact CD Genepod binding/layout and independent component/pixel oracle.

Metadata-only by default; --test runs only source_building_resources unit tests.
"""
from pathlib import Path
import hashlib,json,struct,sys,subprocess,os,uuid
from validate_bturr_status_overlay_pixels import sprite_from_table
import validate_source_building_pixels as reference
P=Path(__file__).resolve().parents[2]
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 source=P/'private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BGENE.ANB';data=source.read_bytes()
 digest='a71f03d3cce3d36798cd7f8ad578d665e65883bfe9b541ba93d0ad9eff814cab'
 assert len(data)==62062 and sha(data)==digest
 manifest=json.loads((P/'research/game-files-manifest.json').read_text())['entries'];entry=next(e for e in manifest if e['entry_id']=='root/2/disc/301')
 assert entry['sha256']==digest and entry['size']==62062
 mapped=P/'private/ghidra/validated/memory-.object3.bin';m=mapped.read_bytes();assert sha(m)=='aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87'
 assert m[0x393c:0x3958].split(b'\0')[0]==b'data/building/BGENE.ANB' and struct.unpack_from('<I',m,0x3958)[0]==0x116824
 callback=P/'private/ghidra/building-loader-refs/instructions-000451f0.txt';listing=callback.read_text()
 for line in ['00047f0b MOV AL,byte ptr [EAX + 0x2e]','00047f11 INC AL','00047f13 MOV byte ptr [EDX + 0x19],AL','00047f1b MOV byte ptr [EAX + 0x19],0x0','00047efc MOV byte ptr [EAX + 0x19],0x4']:assert line in listing
 for rel,digest_pin in reference.reference.LISTINGS.items():reference.reference.check(P/rel,digest_pin)
 header,*offsets=struct.unpack_from('<H4I',data);assert (header,offsets)==(5,[61038,61128,61648,62012])
 output=bytearray();animations=[];pixels=[];reference.reference.decode_frame=reference.decode_frame
 for a in range(5):
  first,last,count=struct.unpack_from('<IIH',data,offsets[3]+a*10);assert last==first+(count-1)*14
  animations.append(dict(index=a,first_frame_offset=first,last_frame_offset=last,count=count,holds=[data[offsets[2]+at+10]for at in range(first,last+1,14)]))
  if a>1:continue
  for index,at in enumerate(range(first,last+1,14)):
   raw=data[offsets[2]+at:offsets[2]+at+14];start=struct.unpack_from('<I',raw)[0];width,height=struct.unpack_from('<hh',raw,4);hold,n=raw[10:12]
   output.extend(struct.pack('<BBBBBhh',0,a,index,hold,n,width,height))
   for ci in range(n):
    c=data[offsets[1]+start+ci*10:offsets[1]+start+(ci+1)*10];output.extend(c);sp=sprite_from_table(data,offsets,struct.unpack_from('<I',c)[0]//6);output.extend(bytes(sp[:2]));output.extend(sp[2]);output.extend(sp[3])
   for origin in [[320,240],[1,1]]:
    case=dict(animation=a,frame=index,owner=0,shade=30,actor_flags=0,global_flags=0,origin=origin,phase=0,building_flags=0)
    result=reference.reference.oracle(data,b'',b'',case);pixels.append(dict(animation=a,frame=index,origin=origin,sha256=result['sha256']))
 assert len(output)==117413 and sha(output)=='2d00cf1abdd1c30f1aca3e113a65f4997f9691f767f548fbc8a95ccd555c2e62'
 test='not run';private_run=None
 if '--test' in sys.argv:
  work=P/'private/integration/genepod-resources'/uuid.uuid4().hex;work.mkdir(parents=True);private_run=work.relative_to(P).as_posix()
  run=subprocess.run([sys.executable,'-B','scripts/cargo.py','test','-p','genewars-runtime','source_building_resources::tests','--offline','--locked'],cwd=P,env=dict(os.environ,CARGO_BUILD_JOBS='2'),capture_output=True,text=True,timeout=120)
  (work/'tests.txt').write_text(run.stdout+run.stderr);assert run.returncode==0,(run.stdout,run.stderr)
  test=next(line for line in run.stdout.splitlines()if line.startswith('test result:'))
 pins=[source,mapped,callback,P/'research/ghidra/building-loader-contract.json',P/'rust/crates/genewars-runtime/src/source_building_resources.rs',Path(__file__),P/'scripts/desktop/validate_source_building_pixels.py',P/'scripts/desktop/validate_bturr_common_pixels.py',P/'scripts/desktop/validate_bturr_status_overlay_pixels.py']
 receipt=dict(status='passed'if test!='not run'else'passed-static-metadata; rust validation not run',selector=0,resource='BGENE.ANB',entry_id=entry['entry_id'],bytes=len(data),sha256=sha(data),mapped_descriptor='0x10393c',destination='0x116824',relocated_selector_table='0x11685c',section_offsets=offsets,sprite_records=15,components=52,frames=26,animations=animations,admitted_animations=[0,1],grade=0,metadata_comparison_bytes=len(output),metadata_oracle_sha256=sha(output),framebuffer_oracle=pixels,framebuffer_bytes_per_case=311032,rust_tests=test,private_run=private_run,pins={p.relative_to(P).as_posix():sha(p.read_bytes())for p in pins},catalog_row=f'buildings/BGENE.ANB\t{sha(data)}\t62062\tprivate/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BGENE.ANB\troot/2/disc/301',limits=['Exact resource and common main-image admission only; no Genepod production callback or campaign acceptance','Incomplete animation0 has3 construction-grade frames; normal completed grade0 selects animation1 through47F08..13','Animations2/3 upgraded grades and animation4 special flag40 state remain unadmitted','No shadow, overlay, production animation sequencing or sound claim'],original_execution=False)
 (P/'research/formats/genepod-resources.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k]for k in ['status','selector','bytes','sha256','rust_tests','catalog_row']}))
if __name__=='__main__':main()
