from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,uuid
p=Path(__file__).resolve().parents[2];w=p/'private/ghidra/construction-command'/uuid.uuid4().hex;w.mkdir(parents=True)
s=importlib.util.spec_from_file_location('m',p/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Source(m.Source):
 def get(self,o):
  if self.pc==0x53671:
   assert(self.r['EDX'],self.r['EBX'],self.r['ECX'],self.load(self.r['ESP'],4),self.load(self.r['ESP']+4,4))==(96,2,0,127,100)
   self.audio=True;return 0x2e0000
  return super().get(o)
x=Source();f=p/'private/ghidra/building-cost-refs/instructions-000534d0.txt';assert hashlib.sha256(f.read_bytes()).hexdigest()=='4fe274a37c60b76c26cd12c335d8cbd68d1c2387419b8a7d881f922b68d4c141';x.code.update(m.m.Model([f],b'').code);x.code[0x2e0000]=('RET',['0x8']);order=sorted(x.code);x.next=dict(zip(order,order[1:]));data=bytearray();expected=bytearray();steps=0
for case in range(192):
 a=bytearray(200);a[6]=3;a[0x55]=9;a[0x7e]=0;a[0x82]=1;a[0x83]=1;struct.pack_into('<H',a,0x88,1+case%2);a[0x90]=case%25;a[0x1b]=case//2%8;a[0x1a]=0 if case%3 else 2
 delta=[0,127,255][case//16%3];struct.pack_into('<hh',a,7,10000,12000);struct.pack_into('<hh',a,11,10000+delta,12000)
 job=bytearray(176);struct.pack_into('<h',job,12,[0,1,100][case//48%3]);native=bytearray(a);native[35:39]=(0x112194).to_bytes(4,'little')
 state={0x16360e+i:b for i,b in enumerate(native)};state.update({0x11ac27+a[0x90]*176+i:b for i,b in enumerate(job)});x.audio=False
 report,_,_=x.run(entry=0x534d0,state_before=state,registers={'EAX':0x16360e});after=bytearray(x.load(0x16360e+i,1)for i in range(200));after[35:39]=a[35:39];expected+=after+bytes([x.audio])+ (struct.pack("<4Bi",96,2,0,127,100)if x.audio else bytes(8));data+=a+job+struct.pack('<I',delta*delta);assert report['draws']==0;steps+=report['steps']
module=(p/'rust/crates/genewars-runtime/src/construction_wait.rs').as_posix();dep=(p/'rust/crates/genewars-runtime/src/actor_construction_lifecycle.rs').as_posix()
# The audio request type is independently declared for this isolated numeric build.
h=w/'wait.rs';h.write_text('mod actor_construction_lifecycle{#[derive(Debug,Clone,PartialEq,Eq)]pub struct ConstructionAudioRequest{pub channel:u8,pub mode:u8,pub parameter:u8,pub stack_byte:u8,pub stack_dword:i32}}'+f'#[path="{module}"]mod wait;'+r'use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(380){let r=wait::plan_architect_terraform_wait(c[..200].try_into().unwrap(),c[200..376].try_into().unwrap(),u32::from_le_bytes(c[376..].try_into().unwrap())).unwrap();out.extend(r.after);out.push(r.pending_audio.is_some()as u8);if let Some(a)=r.pending_audio{out.extend([a.channel,a.mode,a.parameter,a.stack_byte]);out.extend(a.stack_dword.to_le_bytes());}else{out.extend([0;8]);}}std::io::stdout().write_all(&out).unwrap();}')
exe=w/'wait.exe';subprocess.run([str(p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True);out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected
r={'cases':192,'compared_bytes':len(out),'literal_instructions':steps,'audio_requests':sum(expected[i]for i in range(200,len(expected),209)),'production_sha256':hashlib.sha256(Path(module).read_bytes()).hexdigest(),'listing_sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'scope':'534D0 stage1/2 active job with exact source736F0 distances0,127,255; whole actor and audio; zero RNG'};(p/'research/ghidra/construction-wait-evidence.json').write_text(json.dumps(r,indent=2));print(r)
