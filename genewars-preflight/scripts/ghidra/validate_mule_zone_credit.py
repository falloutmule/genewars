"""Literal793C0/792C0/80780 numeric credit with explicit marker/audio/factory ports.
6D500 markers consume real world RNG when integrated. This oracle ports them;
it proves ordered requests and the subsequent isolated zone particle RNG tail.
"""
from pathlib import Path
import argparse,hashlib,importlib.util,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--no-build',action='store_true');ap.add_argument('--source-only',action='store_true',help='prepare literal source fixtures without compilation');args=ap.parse_args()
 work=P/'private/review/mule-zone-credit'/uuid.uuid4().hex;work.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);vm=m.Source()
 files=[P/'private/ghidra/mission-array-refs/instructions-000793c0.txt',P/'private/ghidra/mission-zone-792c0.txt',P/'private/ghidra/mission-zone-score.txt']
 vm.code.update(m.m.Model(files,b'').code);vm.mem={};audio=vm.load(0x111fec+0x44,4)
 vm.code[audio]=('RET',['0x8']);vm.code[0x6d500]=('RET',['0xc']);vm.code[0x48790]=('RET',['0x10']);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]));events=[]
 class Capture(dict):
  def __getitem__(self,pc):
   if pc in [audio,0x6d500]:
    registers=vm.r;tag=1 if pc==audio else 2;n=2 if tag==1 else 3;slot=vm.load(registers['EAX']+15,2)
    events.append(bytes([tag])+struct.pack('<HBBB',slot,registers['EDX']&255,registers['EBX']&255,registers['ECX']&255)+b''.join(vm.load(registers['ESP']+4+i*4,4).to_bytes(4,'little')for i in range(n)))
   elif pc==0x48790:
    r=vm.r;events.append(bytes([3,r['EAX']&255,vm.load(r['ESP']+8,4)&255])+r['EDX'].to_bytes(4,'little')+r['EBX'].to_bytes(4,'little')+bytes(vm.load(r['ECX']+i,1)for i in range(6))+b''.join(vm.load(r['ESP']+4+i*4,4).to_bytes(4,'little')for i in range(4)))
   return super().__getitem__(pc)
 vm.code=Capture(vm.code)
 module=P/'rust/crates/genewars-runtime/src/mule_zone_credit.rs';pins={p.relative_to(P).as_posix():sha(p)for p in [module,Path(__file__),*vm.files,*files]}
 cases=[]
 for owner in [0,4]:
  for quota in [0,1,2,3,4,8,16,17,24,1024]:
   for approval in [-1,19]:cases.append((f'owner{owner}-quota{quota}-approval{approval}',owner,quota,approval,64,8,[0,0],[0,0]))
 for name,frame,state,xy,zxy in [('off-phase',65,8,[0,0],[0,0]),('corpse',64,24,[0,0],[0,0]),('frozen',64,15,[0,0],[0,0]),('boundary',64,8,[1024,0],[0,0]),('outside',64,8,[1025,0],[0,0]),('signed15-wrap',64,8,[32767,0],[-32768,0])]:cases.append((name,0,1024,0,frame,state,xy,zxy))
 data=bytearray();expected=bytearray();rows=[]
 for index,(name,owner,quota,approval,frame,state,xy,zxy)in enumerate(cases):
  zone=bytearray(19);struct.pack_into('<hh',zone,0,*zxy);zone[4]=12;zone[18]=1
  for off,value in [(6,1024),(10,1024),(14,quota)]:struct.pack_into('<i',zone,off,value)
  player=bytearray(1805);struct.pack_into('<ii',player,13,100,approval)
  mule=bytearray(124);mule[6]=1;mule[24]=owner;mule[94]=state;mule[122]=4;struct.pack_into('<hh',mule,7,*xy);struct.pack_into('<i',mule,39,7000);struct.pack_into('<hh',mule,57,2047,512)
  seed=[1,5428,65535][index%3];ring=[0,31][index%2];count=0xfffffffe;limit=10;events.clear()
  source_state={0x11bd58+i:b for i,b in enumerate(zone)};source_state.update({0x118d74+i:b for i,b in enumerate(player)});source_state.update({0x1542be+i:b for i,b in enumerate(mule)})
  for address,n,value in [(0x1542be+35,4,0x111fec),(0x11c2b0,2,1),(0x11878c,1,1),(0x11872c,1,ring),(0x118782,4,count),(0x11a9dc,2,limit)]:source_state.update({address+i:b for i,b in enumerate(value.to_bytes(n,'little'))})
  report,_,_=vm.run(entry=0x793c0,counter=frame,seed=seed,state_before=source_state,registers={'EAX':0x11bd58,'EDX':4},instruction_budget=20000)
  expected+=bytes(vm.load(0x11bd58+i,1)for i in range(19))+bytes(vm.load(0x118d74+i,1)for i in range(1805))+struct.pack('<HBI',vm.load(0x11878a,2),vm.load(0x11872c,1),vm.load(0x118782,4))+bytes([len(events)])+b''.join(events)
  data+=zone+player+mule+struct.pack('<IHHBI',frame,limit,seed,ring,count);rows.append({'case':name,'instructions':report['steps'],'isolated_zone_tail_draws':report['draws'],'continuations':len(events)})
 (work/'input.bin').write_bytes(data);(work/'expected.bin').write_bytes(expected)
 if args.source_only:
  receipt={'status':'source_prepared','cases':len(cases),'expected_bytes':len(expected),'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix(),'scope':'Literal numeric state and ordered requests with explicit audio/marker/factory ports; no Rust comparison or real marker RNG/effects.'}
  (work/'source-result.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items()if k not in ['rows','pins']}));return
 harness=work/'probe.rs';harness.write_text(r'''use genewars_runtime::{mule_zone_credit::*,blueprint::BlueprintRng};use std::io::{Read,Write};fn main(){let mut data=Vec::new();std::io::stdin().read_to_end(&mut data).unwrap();let mut output=Vec::new();for c in data.chunks_exact(1961){let context=CreditContext{frame:u32::from_le_bytes(c[1948..1952].try_into().unwrap()),participating_players:1,selected_player:0,approval_limit:u16::from_le_bytes(c[1952..1954].try_into().unwrap())};let mut rng=BlueprintRng{seed:u16::from_le_bytes(c[1954..1956].try_into().unwrap()),ring:c[1956],draw_count:u32::from_le_bytes(c[1957..1961].try_into().unwrap())};let plan=plan_one_mule(c[..19].try_into().unwrap(),c[19..1824].try_into().unwrap(),c[1824..1948].try_into().unwrap(),context).unwrap();let mut requests=Vec::new();let count=plan.continuations.len();for continuation in plan.continuations{match continuation{Continuation::Audio{creature_slot,code,ebx,ecx,stack}=>{requests.push(1);requests.extend(creature_slot.to_le_bytes());requests.extend([code,ebx,ecx]);for v in stack{requests.extend(v.to_le_bytes());}},Continuation::Marker{creature_slot,code,ebx,ecx,stack}=>{requests.push(2);requests.extend(creature_slot.to_le_bytes());requests.extend([code,ebx,ecx]);for v in stack{requests.extend(v.to_le_bytes());}},Continuation::ParticleTail(tail)=>{let p=plan_particle_tail(&tail,rng).unwrap();rng=p.rng_after;requests.extend([3,p.code,p.owner]);for v in p.position{requests.extend(v.to_le_bytes());}requests.extend(p.physical_height.to_le_bytes());for v in p.angles{requests.extend(v.to_le_bytes());}for v in p.stack{requests.extend(v.to_le_bytes());}}}}output.extend(plan.zone_after);output.extend(plan.player_after);output.extend(rng.seed.to_le_bytes());output.push(rng.ring);output.extend(rng.draw_count.to_le_bytes());output.push(count as u8);output.extend(requests);}std::io::stdout().write_all(&output).unwrap();}''')
 if not args.no_build:
  build=subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True);(work/'build.log').write_bytes(build.stdout+build.stderr);build.check_returncode()
 assert all(sha(P/p)==h for p,h in pins.items()),'source changed during proof'
 target=P/'private/build/rust-target/debug';lib=target/'libgenewars_runtime.rlib'
 if not lib.exists():lib=max((target/'deps').glob('libgenewars_runtime-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
 exe=work/'probe.exe';compile_result=subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={target}/deps','--extern',f'genewars_runtime={lib}','-o',str(exe)],cwd=P,env=environment(),capture_output=True);(work/'compile.log').write_bytes(compile_result.stdout+compile_result.stderr);compile_result.check_returncode()
 actual=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;(work/'actual.bin').write_bytes(actual)
 diff=[i for i,(a,b)in enumerate(zip(actual,expected))if a!=b];passed=len(actual)==len(expected)and not diff
 receipt={'status':'passed'if passed else 'error','cases':len(cases),'compared_bytes':len(expected),'actual_bytes':len(actual),'mismatch_count':len(diff),'first_offsets':diff[:16],'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix(),'scope':'Single participating player; literal ordered793C0/792C0/80780 numeric credit plus Audio/Marker/Particle requests. Marker6D500, audio and48790 factory are explicit ports; isolated zone-tail RNG only. No real marker RNG/effects, whole-world scheduler, quota-exhaustion or campaign acceptance claim.'}
 (work/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');(P/'research/variants/mule-zone-credit-review.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items()if k not in ['rows','pins']}));assert passed,'see local mismatch receipt'
if __name__=='__main__':main()
