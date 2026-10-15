"""Literal95300/952A0 and5A140 typed-boundary proof; no native acceptance.
Pixels (including transparent packed bytes) are deliberately present in source
memory but never read. 5A140 virtual height/frame calls are explicit typed ports;
resource1E840 resolution, world ordering and hardware selection are excluded.
"""
from pathlib import Path
import argparse, hashlib, importlib.util, json, struct, subprocess, sys, uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--source-only',action='store_true');a=ap.parse_args()
 work=P/'private/review/creature-pick'/uuid.uuid4().hex;work.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 files=[P/'private/ghidra/mule-picking-148'/f'instructions-000{x}.txt'for x in ['95300','952a0']]
 model=m.Source();model.code.update(m.m.Model(files,b'').code)
 # These two calls are typed input boundaries, not claimed resource proofs.
 model.code.update({0x95000:('MOV',['EAX','0']),0x95001:('RET',[]),0x95010:('MOV',['EAX','0x500000']),0x95011:('RET',[])})
 order=sorted(model.code);model.next=dict(zip(order,order[1:]));rows=[];inputs=[];expected=[]
 # Every frame has overlapping first/last entries. Last may be null or zero-sized.
 cases=[]
 for factor,s76,s77 in [(150,0,255),(255,100,255),(256,255,255)]:
  for flags in [0,1,2,3,0xfc]:
   for off in [-129,-1,0,1,129]:
    for point in [(-2,-2),(0,0),(1,1),(3,3)]:
     cases.append((factor,s76,s77,off,flags,4,4,False,point))
 for factor,s76,s77 in [(0,0,0),(1,0,0),(150,0,255),(255,100,255),(256,255,255)]:
  for w,h,null in [(0,4,False),(4,0,False),(255,255,False),(4,4,True)]:
   cases.append((factor,s76,s77,-1,3,w,h,null,(0,0)))
 for index,(factor,s76,s77,off,flags,w,h,null,point) in enumerate(cases):
  state={};put=lambda at,n,v:state.update({at+i:(v>>(8*i))&255 for i in range(n)})
  frame=0x500000;components=0x510000;sprite=0x520000
  put(frame,4,components);put(frame+11,1,2)
  for ci in [0,1]:
   cp=components+ci*10;put(cp,4,0 if null and ci==1 else sprite);put(cp+4,2,off);put(cp+6,2,-off);put(cp+8,1,flags)
  put(sprite,4,0x530000);put(sprite+4,1,w);put(sprite+5,1,h)
  # All zero packed4 data: transparency does not suppress rectangle membership.
  for byte in range(32768):state[0x530000+byte]=0
  origin=(-17,23);scaledx=int(off*factor/256);scaledy=int(-off*factor/256)
  query=(origin[0]+scaledx+point[0],origin[1]+scaledy+point[1])
  put(0x300004,4,query[1]);put(0x300008,4,factor)
  report,_,_=model.run(entry=0x95300,state_before=state,registers={'EAX':frame,'EDX':origin[0],'EBX':origin[1],'ECX':query[0]},instruction_budget=1000)
  selected=m.signed(model.r['EAX'],32);assert report['draws']==0
  # Wrapper words deliberately exercise signed ground/pointer coordinates and
  # negative relative-height arithmetic SAR. Primitive factor may be0/1 while
  # wrapper independently clamps its byte products to150.
  height=[-65,-1,0,63,64,7194][index%6];ground=(-17,23+(height>>6))
  put(0x1542be+0x76,1,s76);put(0x1542be+0x77,1,s77);put(0x1542be+0x23,4,0x540000)
  put(0x540004,4,0x95000);put(0x54001c,4,0x95010);put(0x300004,4,query[1])
  model.code[0x95000]=('MOV',['EAX',str(height&0xffffffff)])
  wrap,_,_=model.run(entry=0x5a140,state_before=state,registers={'EAX':0x1542be,'EDX':ground[0],'EBX':ground[1],'ECX':query[0]},instruction_budget=2000)
  hit=model.r['EAX']&255;assert hit in [0,1]and wrap['draws']==0
  values=[factor,s76,s77,off,flags,w,h,int(null),*origin,*query,*ground,height]
  inputs.append(' '.join(map(str,values)));expected.append(f'{selected} {hit}')
  rows.append({'case':index,'primitive_factor':factor,'wrapper_factor':max(150,min(256,s76*s77//100)),'offset':off,'flags':flags,'size':[w,h],'last_null':null,'point':query,'relative_height':height,'component':selected,'wrapper_hit':hit,'steps':[report['steps'],wrap['steps']]})
 (work/'input.txt').write_text('\n'.join(inputs)+'\n');(work/'expected.txt').write_text('\n'.join(expected)+'\n')
 module=P/'rust/crates/genewars-runtime/src/creature_pick.rs'
 pins={p.relative_to(P).as_posix():sha(p)for p in [Path(__file__),module,*files,P/'private/ghidra/initial-creature/instructions-0005a110.txt']}
 result={'status':'source_prepared','cases':len(rows),'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix(),'scope':'95300/952A0 plus5A140 with typed virtualheight/frame ports; zero RNG. Transparency ignored exactly. Resource frame resolution, mixed world ordering, visibility, hardware selection, native acceptance and campaign progress excluded.'}
 if not a.source_only:
  probe=work/'probe.rs';probe.write_text('#[path="'+module.as_posix()+'"]mod pick;use pick::*;use std::io::{Read,Write};fn main(){let mut s=String::new();std::io::stdin().read_to_string(&mut s).unwrap();let mut out=String::new();for line in s.lines(){let v:Vec<i32>=line.split_whitespace().map(|x|x.parse().unwrap()).collect();let c=PickComponent{offset:[v[3]as i16,-v[3]as i16],sprite_dimensions:Some([v[5]as u8,v[6]as u8]),flags:v[4]as u8};let mut last=c;if v[7]!=0{last.sprite_dimensions=None;}let cs=[c,last];let index=hit_frame(&cs,[v[8],v[9]],[v[10],v[11]],v[0]as u16).unwrap().map(i32::from).unwrap_or(-1);let hit=hit_mule(&cs,MulePickInput{scale76:v[1]as u8,scale77:v[2]as u8,ground:[v[12]as i16,v[13]as i16],point:[v[10]as i16,v[11]as i16],relative_height:v[14]}).unwrap();out.push_str(&format!("{} {}\\n",index,u8::from(hit)));}assert_eq!(hit_frame(&[],[0,0],[0,0],256),Err(PickError::EmptyFrame));assert_eq!(hit_frame(&vec![PickComponent{offset:[0,0],sprite_dimensions:None,flags:0};256],[0,0],[0,0],256),Err(PickError::TooManyComponents));assert_eq!(hit_frame(&[PickComponent{offset:[0,0],sprite_dimensions:None,flags:0}],[0,0],[0,0],257),Err(PickError::Factor));std::io::stdout().write_all(out.as_bytes()).unwrap();}')
  exe=work/'probe.exe';compile=subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(probe),'-o',str(exe)],env=environment(),capture_output=True);(work/'compile.log').write_bytes(compile.stdout+compile.stderr);compile.check_returncode()
  run=subprocess.run([str(exe)],input=(work/'input.txt').read_bytes(),capture_output=True);(work/'actual.txt').write_bytes(run.stdout);(work/'probe-stderr.log').write_bytes(run.stderr);run.check_returncode()
  actual=run.stdout.decode().replace('\\n','\n').splitlines();assert actual==expected,[(i,x,y)for i,(x,y)in enumerate(zip(actual,expected))if x!=y][:10]
  result.update(status='passed',compared_results=len(expected)*2,mismatches=0,negative_guards=3)
 assert all(sha(P/p)==h for p,h in pins.items()),'source changed during proof'
 (work/('source-result.json'if a.source_only else'result.json')).write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps({k:v for k,v in result.items()if k not in ['rows','pins']}))
if __name__=='__main__':main()
