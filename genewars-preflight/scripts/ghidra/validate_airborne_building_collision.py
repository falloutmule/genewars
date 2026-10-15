"""Pure6DD50/73A50 comparison; does not admit74270 or run a whole callback."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,math,pickle,random,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(P/'scripts'))
from cargo import environment
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--no-build',action='store_true');ap.add_argument('--entry-pickle',type=Path,help='trusted local source capture containing memory and registers at6DD50 entry');ap.add_argument('--entry-directory',type=Path,help='literal6DD50 entry directory with entry-mule.bin and entry-obstacle.bin');args=ap.parse_args()
 work=P/'private/review/airborne-building-collision'/uuid.uuid4().hex;work.mkdir(parents=True)
 module=P/'rust/crates/genewars-runtime/src/airborne_building_collision.rs'
 spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);vm=m.Source()
 files=[P/'private/ghidra/countdown-expiry'/('instructions-'+n+'.txt')for n in ['0006dd50','00073a50']]
 assert [sha(f)for f in files]==['7c5177522b7e38137f0a7dd0e8d9bcdefeb12a0f65a3948933bbfe592e0c3385','b4253afb76c7f89f22113b34f60a64ba8d8a38deae3702ee4136addd49640059']
 vm.code.update(m.m.Model(files,b'').code);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
 pins={p.relative_to(P).as_posix():sha(p)for p in [module,Path(__file__),P/'rust/crates/genewars-runtime/src/actor_motion.rs',P/'rust/crates/genewars-runtime/src/actor_activation.rs',P/'rust/crates/genewars-runtime/src/physics_tables.rs',*vm.files,*files]}
 rng=random.Random(0x6dd50);cases=[]
 positions=[([0,0],[0,0]),([1919,14719],[1919,14719]),([32760,100],[-32760,101]),([-32760,-100],[32760,-99]),([0,0],[16384,-16384]),([1000,2000],[-1000,-2000]),([-30000,30000],[30000,-30000]),([32767,-32768],[-32768,32767])]
 for xy,obstacle in positions:
  for height,ground in [(0,0),(7666,3354),(-4096,4096)]:
   for velocity in [[0,0,0],[512,0,0],[-503,-292,37],[0,0,3776],[2500,-4800,-1024]]:
    record=bytearray(rng.randbytes(124));record[6]=1;record[24]=len(cases)%5;record[122]=4
    struct.pack_into('<hh',record,7,*xy);struct.pack_into('<i',record,39,height);struct.pack_into('<3h',record,45,*velocity)
    struct.pack_into('<h',record,61,math.isqrt(sum(v*v for v in velocity)))
    cases.append(('numeric-'+str(len(cases)),bytes(record),obstacle,ground))
 # This case preserves actual captured ejection input, but is not mislabeled
 # as the later postphysics6DD50 entry. Explicit entry inputs add that boundary.
 capture=P/'private/review/krell-production-084/frame-2724-slot-7-blocked-before-owned'
 if capture.is_dir():
  c=(capture/'creatures.bin').read_bytes()[7*124:8*124];b=(capture/'buildings.bin').read_bytes()[2*67:3*67]
  cases.append(('captured-terminal-input-not-collision-entry',c,list(struct.unpack_from('<hh',b,7)),struct.unpack_from('<i',b,17)[0]))
  for p in [capture/'creatures.bin',capture/'buildings.bin']:pins[p.relative_to(P).as_posix()]=sha(p)
 if args.entry_pickle:
  capture_data=pickle.loads(args.entry_pickle.read_bytes());memory=capture_data['memory'];registers=capture_data['registers'];cp=registers['EAX'];bp=registers['EDX']
  read=lambda address,n:bytes(memory.get(address+i,0)for i in range(n))
  c=read(cp,124);b=read(bp,67);cases.append(('exact-source-collision-entry',c,list(struct.unpack_from('<hh',b,7)),struct.unpack_from('<i',b,17)[0]));pins[str(args.entry_pickle.resolve())]=sha(args.entry_pickle)
 if args.entry_directory:
  mule_path=args.entry_directory/'entry-mule.bin';obstacle_path=args.entry_directory/'entry-obstacle.bin'
  c=mule_path.read_bytes();b=obstacle_path.read_bytes();assert len(c)==124 and len(b)==67,'exact native collision entry extents required'
  cases.append(('exact-source-collision-entry-directory',c,list(struct.unpack_from('<hh',b,7)),struct.unpack_from('<i',b,17)[0]))
  for path in [mule_path,obstacle_path]:pins[str(path.resolve())]=sha(path)
 data=bytearray();expected=bytearray();rows=[]
 for name,record,obstacle,ground in cases:
  state={0x1542be+i:b for i,b in enumerate(record)};building=bytearray(67);building[6]=4;struct.pack_into('<hh',building,7,*obstacle);struct.pack_into('<i',building,17,ground)
  state.update({0x1645ae+i:b for i,b in enumerate(building)});state[0x11872c]=0;state.update({0x118782+i:0 for i in range(4)})
  report,_,_=vm.run(entry=0x6dd50,state_before=state,registers={'EAX':0x1542be,'EDX':0x1645ae},instruction_budget=10000)
  after=bytes(vm.load(0x1542be+i,1)for i in range(124));impulse=[((struct.unpack_from('<h',after,45+2*i)[0]-struct.unpack_from('<h',record,45+2*i)[0]+32768)%65536)-32768 for i in range(3)]
  assert report['draws']==0
  expected+=after+struct.pack('<3h',*impulse);data+=record+struct.pack('<2hi',*obstacle,ground);rows.append({'case':name,'instructions':report['steps']})
 (work/'input.bin').write_bytes(data);(work/'expected.bin').write_bytes(expected)
 harness=work/'probe.rs';harness.write_text(r'''use genewars_runtime::{airborne_building_collision::plan_collision,actor_activation::TrigTables,physics_tables::PhysicsTables};use std::io::{Read,Write};fn main(){let root=std::path::PathBuf::from(std::env::args().nth(1).unwrap());let p=root.join("private/integration/game-runtime/physics");let physics=PhysicsTables::from_mapped_bytes(&std::fs::read(p.join("angle.bin")).unwrap(),&std::fs::read(p.join("bounce.bin")).unwrap()).unwrap();let trig=TrigTables::generate_pinned().unwrap();let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut output=Vec::new();for c in b.chunks_exact(132){let xy=[i16::from_le_bytes(c[124..126].try_into().unwrap()),i16::from_le_bytes(c[126..128].try_into().unwrap())];let ground=i32::from_le_bytes(c[128..132].try_into().unwrap());let p=plan_collision(c[..124].try_into().unwrap(),xy,ground,&physics,&trig).unwrap();output.extend(p.after);for v in p.impulse{output.extend(v.to_le_bytes());}}std::io::stdout().write_all(&output).unwrap();}''')
 if not args.no_build:
  build=subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True);(work/'build.log').write_bytes(build.stdout+build.stderr);build.check_returncode()
 assert all(sha(P/p)==h for p,h in pins.items()if not Path(p).is_absolute()),'source/evidence changed during proof'
 target=P/'private/build/rust-target/debug';library=target/'libgenewars_runtime.rlib'
 if not library.exists():library=max((target/'deps').glob('libgenewars_runtime-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
 exe=work/'probe.exe';compile_result=subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={target}/deps','--extern',f'genewars_runtime={library}','-o',str(exe)],cwd=P,env=environment(),capture_output=True);(work/'compile.log').write_bytes(compile_result.stdout+compile_result.stderr);compile_result.check_returncode()
 actual=subprocess.run([str(exe),str(P)],input=data,capture_output=True,check=True).stdout;(work/'actual.bin').write_bytes(actual)
 mismatches=[i for i,(a,b)in enumerate(zip(actual,expected))if a!=b];passed=len(actual)==len(expected)and not mismatches
 receipt={'status':'passed'if passed else 'error','cases':len(cases),'compared_bytes':len(expected),'mismatch_count':len(mismatches),'first_offsets':mismatches[:16],'actual_bytes':len(actual),'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix(),'scope':'Literal6DD50/73A50 pure124B record+impulse; zeroRNG; no74270 height-query admission, no whole callback/campaign/native claim'}
 (work/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');(P/'research/variants/airborne-building-collision-review.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:v for k,v in receipt.items()if k not in ['rows','pins']}));assert passed,'see local mismatch receipt'
if __name__=='__main__':main()
