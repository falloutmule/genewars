"""Exact signed-coordinate/native minimap comparison, not terrain picking."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,struct,subprocess,sys
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
spec=importlib.util.spec_from_file_location('environment_oracle',Path(__file__).with_name('actor-environment-contract.py'));oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
WORK=P/'private/review/actor-environment-minimap'
def pin(path):return dict(path=path.relative_to(P).as_posix(),sha256=hashlib.sha256(path.read_bytes()).hexdigest())
def word(v):return (v+32768)%65536-32768
def cases():
 for base in [[0,0],[127,127],[128,255],[255,128],[255,255]]:
  for origin in [[0,0],[-32768,-32768],[32767,32767],[32700,-32700],[10,20]]:
   for y in range(-1,90):
    for x in range(-1,89):yield origin,base,[word(origin[0]+x),word(origin[1]+y)]
 for raw in range(65536):
  for delta in [-1,0,43,87,88]:yield [word(raw),word(raw)],[raw&255,raw>>8],[word(raw+delta),word(raw)]
def main():
 WORK.mkdir(parents=True,exist_ok=True);source=P/'rust/crates/genewars-runtime/src/target_projection.rs';before=pin(source);exe=WORK/'review.exe';rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
 for cmd in [[str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],[str(exe),str(WORK)]]:
  r=subprocess.run(cmd,env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 counts=[0,0];digest=hashlib.sha256()
 with (WORK/'minimap-output.bin').open('rb')as native:
  for i,(origin,base,screen)in enumerate(cases()):
   r=oracle.minimap(origin,base,screen);expected=bytes(7)if r is None else b'\x01'+struct.pack('<3H',*r[0],r[1]);actual=native.read(7);assert actual==expected,(i,origin,base,screen,actual.hex(),expected.hex());digest.update(actual);counts[r is not None]+=1
  assert not native.read(1)
 assert before==pin(source)
 listing=P/'private/ghidra/landing-contact/instructions-000874b0.txt';text=listing.read_text();assert '000874dc CMP EAX,0x58'in text and '000874ed CMP EDI,0x59'in text and '000874fe SAR ECX,0x1'in text
 result=dict(status='passed-independent-native-minimap-review',utc=datetime.now(timezone.utc).isoformat(),cases=sum(counts),outside_cases=counts[0],inside_cases=counts[1],output_sha256=digest.hexdigest(),source=before,instructions=pin(listing),harness=pin(exe),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('actor-environment-contract.py'))],original_execution=False,gui_execution=False,limits=['Explicit context origin/mapbase; control0x12 minimap only','No fullscreen terrain picking or UI command dispatch'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','cases','outside_cases','inside_cases']}))
if __name__=='__main__':main()
