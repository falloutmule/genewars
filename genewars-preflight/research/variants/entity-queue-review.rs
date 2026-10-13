// Native adapter for the independent linked-chain and packet oracle.
#[path="../../rust/crates/genewars-core/src/entity_draw_queue.rs"] mod queue;
use queue::*;
use std::{env,fs,io::{BufWriter,Write},path::Path};
fn n16(b:&[u8],at:&mut usize)->u16{let v=u16::from_le_bytes(b[*at..*at+2].try_into().unwrap());*at+=2;v}
fn n32(b:&[u8],at:&mut usize)->u32{let v=u32::from_le_bytes(b[*at..*at+4].try_into().unwrap());*at+=4;v}
fn admission(out:&mut impl Write,input:AdmissionPacketInput){let bytes=match family1_admission_packet(input){None=>[0;5],Some(p)=>[1,p.context_gate_after as u8,p.flags_after,p.dirty_mark_requested as u8,p.dirty_body_runs as u8]};out.write_all(&bytes).unwrap();}
fn main(){let args:Vec<String>=env::args().collect();let p=Path::new(&args[1]);let input=fs::read(p.join("queues.bin")).unwrap();let mut at=0;let cases=n32(&input,&mut at);let mut out=BufWriter::new(fs::File::create(p.join("results.bin")).unwrap());
 for _ in 0..cases{let active=input[at]!=0;at+=1;let count=n16(&input,&mut at)as i16;let margin=n16(&input,&mut at);let len=n16(&input,&mut at);let mut points=Vec::new();for _ in 0..len{let id=n32(&input,&mut at);let x=n32(&input,&mut at)as i32;let y=n32(&input,&mut at)as i32;let shade=input[at];let flags=input[at+1];at+=2;points.push(ProjectedFamily1{id,x,y,shade,flags});}let before=points.clone();
  match queue_family1(active,count,margin,&points){Err(_)=>out.write_all(&[1]).unwrap(),Ok(q)=>{out.write_all(&[0]).unwrap();out.write_all(&q.final_count.to_le_bytes()).unwrap();out.write_all(&(q.draw_order.len()as u16).to_le_bytes()).unwrap();for e in q.draw_order{out.write_all(&e.id.to_le_bytes()).unwrap();out.write_all(&e.x.to_le_bytes()).unwrap();out.write_all(&e.draw_y.to_le_bytes()).unwrap();out.write_all(&e.projected_y.to_le_bytes()).unwrap();out.write_all(&e.bucket.to_le_bytes()).unwrap();out.write_all(&e.residual.to_le_bytes()).unwrap();out.write_all(&[e.shade,e.flags]).unwrap();}}}assert_eq!(points,before);
 }assert_eq!(at,input.len());
 for flags in 0..=255{for mode in 0..=255{for gate in [false,true]{let p=family1_draw_packet(gate,mode,flags);out.write_all(&[p.draws as u8,p.dirty_class.unwrap_or(0),p.flags_after]).unwrap();}}}
 for flags in 0..=255{for shade in 0..=255{for countdown in [-128,-1,0,1,127]{for gate in [false,true]{for old in [false,true]{admission(&mut out,AdmissionPacketInput{native_type_at_6:2,context_flags_at_1c:0,context_gate_before:gate,shade_at_17:shade,countdown_at_1d6:countdown,flags_at_15:flags,old_frame_present:old});}}}}}
 for countdown in -128..=127{for flags in 0..=255{for shade in [0,1,255]{for gate in [false,true]{for old in [false,true]{admission(&mut out,AdmissionPacketInput{native_type_at_6:2,context_flags_at_1c:0,context_gate_before:gate,shade_at_17:shade,countdown_at_1d6:countdown,flags_at_15:flags,old_frame_present:old});}}}}}
 for native_type in 0..=255{for context in 0..=255{for gate in [false,true]{for old in [false,true]{admission(&mut out,AdmissionPacketInput{native_type_at_6:native_type,context_flags_at_1c:context,context_gate_before:gate,shade_at_17:255,countdown_at_1d6:127,flags_at_15:0x81,old_frame_present:old});}}}}
 out.flush().unwrap();}
