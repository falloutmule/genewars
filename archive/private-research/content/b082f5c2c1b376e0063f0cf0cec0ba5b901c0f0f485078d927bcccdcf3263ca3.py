"""Independent source-pinned shipping landing validation; no original execution."""
from pathlib import Path
import struct,random,subprocess,hashlib,json
p=Path(__file__).resolve().parents[2];work=p/'private/ghidra/landing-contact';rng=random.Random(0x724c0);data=bytearray(struct.pack('<I',2048));expected=bytearray();shapes={1:(300,124,[77,80,85,88,91]),3:(20,200,[77,80,127,123]),5:(200,81,[71]),7:(32,97,[77,80,87,90]),8:(16,102,[89])};matching=0
pins=[]
assert hashlib.sha256((p/'private/ghidra/GW-CD-candidate.exe').read_bytes()).hexdigest()=='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
pin_path=p/'private/ghidra/panel-activation-refs/instructions-000724c0.txt';pin_bytes=pin_path.read_bytes();assert '000724c0 PUSH EBX' in pin_bytes.decode();pins.append(dict(path='private/ghidra/panel-activation-refs/instructions-000724c0.txt',sha256=hashlib.sha256(pin_bytes).hexdigest()))
pin_path=p/'private/ghidra/terrain-map-refs/instructions-00049d30.txt';pin_bytes=pin_path.read_bytes();assert '00049ded DEC word ptr [0x0011c2b4]' in pin_bytes.decode();pins.append(dict(path='private/ghidra/terrain-map-refs/instructions-00049d30.txt',sha256=hashlib.sha256(pin_bytes).hexdigest()))
pin_path=p/'private/ghidra/entity-order-refs/instructions-0004e950.txt';pin_bytes=pin_path.read_bytes();assert '0004e9c6 CALL dword ptr [ECX + 0x64]' in pin_bytes.decode();pins.append(dict(path='private/ghidra/entity-order-refs/instructions-0004e950.txt',sha256=hashlib.sha256(pin_bytes).hexdigest()))
for i in range(2048):
 kind=list(shapes)[i%5];count,width,offsets=shapes[kind];slot=i%count;target=i%200;b=bytearray(rng.randbytes(width));b[6]=kind if i%7 else 0;struct.pack_into('<h',b,15,slot)
 for j,o in enumerate(offsets):struct.pack_into('<hB',b,o,target if (i+j)%3 else (target+1)%200,5 if(i+j)%4 else 3)
 data.extend(struct.pack('<hBBH',target,5,kind,slot)+b);patches=[]
 if b[6]:
  for o in offsets:
   if struct.unpack_from('<hB',b,o)==(target,5):
    patches.append((kind,slot,o+2,b[o+2],0));matching+=1
    if kind==1 and o==91 and b[96]&2:patches.append((kind,slot,96,b[96],b[96]&~2))
 expected.extend(struct.pack('<H',len(patches)))
 for patch in patches:expected.extend(struct.pack('<BHHBB',*patch))
(work/'lifecycle-input.bin').write_bytes(data);(work/'lifecycle-expected.bin').write_bytes(expected)
base=(p/'rust/crates/genewars-runtime/src').as_posix();harness=work/'lifecycle-oracle.rs';harness.write_text(f'#[path="{base}/landing.rs"] mod landing;\n#[path="{base}/landing_effect_lifecycle.rs"] mod landing_effect_lifecycle;\n'+r'''use landing::*;use landing_effect_lifecycle::*;use std::io::{Read,Write};fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let q=*p;*p+=n;&b[q..*p]}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let mut out=Vec::new();for _ in 0..n{let id=i16::from_le_bytes(take(&b,&mut p,2).try_into().unwrap());let deadkind=take(&b,&mut p,1)[0];let kind=take(&b,&mut p,1)[0];let slot=u16::from_le_bytes(take(&b,&mut p,2).try_into().unwrap());let width=match kind{1=>124,3=>200,5=>81,7=>97,8=>102,_=>panic!()};let r=take(&b,&mut p,width);let patches=plan_reference_cleanup(kind,slot,r,LandingReference{index:id,kind:deadkind}).unwrap();out.extend((patches.len() as u16).to_le_bytes());for q in patches{out.push(q.kind);out.extend(q.slot.to_le_bytes());out.extend(q.offset.to_le_bytes());out.push(q.before);out.push(q.after);}}assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}''')
exe=work/'lifecycle-oracle.exe';rustc=p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(harness),'-o',str(exe)],check=True);out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected
receipt={'instruction_contracts':pins,'source_model':'Independent source724C0 matching-kind-byte cleanup with preservedindices; kind1reference5B clearsflag60mask2; inactiveopaque','cases':2048,'matched_references':matching,'output_bytes':len(out),'output_sha256':hashlib.sha256(out).hexdigest(),'harness_sha256':hashlib.sha256(harness.read_bytes()).hexdigest(),'production_sha256':hashlib.sha256((p/'rust/crates/genewars-runtime/src/landing_effect_lifecycle.rs').read_bytes()).hexdigest()};(p/'research/ghidra/landing-lifecycle-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n');print(receipt)
