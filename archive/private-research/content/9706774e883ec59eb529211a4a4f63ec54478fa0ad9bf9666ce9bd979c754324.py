"""Independent signed-byte/count58780 loop fixtures against production helper."""
from pathlib import Path
import struct,random,subprocess,hashlib,json
p=Path(__file__).resolve().parents[2];w=p/'private/ghidra/actor-motion';rng=random.Random(0x58780);sha=lambda b:hashlib.sha256(b).hexdigest();vectors=[];expected=bytearray()
for count in range(256):
 for index in range(8):vectors.append((bytes([0,0,0,count,0,1,19,128,255]),index))
for case in range(1024):
 a=bytearray(rng.randbytes(9));a[3]=5
 for index in range(6):
  # Fresh snapshot before every source callback: count and ID list can change.
  if index:a[3]=[0,1,2,3,4,5,255][(case+index)%7];a[4+(index%5)]=(case*17+index)&255
  vectors.append((bytes(a),index))
for a,index in vectors:
 count=a[3]-256 if a[3]>=128 else a[3]
 if count<=0 or index>=count:expected.append(0)
 elif count>5:expected.append(2)
 else:expected.extend(bytes([1])+struct.pack('<h',a[4+index]-256 if a[4+index]>=128 else a[4+index]))
data=bytearray(struct.pack('<I',len(vectors)))
for a,i in vectors:data.extend(a+bytes([i]))
module=(p/'rust/crates/genewars-runtime/src/actor_motion.rs').as_posix();h=w/'order-oracle.rs';h.write_text(f'#[path="{module}"]mod actor_motion;\n'+r'''use actor_motion::*;use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let n=u32::from_le_bytes(b[..4].try_into().unwrap())as usize;assert_eq!(b.len(),4+n*10);let mut out=Vec::new();for c in b[4..].chunks_exact(10){match next_player_actor_id(&c[..9],c[9]as usize){Ok(None)=>out.push(0),Ok(Some(id))=>{out.push(1);out.extend(id.to_le_bytes());},Err(_)=>out.push(2)}}std::io::stdout().write_all(&out).unwrap();}''');exe=w/'order-oracle.exe';rustc=p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True);out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected
f=p/'private/ghidra/mission-array-refs/instructions-00058780.txt';assert sha(f.read_bytes())=='e7f66d33d468912942e86f875cf4d9e1d0157ce8671a911104b8cb9ad6fcb623';r={'cases':len(vectors),'compared_bytes':len(out),'output_sha256':sha(out),'source_sha256':sha(Path(module).read_bytes()),'instruction_contracts':[{'path':'private/ghidra/mission-array-refs/instructions-00058780.txt','sha256':sha(f.read_bytes())}],'scope':'Signedbyte IDs4+index, live signedcount3 percallback; negativecountterminates; owned count>5unsupported. Includes [0,1] byteIDs regression and changing count/list snapshots.'};(p/'research/ghidra/actor-order-evidence.json').write_text(json.dumps(r,indent=2)+'\n');print(r)
