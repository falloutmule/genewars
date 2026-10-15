"""Bounded original Genepod capture and readiness versus standalone Rust."""
from pathlib import Path
import importlib.util,contextlib,io,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('contract',Path(__file__).with_name('genepod_capture_contract.py'));c=importlib.util.module_from_spec(sp)
with contextlib.redirect_stdout(io.StringIO()):sp.loader.exec_module(c)
rust=P/'rust/crates/genewars-runtime/src/genepod_capture.rs';work=P/'private/review/genepod-capture'/uuid.uuid4().hex;work.mkdir(parents=True);h=work/'probe.rs';exe=work/'probe.exe'
h.write_text(f'#[path="{rust.as_posix()}"]mod capture;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();if std::env::args().any(|a|a=="ready"){for c in b.chunks_exact(8){let p=capture::plan_ready(c[0],u16::from_le_bytes([c[1],c[2]]),c[3],i32::from_le_bytes(c[4..8].try_into().unwrap()));out.extend([p.panel_state_after,p.service_40810_55_requested as u8]);}}else{for c in b.chunks_exact(7){let p=capture::plan_capture(capture::CaptureState{captured:c[0]!=0,inside:c[1]!=0,draw:c[2],event:u16::from_le_bytes([c[3],c[4]])},c[5],c[6]!=0);if std::env::args().any(|a|a=="reject"){assert!(p.is_err());out.push(1);continue;}let p=p.unwrap();out.extend(p.action.to_le_bytes());out.extend(p.after.event.to_le_bytes());out.extend([p.after.captured as u8,p.after.inside as u8,p.after.draw]);}}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
for name,events in [('inside',[(40,358,1),(40,358,4),(40,358,2)]),('outside_release',[(40,358,1),(60,358,2)]),('leave_return',[(40,358,1),(60,358,4),(40,358,4),(40,358,2)]),('transparent_press',[(26,344,1)]),('idle_cancel',[(40,358,1),(60,358,0),(40,358,0),(60,358,2),(26,344,0)])]:
 state=c.base.copy();prev=bytes(5)
 for x,y,bits in events:
  inputs+=prev+bytes([bits,int((x,y)==(40,358))]);c.put(state,0x420000,struct.pack('<hh4xBB',x,y,bits,0));c.vm.run(state_before=state,entry=0xa87d0,registers={'EAX':0});state=c.vm.saved_state.copy()
  action=c.vm.r['EAX']&65535;event=c.vm.load(0x19ff70,4);captured=int(c.vm.load(0x19ff88,4)!=0);inside=c.vm.load(0x1a011c,2);draw=c.vm.load(c.control+39,1)
  expected+=struct.pack('<HHBBB',action,event,captured,inside,draw);prev=struct.pack('<BBBH',captured,inside,draw,event);cases.append(dict(sequence=name,packet=bits,action=action,event=event))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i//7,i%7,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
ready_inputs=bytearray();ready_expected=bytearray()
vm=c.m.Source();vm.code.update(c.m.m.Model([P/c.paths[4]],b'').code);vm.code[0x40810]=('RET',[]);vm.code[0x21faa]=('JMP',['0xdeadbeef']);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
for panel in range(4):
 for flags in [0,1,5,0x41,0x101]:
  for frame in [0,1]:
   for remaining in [-1,0,1]:
    state={};c.word(state,0x400004,0x410000);c.put(state,0x400008,bytes([panel]));c.put(state,0x410031,struct.pack('<H',flags));c.put(state,0x41001b,bytes([frame]));c.word(state,0x410036,remaining&0xffffffff)
    r,_,trace=vm.run(state_before=state,entry=0x21f0c,registers={'ESI':0x400000},instruction_budget=2000)
    ready_inputs+=struct.pack('<BHBi',panel,flags,frame,remaining);ready_expected+=bytes([vm.load(0x400008,1),int(0x40810 in trace)])
assert subprocess.run([str(exe),'ready'],input=ready_inputs,capture_output=True,check=True).stdout==ready_expected
negative=bytes(5)+bytes([3,1])+bytes([0,0,2,0,0,1,1]);assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1,1])
(work/'capture-inputs.bin').write_bytes(inputs);(work/'capture-expected.bin').write_bytes(expected);(work/'ready-inputs.bin').write_bytes(ready_inputs);(work/'ready-expected.bin').write_bytes(ready_expected)
r=dict(status='passed',capture_polls=len(cases),ready_cases=len(ready_inputs)//8,negative_cases=2,cases=cases,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,Path(__file__),Path(c.__file__),c.asset]},source_pins={p:hashlib.sha256((P/p).read_bytes()).hexdigest()for p in c.paths},scope='Isolated enabled9128 control literalA87D0/A86D0/98B60 polls with original sprite opacity; readiness21F0C..21FAA with explicit40810 request port. Caller owns managerorigin/hit traversal and actual producer. No native acceptance.')
(P/'research/variants/genepod-capture-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',polls=len(cases),ready_cases=len(ready_inputs)//8)))
