"""Source-controlled action8 producer: packet/UI state, not gameplay/UI acceptance."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
class Source(m.Source):
 def get(self,operand):
  if self.pc==0x41910:self.sound=self.r['EBX']&255
  return super().get(operand)
work=P/'private/review/genepod-human-producer'/uuid.uuid4().hex;work.mkdir(parents=True);rust=P/'rust/crates/genewars-runtime/src/genepod_human_producer.rs';command=P/'rust/crates/genewars-runtime/src/genepod_command.rs';listing=P/'private/ghidra/building-cost-refs/instructions-00021e20.txt'
h=work/'probe.rs';exe=work/'probe.exe'
h.write_text(f'#[path="{command.as_posix()}"]mod genepod_command;#[path="{rust.as_posix()}"]mod producer;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(1878){let i=producer::ProducerInput{selected_player_index:c[1872],panel_state:c[1873],species_list_index:c[1874]as i8,mouse_flags:u16::from_le_bytes([c[1875],c[1876]]),captured_opaque_hit:c[1877]!=0};let q=producer::plan_mule_producer(c[..67].try_into().unwrap(),c[67..1872].try_into().unwrap(),i);if std::env::args().any(|a|a=="reject"){assert!(q.is_err());out.push(1);continue;}let q=q.unwrap();out.push(q.panel_state_after);match q.outcome{producer::ProducerOutcome::Ignored(_)=>out.extend([0,255,0,0,0,0,0,0]),producer::ProducerOutcome::Denied{sound}=>out.extend([1,sound,0,0,0,0,0,0]),producer::ProducerOutcome::Issued{request,sound}=>{out.push(2);out.push(sound.unwrap_or(255));out.extend(request.species.to_le_bytes());out.extend(request.building_slot.to_le_bytes());out.extend((request.start as u16).to_le_bytes());}}}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],env=environment(),check=True)
baseb=bytearray(67);baseb[6]=4;baseb[15]=7;baseb[49]=1;baseb[51]=4
basep=bytearray(1805);basep[0x6ae]=4;basep[0x6e0]=1;struct.pack_into('<i',basep,0x62a,71680);struct.pack_into('<i',basep,0x4a3,75)
cases=[]
for workvalue in [0,-1,20]:
 for mouse in [0,8,0x8008,9]:
  cases.append(dict(work=workvalue,mouse=mouse))
cases += [dict(funds=74),dict(funds=0,secondary=75),dict(funds=2147483647,secondary=1),dict(research=0),dict(research=-1),dict(grade=1),dict(grade=2),dict(panel=1),dict(flags=0),dict(flags=0x41),dict(flags=0x101),dict(work=20,flags=0x101),dict(work=20,payer=1),dict(frame=1),dict(owner=1),dict(hit=False),dict(index=1,list_two=True)]
inputs=bytearray();expected=bytearray();evidence=[]
for case in cases:
 b=bytearray(baseb);p=bytearray(basep);b[24]=case.get('owner',0);b[27]=case.get('frame',0);b[46]=case.get('grade',0);struct.pack_into('<H',b,49,case.get('flags',1));struct.pack_into('<i',b,54,case.get('work',0));b[58]=case.get('payer',0)
 struct.pack_into('<i',p,0x4a3,case.get('funds',75));struct.pack_into('<i',p,0x4a7,case.get('secondary',0));struct.pack_into('<i',p,0x62a,case.get('research',71680))
 if case.get('list_two'):p[0x6e0]=2;p[0x6ae]=1;p[0x6af]=4
 panel=case.get('panel',2);index=case.get('index',0);mouse=case.get('mouse',8);hit=case.get('hit',True)
 inputs+=b+p+struct.pack('<BBbHB',0,panel,index,mouse,hit)
 state={0x410000+i:v for i,v in enumerate(b)};state.update({0x118d74+i:v for i,v in enumerate(p)});state.update({0x400004+i:v for i,v in enumerate(struct.pack('<I',0x410000))});state[0x400008]=panel;state[0x40007d]=index;state[0x11873d]=0;state.update({0x19ff70+i:v for i,v in enumerate(struct.pack('<H',mouse))});state.update({0x116980+i:v for i,v in enumerate(struct.pack('<I',0x420000))})
 state.update({0x420000+i:0 for i in range(12)})
 model=Source();model.sound=255;model.code.update(m.m.Model([listing],b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.code[0x41910]=('RET',['16'])
 for end in [0x22420,0x22438,0x22582,0x225b1,0x22627,0x2263f,0x2267e,0x226b1,0x226e0,0x2273d]:model.code[end]=('RET',[])
 #The generic21E96 enable predicate is separately run below, before action8.
 state.update({0x400000+i:v for i,v in enumerate(struct.pack('<I',0x430000))});state.update({0x430004+i:v for i,v in enumerate(struct.pack('<I',0x100))})
 model.code[0x21f0c]=('RET',[]);model.run(state_before=state,entry=0x21e96,registers={'ESI':0x400000})
 disabled=bool(model.load(0x400105,1)&4)
 if disabled or not hit:expected+=bytes([panel,0,255,0,0,0,0,0,0]);evidence.append(dict(**case,status='generic_disabled_or_nohit'));continue
 r,_,trace=model.run(state_before=state,entry=0x223e3,registers={'ESI':0x400000,'EBP':0x2fff00})
 packet=bytes(model.load(0x420000+i,1)for i in range(6));kind=2 if model.load(0x42000a,1)==12 else(1 if model.sound!=255 else 0)
 expected+=bytes([model.load(0x400008,1),kind,model.sound])+(packet if kind==2 else bytes(6));evidence.append(dict(**case,instructions=r['steps'],kind=kind,sound=model.sound))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i//9,i%9,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
negative=bytearray()
for which in ['list','species','grade','player']:
 b=bytearray(baseb);p=bytearray(basep);index=0
 if which=='list':index=1
 if which=='species':p[0x6ae]=5
 if which=='grade':b[46]=3
 if which=='player':p[2]=1
 negative+=b+p+struct.pack('<BBbHB',0,2,index,8,1)
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*4
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=evidence,compared_bytes=len(actual),guard_cases=4,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,command,listing,Path(__file__)]},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},ports=['captured opaquehit supplied; no new hit test','41910 audio request recorded; no playback','return epilogues stopped before caller stack unwind'],scope='Generic21E96 disable bit and223E3action8 packet/panel/sound literal comparison; noUIorproduction acceptance')
(P/'research/variants/genepod-human-producer-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))
