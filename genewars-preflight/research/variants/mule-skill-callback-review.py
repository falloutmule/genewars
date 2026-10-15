"""Synthetic bounded timer20 callbacks, exact source versus current Rust rlib."""
from pathlib import Path
import json,struct,subprocess,sys,uuid,shutil,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
def main():
 w=P/'private/review/mule-skill-callback'/uuid.uuid4().hex;w.mkdir(parents=True);origin=P/'private/review/krell-first-tick-001';inputs=bytearray();expected=bytearray();cases=[]
 for skill in [25,50]:
  for slot in [0,1]:
   for frame in [0,1]:
    d=w/f'{skill}-{slot}-{frame}';d.mkdir()
    for f in origin.glob('*.bin'):shutil.copyfile(f,d/f.name)
    c=bytearray((d/'creatures.bin').read_bytes());offset=slot*124;c[offset+0x76]=skill;struct.pack_into('<h',c,offset+0x6c,20);(d/'creatures.bin').write_bytes(c)
    proc=subprocess.run([sys.executable,str(Path(__file__).with_name('krell-mule-callback-review.py')),'--before',str(d),'--slot',str(slot),'--frame',str(frame)],capture_output=True,text=True,cwd=P,check=True);r=json.loads(proc.stdout);assert r['status']=='source_prepared',r
    source=Path(r['output']);after=(source/'creatures.bin').read_bytes()[offset:offset+124];inputs+=struct.pack('<I',frame)+c[offset:offset+124];expected+=after;cases.append({'skill':skill,'slot':slot,'frame':frame,'literal_instructions':r['literal_instructions'],'source':str(source)});assert (source/'rng.bin').read_bytes()==(d/'rng.bin').read_bytes()
 (w/'input.bin').write_bytes(inputs);(w/'expected.bin').write_bytes(expected);b=(origin/'backing.bin').read_bytes();(w/'map.bin').write_bytes(b[0x355e:0x3b55e])
 deps=P/'private/build/rust-target/debug/deps';libs=list(deps.glob('libgenewars_runtime-*.rlib'));lib=max(libs,key=lambda p:p.stat().st_mtime);pin=hashlib.sha256(lib.read_bytes()).hexdigest();exe=w/'review.exe'
 subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={deps}','--extern',f'genewars_runtime={lib}','-o',str(exe)],env=environment(),cwd=P,check=True,capture_output=True)
 proc=subprocess.run([str(exe),str(w),str(P/'private/review/mule-mortality')],cwd=P,env=environment(),capture_output=True,text=True)
 r={'status':'error','cases':cases,'output':str(w),'runtime_rlib':str(lib),'runtime_rlib_sha256':pin,'scope':'synthetic timer20; full124-byte records; source RNG unchanged; numeric API contact0 and obstaclefalse ports'}
 if proc.returncode:r['error']=proc.stderr
 else:
  actual=(w/'actual.bin').read_bytes();diff=next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None);r.update(status='passed' if actual==expected else 'error',compared_bytes=len(expected),first_difference=diff)
 (w/'result.json').write_text(json.dumps(r,indent=2)+'\n');Path(__file__).with_suffix('.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
if __name__=='__main__':main()
