"""Captured marker descent/wait: literal118E0 comparison with narrow service ports."""
from pathlib import Path
import argparse,importlib.util,struct,json,hashlib,sys,subprocess,uuid,shutil
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('f8',Path(__file__).with_name('family8-state4-source.py'));m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
def sha(f):return hashlib.sha256(f.read_bytes()).hexdigest()
def prepare(W):
 D=P/'private/review/krell-zone-bootstrap-001/after';b=(D/'backing.bin').read_bytes();mp=b[0x355e:0x3b55e];(W/'map.bin').write_bytes(mp);state={0x118d60+i:v for i,v in enumerate(b[:0x355e])}
 for name,base in [('actors',0x16360e),('creatures',0x1542be),('buildings',0x1645ae),('effects',0x1688ae),('family8',m.BASE),('players',0x118d74)]:state.update({base+i:v for i,v in enumerate((D/(name+'.bin')).read_bytes())})
 for base,width,count,vt in [(0x1542be,124,300,0x111fec),(0x16360e,200,20,0x112194),(0x1688ae,81,200,0x112254),(m.BASE,102,16,0x11242c)]:
  for i in range(count):state.update({base+i*width+35+j:v for j,v in enumerate(struct.pack('<I',vt))})
 for i in range(512):
  raw=b[0x43d4e+i*41:0x43d4e+(i+1)*41];native=bytearray(raw[:31]+bytes(8)+raw[31:]);struct.pack_into('<I',native,35,0x1120ac);state.update({0x15d40e+i*49+j:v for j,v in enumerate(native)})
 model=m.Model();files=[P/'private/ghidra'/f for f in ['initial-creature/instructions-0006d230.txt','terrain-map-refs/instructions-000305f0.txt','countdown-expiry/instructions-0001ea80.txt']];model.files+=files;model.code.update(m.m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
 a=(D/'family8.bin').read_bytes()[:102];rng=struct.unpack('<HBI',(D/'rng.bin').read_bytes());inputs=bytearray();expected=bytearray();timeline=[];steps=0;waits=0;laststate=None
 def one(a,frame,state):
  nonlocal steps
  after,nrng,n,req,trace=model.step(a,mp,frame,rng,extra_state=state);steps+=n
  assert nrng==rng and not req['child'] and not req['particle'];assert bytes(model.load(0x11c2be+i,1)for i in range(229376))==mp
  inputs.extend(a+struct.pack('<IHBI',frame,*rng));expected.extend(after+struct.pack('<HBI',*nrng)+bytes([req['moving_audio'],req['query_stop_audio'],0x83140 in model.calls]))
  timeline.append({'frame':frame,'state':after[0x56],'timer':struct.unpack_from('<h',after,0x64)[0],'height':struct.unpack_from('<i',after,0x27)[0],'idle_search':0x305f0 in trace})
  return after,model.saved_state
 for frame in range(1,501):
  a,state=one(a,frame,state)
  if a[0x56]==12:
   waits+=1
   if waits==40:break
 else:raise AssertionError('marker did not reach40waitcallbacks')
 assert {t['state']for t in timeline}>={10,11,12}
 for timer in [19,20,32767,-1]:
  q=bytearray(a);q[0x56]=11;struct.pack_into('<h',q,0x64,timer);one(bytes(q),503,state)
 q=bytearray(a);q[0x56]=12;q[0x57]=1;one(bytes(q),505,state)
 (W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected)
 for name in ['angles.bin','bounce.bin','ethereal.anb']:(W/name).write_bytes((P/'private/review/family8-state4'/name).read_bytes())
 result={'status':'passed-source-execution','production_comparison':'not run','cases':len(inputs)//113,'literal_instructions':steps,'timeline':timeline,'source_pins':{str(f.relative_to(P)):sha(f)for f in model.files},'fixture_pins':{str(f.relative_to(P)):sha(f)for f in D.glob('*.bin')},'ports':['audioengine requests captured; playbackquery false','83140 queue request only','97910math.isqrt; typedvtable/animation bindings'],'scope':'markerrecord+RNG+audio/queue and unchangedmap; nofullmissionclaim'}
 (W/'source.json').write_text(json.dumps(result,indent=2)+'\n');return result

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--compile',action='store_true');ap.add_argument('--work',type=Path);args=ap.parse_args();W=args.work or P/'private/review/zone-marker-motion'/uuid.uuid4().hex;W.mkdir(parents=True,exist_ok=True)
 try:
  result=prepare(W)
  if args.compile:
   pins={str(f.relative_to(P)):sha(f)for c in ['assets','core','runtime']for f in(P/f'rust/crates/genewars-{c}/src').rglob('*.rs')}
   def run(cmd):
    r=subprocess.run([str(v)for v in cmd],cwd=P,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace')
   rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
   for c in ['assets','core','runtime']:
    cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{c}','--crate-type','rlib',P/f'rust/crates/genewars-{c}/src/lib.rs','-L',f'dependency={W}','-o',W/f'libgenewars_{c}.rlib']
    if c=='runtime':
     for dep in ['assets','core']:cmd+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
    run(cmd)
   cmd=[rustc,'--edition=2021','-O',Path(__file__).with_suffix('.rs'),'-L',f'dependency={W}','-o',W/'review.exe']
   for dep in ['assets','core','runtime']:cmd+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
   run(cmd);run([W/'review.exe',W]);got=(W/'actual.bin').read_bytes();want=(W/'expected.bin').read_bytes();assert got==want,[(i,a,b)for i,(a,b)in enumerate(zip(got,want))if a!=b][:15]
   assert all(sha(P/f)==h for f,h in pins.items()),'productionchanged';result.update(status='passed',production_comparison='passed',implementation_pins=pins,compared_bytes=len(want))
  result['private_run']=str(W);result['harness_sha256']=sha(Path(__file__))
 except Exception as e:result={'status':'error','error':repr(e),'private_run':str(W)}
 Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k not in ['source_pins','fixture_pins','timeline','implementation_pins']},indent=2));return 0 if result['status'].startswith('passed')else 1
if __name__=='__main__':raise SystemExit(main())
