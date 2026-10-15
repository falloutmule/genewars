"""Literal kind5 code3C postphysics4B630/EFF60 lifetime.
6DF30 and49D30 are explicit ports: compare numeric animation + retirement
request, never claim world physics/cleanup/render or campaign acceptance.
"""
from pathlib import Path
import argparse,hashlib,importlib.util,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--source-only',action='store_true');ap.add_argument('--no-build',action='store_true');ap.add_argument('--marker-directory',type=Path);args=ap.parse_args()
 work=P/'private/review/zone-score-marker-motion'/uuid.uuid4().hex;work.mkdir(parents=True)
 sp=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m);vm=m.Source()
 files=[P/'private/ghidra/plant-command/instructions-0004b630.txt',P/'private/ghidra/mission-array-refs/instructions-0004b3b0.txt'];vm.code.update(m.m.Model(files,b'').code);vm.code[0x6df30]=('RET',[]);vm.code[0x49d30]=('RET',[]);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
 obj=P/'private/ghidra/validated/memory-.object3.bin';binary=obj.read_bytes();assert binary[0x4eb4+60*6:0x4eb4+61*6]==bytes([0,0,11,255,0,0]);assert struct.unpack_from('<I',binary,0x12254+0x64)[0]==0x4b630
 code=P/'private/ghidra/validated/memory-.object1.bin';assert struct.unpack_from('<I',code.read_bytes(),0x3b524+(60-6)*4)[0]==0x4b7d5
 resource=P/'private/integration/game-runtime/effects/EFF.ANB';db=bytearray(resource.read_bytes());_,_,_,frames,animations=struct.unpack_from('<H4I',db);start,end,n,flags=struct.unpack_from('<IIBB',db,animations+600);assert(n,flags,end-start)==(6,0,70)and all(db[frames+start+i*14+10]==0 for i in range(6));address=0x620000
 for i in range(struct.unpack_from('<H',db)[0]):
  for o in [0,4]:struct.pack_into('<I',db,animations+i*10+o,address+frames+struct.unpack_from('<I',db,animations+i*10+o)[0])
 marker=args.marker_directory
 if marker is None:
  receipts=[p for p in (P/'private/review/zone-score-marker').glob('*/result.json')if json.loads(p.read_text())['status']=='passed'];assert receipts,'a passed real marker factory fixture is required';marker=max(receipts,key=lambda p:p.stat().st_mtime_ns).parent
 result=json.loads((marker/'result.json').read_text());index=next(i for i,r in enumerate(result['rows'])if r['case']=='captured-owned-before2800');slot=result['rows'][index]['slot'];real=(marker/'expected.bin').read_bytes()[index*16227+slot*81:index*16227+(slot+1)*81];assert len(real)==81 and real[69]==60
 cases=[]
 for frame in range(6):
  for delay in range(1,4):
   for counter in [0,1,2,3,61,127,128,255]:
    for timer in [-1,0,1,32767]:
     b=bytearray(real);b[27]=frame;b[29]=delay;b[30]=counter;struct.pack_into('<h',b,74,timer);cases.append((f'frame{frame}-delay{delay}-counter{counter}-timer{timer}',bytes(b)))
 pins={p.relative_to(P).as_posix():sha(p)for p in [Path(__file__),P/'rust/crates/genewars-runtime/src/zone_score_marker_motion.rs',P/'rust/crates/genewars-runtime/src/landing.rs',P/'rust/crates/genewars-runtime/src/landing_effect_lifecycle.rs',resource,obj,code,*files,*vm.files,marker/'result.json',marker/'expected.bin']}
 data=bytearray();expected=bytearray();rows=[]
 def oracle(record):
  source=bytearray(record);struct.pack_into('<II',source,31,address+animations+600,0x112254);state={address+i:b for i,b in enumerate(db)};state.update({0x1688ae+i:b for i,b in enumerate(source)});state[0x11872c]=0;state.update({0x118782+i:0 for i in range(4)})
  report,_,_=vm.run(seed=5428,state_before=state,entry=0x4b630,registers={'EAX':0x1688ae},instruction_budget=10000);after=bytearray(vm.load(0x1688ae+i,1)for i in range(81));after[31:39]=record[31:39];assert report['draws']==0;assert 0x48790 not in vm.calls;return bytes(after),0x49d30 in vm.calls,report
 for name,b in cases:
  after,retire,report=oracle(b);data+=b;expected+=bytes([retire])+after;rows.append({'case':name,'instructions':report['steps'],'retirement_requested':retire})
 record=real
 for tick in range(32):
  after,retire,report=oracle(record);data+=record;expected+=bytes([retire])+after;rows.append({'case':f'actual-marker-postphysics-lifetime-{tick+1}','instructions':report['steps'],'retirement_requested':retire});record=after
  if retire:break
 else:raise AssertionError('real marker did not reach six-frame retirement')
 (work/'input.bin').write_bytes(data);(work/'expected.bin').write_bytes(expected)
 receipt={'status':'source_prepared','cases':len(rows),'expected_bytes':len(expected),'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix(),'scope':'Literal4B630/EFF60 six-frame postphysics animation, zero RNG/no factories and49D30 retirement requests.6DF30 and49D30 are explicit ports; no whole physics/cleanup/render/native/zone-arrival acceptance.'}
 if args.source_only:(work/'source-result.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items()if k not in ['rows','pins']}));return
 harness=work/'probe.rs';harness.write_text(r'''use genewars_runtime::zone_score_marker_motion::plan_animation_after_physics;use std::io::{Read,Write};fn main(){let mut data=Vec::new();std::io::stdin().read_to_end(&mut data).unwrap();let mut out=Vec::new();for c in data.chunks_exact(81){let p=plan_animation_after_physics(c.try_into().unwrap()).unwrap();out.push(p.retirement_required as u8);out.extend(p.after);}std::io::stdout().write_all(&out).unwrap();}''')
 if not args.no_build:
  build=subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True);(work/'build.log').write_bytes(build.stdout+build.stderr);build.check_returncode()
 assert all(sha(P/p)==h for p,h in pins.items()),'source changed during comparison'
 target=P/'private/build/rust-target/debug';exe=work/'probe.exe';compile_result=subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={target}/deps','--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-o',str(exe)],env=environment(),capture_output=True);(work/'compile.log').write_bytes(compile_result.stdout+compile_result.stderr);compile_result.check_returncode()
 execution=subprocess.run([str(exe)],input=data,capture_output=True);(work/'probe-stderr.log').write_bytes(execution.stderr);(work/'actual.bin').write_bytes(execution.stdout);execution.check_returncode();actual=execution.stdout;(work/'actual.bin').write_bytes(actual);diff=[i for i,(a,b)in enumerate(zip(actual,expected))if a!=b];passed=len(actual)==len(expected)and not diff;receipt.update(status='passed'if passed else 'error',compared_bytes=len(expected),actual_bytes=len(actual),mismatch_count=len(diff),first_offsets=diff[:20]);(work/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items()if k not in ['rows','pins']}));assert passed,'inspect preserved local comparison'
if __name__=='__main__':main()
