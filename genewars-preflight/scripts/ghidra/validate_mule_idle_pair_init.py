"""Literal62F23..63045 idle mating initializer on authentic2601 entry.
Selection/5D700 caller eligibility is separately source-inspected, not executed
by this slice. Both124B records, fullplantarray and zero RNG are checked.
"""
from pathlib import Path
import argparse,hashlib,importlib.util,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--source-only',action='store_true');args=ap.parse_args();work=P/'private/review/mule-idle-pair-init'/uuid.uuid4().hex;work.mkdir(parents=True)
 fixture=P/'private/review/krell-mule-callback/0d379b164a094c66866e2d514f828858/idle-pair-boundaries';before=fixture/'62f23';after=fixture/'63045';current=(before/'creature-4.bin').read_bytes();peer=(before/'creature-0.bin').read_bytes();plants=(before/'plants.bin').read_bytes();rng=(before/'rng.bin').read_bytes();backing=(P/'private/review/authentic-krell-141/frame-2601-slot-4-blocked-before/backing.bin').read_bytes()
 sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
 module=P/'rust/crates/genewars-runtime/src/mule_idle_pair_init.rs';world=P/'rust/crates/genewars-runtime/src/creature_world.rs';inputs=bytearray();expected=bytearray();rows=[]
 # Capture's negative peer stage and low reserve are legitimate. Traitvariants
 # prove63010 usesCURRENT virtual80 for BOTH records, not peer's own speed.
 variants=[('actual',None,None,False),('speed0',0,100,False),('speed50',50,0,False),('speed100',100,0,False),('local-stale-targets',None,None,True)]
 variants += [(f'peer-stage{v}',None,None,False)for v in [0,3,128,255]]
 for name,skill,other_skill,stale in variants:
  a=bytearray(current);b=bytearray(peer)
  if skill is not None:a[118]=skill;b[118]=other_skill
  if name.startswith('peer-stage'):b[121]=int(name[10:])
  if stale:
   a[85:88]=struct.pack('<hB',13,1);b[85:88]=struct.pack('<hB',14,1)
  state={0x1542be+496+i:v for i,v in enumerate(a)};state.update({0x1542be+i:v for i,v in enumerate(b)});state.update({0x15d40e+i:v for i,v in enumerate(plants)})
  ebp=0x2fff00;state.update({ebp-0x5fe+i:v for i,v in enumerate(struct.pack('<I',0x1542be))});seed,ring,draws=struct.unpack('<HBI',rng);state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))})
  model=m.Source(backing);model.code[0x63045]=('RET',[]);report,_,_=model.run(seed=seed,state_before=state,entry=0x62f23,registers={'ESI':0x1542be+496,'EBP':ebp},instruction_budget=10000)
  aa=bytes(model.load(0x1542be+496+i,1)for i in range(124));bb=bytes(model.load(0x1542be+i,1)for i in range(124));pp=bytes(model.load(0x15d40e+i,1)for i in range(len(plants)));rr=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4));assert pp==plants and rr==rng and report['draws']==0
  if name=='actual':assert aa==(after/'creature-4.bin').read_bytes()and bb==(after/'creature-0.bin').read_bytes()
  inputs+=a+b;expected+=aa+bb;rows.append({'case':name,'instructions':report['steps'],'draws':0,'plant_bytes_checked':len(pp)})
 (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
 pins={p.relative_to(P).as_posix():sha(p)for p in [Path(__file__),module,world,*before.glob('*'),*after.glob('*'),*model.files]if p.is_file()}
 receipt={'status':'source_prepared','cases':len(rows),'expected_bytes':len(expected),'plant_bytes':len(plants)*len(rows),'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix(),'scope':'Literal idlepair62F23..63045: both124B records, fullplantarray unchanged, zero RNG. Source-selectedmate/parentcriteria and latermotion/lifecycle/nativecampaign acceptance excluded.'}
 if args.source_only:(work/'source-result.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items()if k not in ['rows','pins']}));return
 text=world.read_text();functions=[]
 for signature in ['fn mule_skill_percent(', 'pub(crate) fn mule_requested_speed(']:
  start=text.index(signature);end=text.index('\n}',start)+2;functions.append(text[start:end])
 harness=work/'probe.rs';harness.write_text('mod creature_world{type CreatureRecord=[u8;124];'+''.join(functions)+'}\n#[path="'+module.as_posix()+'"]mod pair;'+r'''use std::io::{Read,Write};fn main(){let mut data=Vec::new();std::io::stdin().read_to_end(&mut data).unwrap();let mut out=Vec::new();for c in data.chunks_exact(248){let p=pair::plan_pair_init(c[..124].try_into().unwrap(),c[124..].try_into().unwrap());if std::env::args().any(|a|a=="reject"){assert!(p.is_err());out.push(1);continue;}let p=p.unwrap();out.extend(p.current_after);out.extend(p.peer_after);}std::io::stdout().write_all(&out).unwrap();}''')
 exe=work/'probe.exe';compile_result=subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-o',str(exe)],env=environment(),capture_output=True);(work/'compile.log').write_bytes(compile_result.stdout+compile_result.stderr);compile_result.check_returncode()
 execution=subprocess.run([str(exe)],input=inputs,capture_output=True);(work/'probe-stderr.log').write_bytes(execution.stderr);execution.check_returncode();actual=execution.stdout;(work/'actual.bin').write_bytes(actual);diff=[i for i,(a,b)in enumerate(zip(actual,expected))if a!=b];assert len(actual)==len(expected)and not diff,diff[:20]
 negative=bytearray()
 for off,value in [(6,3),(24,0),(122,5),(94,7),(119,254),(121,253),(87,2),(82,1),(96,0x8a)]:
  a=bytearray(current);a[off]=value;negative+=a+peer
 a=bytearray(current);a[104:106]=a[98:100];a[104:106]=struct.pack('<h',struct.unpack_from('<h',a,98)[0]>>1);negative+=a+peer
 for off,value in [(94,3),(119,254),(74,4),(87,2),(82,1),(96,0x8a)]:
  b=bytearray(peer);b[off]=value;negative+=current+b
 b=bytearray(peer);b[15:17]=current[15:17];negative+=current+b
 rejected=subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout;assert rejected==bytes([1])*(len(negative)//248)
 assert all(sha(P/p)==h for p,h in pins.items()),'source changed during proof'
 receipt.update(status='passed',compared_bytes=len(expected),negative_guards=len(negative)//248,mismatch_count=0);(work/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items()if k not in ['rows','pins']}))
if __name__=='__main__':main()
