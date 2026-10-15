"""Standalone Rust owned command versus literal factory-derived afterimages."""
from pathlib import Path
import importlib.util,contextlib,io,struct,subprocess,sys,json,hashlib,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('probe',Path(__file__).with_name('krell_owned_mule_command_probe.py'));q=importlib.util.module_from_spec(sp)
with contextlib.redirect_stdout(io.StringIO()):sp.loader.exec_module(q)
vm=q.vm;w=P/'private/review/krell-owned-mule-command-review'/uuid.uuid4().hex;w.mkdir(parents=True)
# Case1 is exact initial source factory. Case2 is exact first callback output.
cases=[(q.before,[1536,1536],q.after)]
s=vm.saved_state.copy();before=q.record();q.put(s,0x440002,2,2048);q.put(s,0x440004,2,1792)
r,_,_=vm.run(entry=0x81c60,state_before=s,seed=q.rng()[0],registers={'EAX':0x118d74,'EDX':0x440000});cases.append((before,[2048,1792],q.record()))
module=P/'rust/crates/genewars-runtime/src/mule_owned_command.rs';h=w/'probe.rs';exe=w/'probe.exe'
h.write_text(f'#[path="{module.as_posix()}"]mod command;'+r'''use std::io::{Read,Write};use command::*;fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(128){let before:&[u8;124]=c[..124].try_into().unwrap();let context=CommandContext{caller_owner:0,virtual_dead:false,caller_is_selected:true,selected_panel_refresh:false};let target=[i16::from_le_bytes([c[124],c[125]]),i16::from_le_bytes([c[126],c[127]])];let plan=plan_move(before,target,context).unwrap();assert_eq!(plan.outcome,CommandOutcome::Installed);assert!(plan.feedback_requested);assert!(!plan.panel_refresh_requested);out.extend(plan.after);for mutation in 0..4{let mut bad=*before;match mutation{0=>bad[0x60]|=2,1=>bad[0x57]=1,2=>bad[0x5e]=2,_=>bad[24]=1};assert!(plan_move(&bad,target,context).is_err());}for mode in 0..3{let mut bad=*before;let mut ctx=context;match mode{0=>ctx.virtual_dead=true,1=>bad[0x61]|=1,_=>bad[0x5e]=0x12};let ignored=plan_move(&bad,target,ctx).unwrap();assert_eq!(ignored.after,bad);assert_eq!(ignored.outcome,CommandOutcome::IgnoredIneligible);}}std::io::stdout().write_all(&out).unwrap();}''')
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'-o',str(exe)],env=environment(),check=True,capture_output=True)
data=b''.join(a+struct.pack('<hh',*t)for a,t,_ in cases);expected=b''.join(b for _,_,b in cases);actual=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout
assert actual==expected,next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None)
r={'status':'passed','cases':2,'compared_bytes':248,'negative_guards':8,'ineligible_unchanged_cases':6,'module_sha256':hashlib.sha256(module.read_bytes()).hexdigest(),'validator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'source_probe_receipt_sha256':hashlib.sha256((P/'research/campaign/krell-owned-mule-command-probe.json').read_bytes()).hexdigest(),'private_run':w.relative_to(P).as_posix(),'scope':'Exact owner0 sourcefactory and its firstfullcallback record, literal81C60 cleanup versus pureRust. Syntheticflat; noGenepod caller ejection, liveownedcallback port, or nativeacceptance.'}
(P/'research/campaign/krell-owned-mule-command-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
