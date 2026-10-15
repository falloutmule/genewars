"""Closed2CC87 single-creature producer, parent action5 already established.
2C72A is a return; group opcode12/actor opcode3 branches are explicit unsupported
ports. Parent suppression/pause and Mule identity gates have Rust tests only.
No hardware-button mapping, projection execution or world movement claim.
"""
from pathlib import Path
import argparse,hashlib,importlib.util,json,random,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--source-only',action='store_true');ap.add_argument('--no-build',action='store_true');a=ap.parse_args();work=P/'private/review/mule-human-producer'/uuid.uuid4().hex;work.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);vm=m.Source();listing=P/'private/ghidra/entity-order/instructions-00030100.txt';assert sha(listing)=='174fa08085b159b3277ccbc2108913459f5d5f59d4b222d81ac3a9aa067f6c15';vm.code.update(m.m.Model([listing],b'').code)
 for pc in [0x2c72a,0x2ccbf,0x2cd80]:vm.code[pc]=('RET',[])
 order=sorted(vm.code);vm.next=dict(zip(order,order[1:]));module=P/'rust/crates/genewars-runtime/src/mule_human_producer.rs';pins={p.relative_to(P).as_posix():sha(p)for p in [Path(__file__),module,listing,P/'research/campaign/krell-owned-mule-command.json',*vm.files]}
 cases=[]
 for owner in range(4):
  for point in [[0,0],[-32768,32767],[32767,-32768],[2433,14720]]:cases.append((f'owner{owner}-xy{point}',1,owner,owner,True,0,0,0,point,299))
 cases += [('foreign-owner',1,1,0,True,0,0,0,[1,2],7),('neutral-owner',1,4,0,True,0,0,0,[1,2],7),('null-selected',1,0,0,False,0,0,0,[1,2],7),('hover-consumed1',1,0,0,True,1,0,0,[1,2],7),('hover-consumed255',1,0,0,True,255,65535,255,[1,2],7),('group-count',1,0,0,True,0,1,0,[1,2],7),('group-unsignedFFFF',1,0,0,True,0,65535,0,[1,2],7),('group-mask',1,0,0,True,0,0,128,[1,2],7),('inactive-kind',0,0,0,True,0,0,0,[1,2],7),('plant-kind',2,0,0,True,0,0,0,[1,2],7),('effect-kind',5,0,0,True,0,0,0,[1,2],7),('actor-kind-unsupported',3,0,0,True,0,0,0,[1,2],7)]
 inputs=bytearray();expected=bytearray();rows=[];rng=random.Random(0x2cc87)
 for name,kind,owner,player,present,cl,count,mask,point,slot in cases:
  record=bytearray(rng.randbytes(124));record[6]=kind;record[24]=owner;record[122]=4;struct.pack_into('<h',record,15,slot)
  manager=bytearray(rng.randbytes(0x59d));struct.pack_into('<H',manager,0x526,count);manager[0x52f]=mask;packet=bytearray(rng.randbytes(11));packet[10]=99;manager_address=0x2d0000;packet_address=0x2e0000;creature_address=0x1542be
  state={manager_address+i:b for i,b in enumerate(manager)};state.update({packet_address+i:b for i,b in enumerate(packet)});state.update({creature_address+i:b for i,b in enumerate(record)})
  for address,width,value in [(0x118740,4,creature_address if present else 0),(0x116980,4,packet_address),(0x16e0b4,2,point[0]),(0x16e0b6,2,point[1]),(0x118d75,1,player)]:state.update({address+i:b for i,b in enumerate((value&((1<<(8*width))-1)).to_bytes(width,'little'))})
  report,_,_=vm.run(entry=0x2cc87,state_before=state,registers={'ESI':manager_address,'ECX':cl},instruction_budget=3000);assert report['draws']==0
  after_packet=bytes(vm.load(packet_address+i,1)for i in range(11));after_manager=bytes(vm.load(manager_address+i,1)for i in range(0x59d));status=1 if cl==0 and(count!=0 or mask!=0)else 2 if cl==0 and present and owner==player and kind==3 else 0
  expected+=bytes([status,after_packet[10]==17])+after_packet+after_manager
  inputs+=record+manager+packet+struct.pack('<4B2h',present,0,player,cl,*point);rows.append({'case':name,'instructions':report['steps'],'unsupported_branch_port':status,'issued':after_packet[10]==17})
 (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected);scope='Closed2CC87..2CE90 single-creature packet/manager numeric writes and ignores;2C72A return, group2CCBF and actor2CD80 explicit unsupported ports. Caller action5, projectedpoint, hoverCL already established. Parent19FF71&0A/pause guards and bounded Mule identity are not executed by this source validator. No hardware mapping, projection, world movement or native acceptance.'
 result={'status':'source_prepared','cases':len(cases),'expected_bytes':len(expected),'rows':rows,'pins':pins,'private_run':work.relative_to(P).as_posix(),'scope':scope}
 if a.source_only:(work/'source-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k not in ['rows','pins']}));return
 harness=work/'probe.rs';harness.write_text(r'''use genewars_runtime::mule_human_producer::*;use std::io::{Read,Write};fn main(){let mut data=Vec::new();std::io::stdin().read_to_end(&mut data).unwrap();let mut out=Vec::new();for c in data.chunks_exact(1580){let record=c[..124].try_into().unwrap();let manager:&[u8;0x59d]=c[124..1561].try_into().unwrap();let mut packet=CommandPacket{words:[0;5],opcode:c[1571]};for(i,w)in packet.words.iter_mut().enumerate(){*w=i16::from_le_bytes(c[1561+2*i..1563+2*i].try_into().unwrap());}let input=ProducerInput{selected_player_index:c[1573],selected_player_owner:c[1574],suppression_flags:0,pause_flags:0,hover_consumed_cl:c[1575],projected_point:[i16::from_le_bytes(c[1576..1578].try_into().unwrap()),i16::from_le_bytes(c[1578..1580].try_into().unwrap())]};let p=plan_action5(if c[1572]!=0{Some(record)}else{None},packet,manager,input);let(status,issued,after_packet,after_manager)=match p{Ok(p)=>(0,p.outcome==ProducerOutcome::Issued,p.packet_after,p.manager_after),Err(ProducerError::UnsupportedGroup)=>(1,false,packet,*manager),Err(ProducerError::UnsupportedActorKind)=>(2,false,packet,*manager),Err(e)=>panic!("{:?}",e)};out.extend([status,issued as u8]);for w in after_packet.words{out.extend(w.to_le_bytes());}out.push(after_packet.opcode);out.extend(after_manager);}std::io::stdout().write_all(&out).unwrap();}''')
 if not a.no_build:
  build=subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True);(work/'build.log').write_bytes(build.stdout+build.stderr);build.check_returncode()
 assert all(sha(P/p)==h for p,h in pins.items()),'source changed during comparison'
 target=P/'private/build/rust-target/debug';exe=work/'probe.exe';build=subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={target}/deps','--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-o',str(exe)],env=environment(),capture_output=True);(work/'compile.log').write_bytes(build.stdout+build.stderr);build.check_returncode();actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout;(work/'actual.bin').write_bytes(actual);diff=[i for i,(x,y)in enumerate(zip(actual,expected))if x!=y];passed=len(actual)==len(expected)and not diff;result.update(status='passed'if passed else 'error',compared_bytes=len(expected),actual_bytes=len(actual),mismatch_count=len(diff),first_offsets=diff[:20]);(work/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k not in ['rows','pins']}));assert passed,'inspect preserved source comparison'
if __name__=='__main__':main()
