"""Literal6D500 + real48790 code3C numeric factory comparison.
Pointers are typed bindings: normalize only animation/vtable pointer bytes;
compare all remaining81B of every pool record, animation60, counts and RNG.
No world scheduler, lifecycle or authentic zone-arrival acceptance claim.
"""
from pathlib import Path
import argparse,hashlib,importlib.util,json,random,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def put(s,a,n,v):s.update({a+i:b for i,b in enumerate((v&((1<<(8*n))-1)).to_bytes(n,'little'))})
def normalize(pool):
 p=bytearray(pool)
 for slot in range(200):p[slot*81+31:slot*81+39]=bytes(8)
 return p

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--no-build',action='store_true');ap.add_argument('--source-only',action='store_true');a=ap.parse_args()
 work=P/'private/review/zone-score-marker'/uuid.uuid4().hex;work.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('callback',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 capture=P/'private/review/owned-command-sequence-110';backing=(capture/'backing.bin').read_bytes();vm=m.Source(backing)
 listing=P/'private/ghidra/activation-rng-refs/instructions-0006d500.txt';vm.code.update(m.m.m.Model([listing],b'').code);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
 eff=P/'private/integration/game-runtime/effects/EFF.ANB';resource=bytearray(eff.read_bytes());n,table,components,frames,animations=struct.unpack_from('<H4I',resource);base=0xa00000
 for i in range(n):
  for o in [0,4]:struct.pack_into('<I',resource,animations+10*i+o,base+frames+struct.unpack_from('<I',resource,animations+10*i+o)[0])
 pins={str(p.relative_to(P)).replace('\\','/'):sha(p)for p in [Path(__file__),P/'rust/crates/genewars-runtime/src/zone_score_marker.rs',P/'rust/crates/genewars-runtime/src/landing.rs',listing,eff,capture/'creatures.bin',capture/'backing.bin',*vm.files]}
 actual_mule=(capture/'creatures.bin').read_bytes()[7*124:8*124];cases=[]
 for relative in [-2147483648,-1,0,127,128,129,2147483647]:
  for seed in [1,5428,65535]:
   c=bytearray(actual_mule);struct.pack_into('<i',c,17,0);struct.pack_into('<i',c,39,relative);cases.append((f'height{relative}-seed{seed}',bytes(c),seed,0,0,-1))
 cases += [('captured-owned-before2800',actual_mule,5428,31,21,-1),('explicit-neutral-owner',actual_mule,1,31,5,4),('last-free-slot',actual_mule,65535,0,199,0)]
 data=bytearray();expected=bytearray();rows=[];randomizer=random.Random(0x6d500)
 for name,mule,seed,ring,first_free,owner in cases:
  pool=bytearray(randomizer.randbytes(200*81))
  for slot in range(200):pool[slot*81+6]=5 if slot<first_free else 0;pool[slot*81+69]=6;struct.pack_into('<I',pool,slot*81+35,0x112254)
  c=bytearray(mule);struct.pack_into('<I',c,35,0x111fec);struct.pack_into('<h',c,15,7)
  xy=struct.unpack_from('<hh',c,7);cell=((xy[1]>>8)&127)*128+((xy[0]>>8)&127);map_address=0x11c2be+cell*14
  state={0x118d60+i:b for i,b in enumerate(backing[:0x355e])};state.update({0x11c2be+i:b for i,b in enumerate(backing[0x355e:0x3b55e])});state.update({0x1688ae+i:b for i,b in enumerate(pool)});state.update({0x1542be+7*124+i:b for i,b in enumerate(c)});state.update({base+i:b for i,b in enumerate(resource)})
  for address,width,value in [(0x116818,4,base+animations),(0x11c2b4,2,first_free),(0x11872c,1,ring),(0x118782,4,0xfffffffe),(map_address+6,2,7),(map_address+8,1,1),(0x300004,4,10),(0x300008,4,1),(0x30000c,4,owner)]:put(state,address,width,value)
  report,_,_=vm.run(entry=0x6d500,seed=seed,state_before=state,registers={'EAX':0x1542be+7*124,'EDX':0x3c,'EBX':0,'ECX':10},instruction_budget=20000)
  after=bytes(vm.load(0x1688ae+i,1)for i in range(200*81));record=after[first_free*81:(first_free+1)*81];ground=struct.unpack_from('<i',record,17)[0];shade=record[23];animation=(struct.unpack_from('<I',record,31)[0]-(base+animations))//10
  journals=[vm.load(0x11862c+((ring+i)&31)*4,4)for i in range(4)];assert report['draws']==4 and journals==[0xf6]*3+[0x229]
  expected+=normalize(after)+struct.pack('<HHH',vm.load(0x11c2b4,2),first_free,animation)+struct.pack('<HBI',vm.load(0x11878a,2),vm.load(0x11872c,1),vm.load(0x118782,4))+struct.pack('<4H',*journals)+bytes(vm.load(map_address+6+i,1)for i in range(3))+bytes(vm.load(0x1542be+7*124+i,1)for i in range(3))
  data+=mule+pool+struct.pack('<hiiBBhHBI',first_free,ground,10,shade,owner&255,10,seed,ring,0xfffffffe)
  rows.append({'case':name,'instructions':report['steps'],'draws':report['draws'],'slot':first_free,'ground':ground,'shade':shade})
 (work/'input.bin').write_bytes(data);(work/'expected.bin').write_bytes(expected)
 if a.source_only:
  result={'status':'source_prepared','cases':len(cases),'expected_bytes':len(expected),'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix()};(work/'source-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k not in ['rows','pins']}));return
 harness=work/'probe.rs';harness.write_text(r'''use genewars_runtime::{zone_score_marker::*,landing::*,actor_activation::TrigTables};use std::io::{Read,Write};fn main(){let trig=TrigTables::generate_pinned().unwrap();let mut bytes=Vec::new();std::io::stdin().read_to_end(&mut bytes).unwrap();let mut output=Vec::new();for c in bytes.chunks_exact(16345){let m:&[u8;124]=c[..124].try_into().unwrap();let mut pool=[[0;81];200];for(i,r)in pool.iter_mut().enumerate(){r.copy_from_slice(&c[124+i*81..124+(i+1)*81]);}let p=&c[16324..];let count=i16::from_le_bytes(p[0..2].try_into().unwrap());let ground=i32::from_le_bytes(p[2..6].try_into().unwrap());let base=i32::from_le_bytes(p[6..10].try_into().unwrap());let shade=p[10];let owner=p[11]as i8;let divisor=i16::from_le_bytes(p[12..14].try_into().unwrap())as i32;let rng=LandingRng{seed:u16::from_le_bytes(p[14..16].try_into().unwrap()),ring:p[16],draw_count:u32::from_le_bytes(p[17..21].try_into().unwrap())};let head=LandingReference{index:7,kind:1};let plan=plan_one(m,&pool,count,base,divisor,owner,ground,shade,Some(ResolvedCellHead{identity:head}),1024,rng,trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap()).unwrap();let f=&plan.factory;pool[f.slot as usize]=f.after_nonpointer;for mut r in pool{r[31..39].fill(0);output.extend(r);}output.extend(f.pool_count_after.to_le_bytes());output.extend(f.slot.to_le_bytes());output.extend(f.animation_index.to_le_bytes());output.extend(plan.rng_after.seed.to_le_bytes());output.push(plan.rng_after.ring);output.extend(plan.rng_after.draw_count.to_le_bytes());for d in plan.draws{output.extend(d.journal_code.to_le_bytes());}let id=[f.slot as u8,(f.slot>>8)as u8,5];output.extend(id);output.extend(id);}std::io::stdout().write_all(&output).unwrap();}''')
 if not a.no_build:
  build=subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True);(work/'build.log').write_bytes(build.stdout+build.stderr);build.check_returncode()
 assert all(sha(P/p)==h for p,h in pins.items()),'source changed during proof'
 target=P/'private/build/rust-target/debug';lib=target/'libgenewars_runtime.rlib';exe=work/'probe.exe'
 compile_result=subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={target}/deps','--extern',f'genewars_runtime={lib}','-o',str(exe)],cwd=P,env=environment(),capture_output=True);(work/'compile.log').write_bytes(compile_result.stdout+compile_result.stderr);compile_result.check_returncode()
 actual=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;(work/'actual.bin').write_bytes(actual);diff=[i for i,(x,y)in enumerate(zip(actual,expected))if x!=y];passed=len(actual)==len(expected)and not diff
 result={'status':'passed'if passed else 'error','cases':len(cases),'compared_bytes':len(expected),'actual_bytes':len(actual),'mismatch_count':len(diff),'first_offsets':diff[:20],'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix(),'scope':'Literal one6D500 iteration and real3C48790: all200 normalized81B records, animation binding60, pool count, four interleaved RNG draws and head/previous link requests. Pointer bytes are normalized opaque bindings. No lifecycle, replacement, scheduler, zone-arrival or campaign acceptance claim.'}
 (work/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k not in ['rows','pins']}));assert passed,'inspect preserved local mismatch files'
if __name__=='__main__':main()
