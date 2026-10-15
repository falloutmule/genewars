"""Literal74270/74160 typed airborne query comparison, without physics."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
OFFSETS=[(0,0),(256,0),(256,256),(0,256),(-256,256),(-256,0),(-256,-256),(256,-256),(7680,0)]
cell=lambda p:((p[1]>>8)&127)*128+((p[0]>>8)&127)
wrap=lambda n:((n+32768)%65536)-32768

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--no-build',action='store_true');ap.add_argument('--entry-directory',type=Path,default=P/'private/review/krell-mule-callback/caf7179999cb4f03afa02464671f804d');args=ap.parse_args()
 work=P/'private/review/airborne-building-query'/uuid.uuid4().hex;work.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);vm=m.Source()
 files=[P/'private/ghidra'/n for n in ['command-resolver/instructions-00074270.txt','actor-activation/instructions-00074160.txt','landing-contact/instructions-0008b460.txt']]
 vm.code.update(m.m.Model(files,b'').code);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
 module=P/'rust/crates/genewars-runtime/src/building_obstacle.rs';pins={p.relative_to(P).as_posix():sha(p)for p in [module,Path(__file__),*vm.files,*files]}
 cases=[]
 def building(selector,xy,ground,flags,id):
  b=bytearray(67);b[6]=4;b[45]=selector;struct.pack_into('<hh',b,7,*xy);struct.pack_into('<h',b,15,id);struct.pack_into('<i',b,17,ground);struct.pack_into('<H',b,49,flags);return bytes(b)
 for selector,extent in [(0,256),(1,85),(2,128)]:
  for raw in [-1,0,5,168]:
   for height in [-1,3354+raw*64-320,3354+raw*64-319]:
    for delta in [0,extent,extent+1]:
     point=[8192,8192];b=building(selector,[point[0]+delta,point[1]],3354,1,0)
     cases.append((f'height-{selector}-{raw}-{height}-{delta}',point,height,0,1,[(cell(point),b,raw)]))
 for index in [1,4,8]:
  point=[8192,8192];at=[point[i]+OFFSETS[index][i]for i in range(2)]
  first=building(2,point,3354,1,0);last=building(0,point,3354,1,1)
  cases.append((f'ordered-class2-index-{index}',point,7666,0,1,[(cell(at),first,168),(cell([point[0]+7680,point[1]])if index!=8 else cell(point),last,168)]))
 point=[32767,-32768];b=building(0,[-32768,32767],0,1,0)
 cases.append(('wrapped-extent',point,0,0,1,[(cell(point),b,168)]))
 b=building(0,[8192,8192],3354,0,0);later=building(0,[8192,8192],3354,1,1)
 cases.append(('first-flags-rejection',[8192,8192],7666,1,1,[(cell([8192,8192]),b,168),(cell([8448,8192]),later,168)]))
 cases.append(('map-gate',[8192,8192],7666,0,0,[(cell([8192,8192]),later,168)]))
 entry=args.entry_directory
 if entry.is_dir():
  mule=(entry/'entry-mule.bin').read_bytes();b=(entry/'entry-obstacle.bin').read_bytes();assert len(mule)==124 and len(b)==67
  resource=P/'private/integration/game-runtime/buildings/BGENE.ANB';anb=resource.read_bytes();_,_,_,frame_table,animation_table=struct.unpack_from('<H4I',anb)
  assert b[45]==0;first=struct.unpack_from('<I',anb,animation_table+b[25]*10)[0];raw=struct.unpack_from('<h',anb,frame_table+first+b[27]*14+6)[0]
  point=list(struct.unpack_from('<hh',mule,7));height=struct.unpack_from('<i',mule,39)[0]
  cases.append(('captured2724-first-building',point,height,0,1,[(cell(point),b,raw)]))
  for path in [entry/'entry-mule.bin',entry/'entry-obstacle.bin',resource]:pins[str(path.resolve())]=sha(path)
 data=bytearray();expected=bytearray();rows=[]
 for name,point,height,flags,gate,entries in cases:
  terrain=bytearray(229376);terrain[cell(point)*14+11]=gate
  state={0x440000+i:b for i,b in enumerate(struct.pack('<hh',*point))}
  for index,(location,record,raw)in enumerate(entries):
   native=bytearray(record);native[3:6]=bytes(3);struct.pack_into('<I',native,31,0x600000+index*0x100)
   state.update({0x1645ae+index*67+i:b for i,b in enumerate(native)})
   terrain[location*14+6:location*14+9]=struct.pack('<hB',index,4)
   descriptor=0x600000+index*0x100+record[25]*10;frame=0x610000+index*0x1000
   state.update({descriptor+i:b for i,b in enumerate(struct.pack('<I',frame))});state.update({frame+record[27]*14+6+i:b for i,b in enumerate(struct.pack('<h',raw))})
  state.update({0x11c2be+i:b for i,b in enumerate(terrain)});state[0x11872c]=0;state.update({0x118782+i:0 for i in range(4)})
  report,_,_=vm.run(entry=0x74270,state_before=state,registers={'EAX':0x440000,'EDX':flags,'EBX':height},instruction_budget=10000)
  pointer=vm.r['EAX'];assert report['draws']==0
  if pointer:
   index=(pointer-0x1645ae)//67;assert pointer==0x1645ae+index*67 and 0<=index<len(entries);expected+=bytes([1])+entries[index][1]
  else:expected+=bytes(68)
  data+=struct.pack('<2hiBB',*point,height,flags,gate)
  for location,record,raw in entries+[(0,bytes(67),0)]*(2-len(entries)):data+=struct.pack('<H',location)+record+struct.pack('<h',raw)
  rows.append({'case':name,'selected_index':(pointer-0x1645ae)//67 if pointer else None,'instructions':report['steps']})
 (work/'input.bin').write_bytes(data);(work/'expected.bin').write_bytes(expected)
 harness=work/'probe.rs';harness.write_text(r'''use genewars_runtime::building_obstacle::building_at_height;use std::io::{Read,Write};fn main(){let mut data=Vec::new();std::io::stdin().read_to_end(&mut data).unwrap();let mut out=Vec::new();for c in data.chunks_exact(152){let point=[i16::from_le_bytes(c[..2].try_into().unwrap()),i16::from_le_bytes(c[2..4].try_into().unwrap())];let mut map=vec![0;229376];let location=(((i32::from(point[1])>>8)&127)*128+((i32::from(point[0])>>8)&127))as usize;map[location*14+11]=c[9];let entries:Vec<_>=(0..2).map(|i|{let n=10+i*71;(u16::from_le_bytes(c[n..n+2].try_into().unwrap())as usize,<[u8;67]>::try_from(&c[n+2..n+69]).unwrap(),i16::from_le_bytes(c[n+69..n+71].try_into().unwrap()))}).collect();let result=building_at_height(&map,point,i32::from_le_bytes(c[4..8].try_into().unwrap()),c[8],|cell|Ok::<_,()>(entries.iter().find(|(at,r,_)|*at==cell&&r[6]!=0).map(|(_,r,_)|*r)),|r|Ok(entries.iter().find(|(_,record,_)|record==r).unwrap().2)).unwrap();match result{Some(r)=>{out.push(1);out.extend(r)},None=>out.extend([0;68])}}std::io::stdout().write_all(&out).unwrap();}''')
 if not args.no_build:
  build=subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True);(work/'build.log').write_bytes(build.stdout+build.stderr);build.check_returncode()
 assert all(sha(P/p)==h for p,h in pins.items()if not Path(p).is_absolute()),'source changed during proof'
 target=P/'private/build/rust-target/debug';lib=target/'libgenewars_runtime.rlib'
 if not lib.exists():lib=max((target/'deps').glob('libgenewars_runtime-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
 exe=work/'probe.exe';compile_result=subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={target}/deps','--extern',f'genewars_runtime={lib}','-o',str(exe)],cwd=P,env=environment(),capture_output=True);(work/'compile.log').write_bytes(compile_result.stdout+compile_result.stderr);compile_result.check_returncode()
 actual=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;(work/'actual.bin').write_bytes(actual)
 differences=[i for i,(a,b)in enumerate(zip(actual,expected))if a!=b];passed=len(actual)==len(expected)and not differences
 result={'status':'passed'if passed else 'error','cases':len(cases),'compared_bytes':len(expected),'actual_bytes':len(actual),'mismatch_count':len(differences),'first_offsets':differences[:16],'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix(),'scope':'Literal74270/74160 first-building query, raw signed frame-height fixtures and captured2724 frame metadata; no whole physics/campaign/native admission'}
 (work/'result.json').write_text(json.dumps(result,indent=2)+'\n');(P/'research/variants/airborne-building-query-review.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items()if k not in ['pins','rows']}));assert passed,'inspect local mismatch receipt'
if __name__=='__main__':main()
