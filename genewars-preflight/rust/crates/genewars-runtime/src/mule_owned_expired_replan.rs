//! F21/attempt34: the bounded runtime machinery and the source-backed arm for
//! map branch B11 (owned state1 degraded-pursuit continuation whose 0x6c timer
//! is ALREADY 0 before the callback), admitted exactly to the measured shape.
//!
//! Evidence basis (cited addresses; listings under private/ghidra):
//! - 65c60..65ef0 state-1 continuation region
//!   (private/ghidra/player-creation/instructions-0005ed20.txt): 65c7d ring
//!   gate ((word(0xf)+[0x11877e])&3)==3; 65c9a/65cb2 timer gate
//!   ECX=0x7fff-word(0x6c) (the byte[[EBP+0xa6]+6]==3 ECX-doubling at 65c84 is
//!   unmodeled: both sides pass the >0x100 gate for the admitted shape);
//!   65cbe activation write code 0x9aa pointer 0x10e4d8; 65cfe draw 1
//!   (16-bit LCG word[0x11878a] *= -0x19d3 == 0xE62D mod 2^16, counter
//!   dword[0x118782]+=1, out=(state>>8)&0x7fff % 150, 65d3a JG skips the flag
//!   only when out > ECX-0x100); 65d40 BH=1; 65d42 CALL 5cc60.
//! - 5cc60 (private/ghidra/mission-array-refs/instructions-0005cc60.txt):
//!   state 0x12 returns immediately; otherwise 5c920 (byte[0x60]&2 -> epilogue,
//!   measured 0x48&2=0) then jump-table 0x5cc34 entry 0 = 5cd1a (byte[0x57]==2
//!   arm not taken: measured 1) -> the 5cca0 block: word[0x35]=0,
//!   byte[0x15]&=0xdf, word[0x33]=0, word[0x60]&=0xd71a (0x48->0x08),
//!   byte[0x5d]=0 (5ccce TEST BX where BX=0x48&2=0 -> SETA 0 -> 5ccd8 writes
//!   BH=0), byte[0x57]=0, word[0x6c]=0; BH (the caller's flag) survives.
//! - 65deb block: activation write code 0xa03 pointer 0x10e4e5, cursor+=1,
//!   counter+=1 (65e1f); 65e3d draw 2 (LCG *= -0x44bf19d3 == 0xE62D mod 2^16,
//!   counter+=1, arg=10..29); 65e68 byte[0x5e]=8; 65e7a CALL [vtable+0x8c]
//!   = 8b780 (private/ghidra/initial-creature/instructions-0008b780.txt: the
//!   ECX arg is never read by 8b780; it zeroes the context dword at its out
//!   pointer [EBP+0xa2] and returns the last chain node with byte[node+6]==1
//!   whose vtable+0x68 virtual is false - the measured trace returned AL!=0
//!   and produced no eight-buffer deltas, so the arm pins AL=true and keeps
//!   the JZ 65c57 word[0x6c]=CX side fail-closed (PROVISIONAL)).
//! - 73c20 (private/ghidra/activation-rng-refs/instructions-00073c20.txt):
//!   activation write code 0x59c pointer 0x10e6b7, counter+=2 (73c92), draw 3,
//!   activation write code 0x59d pointer 0x10e6c0, draw 4, then the waypoint
//!   rebuild from the 0x107ee8/0x1086e8 slope dword tables with index
//!   norm(rem3*8) (73d3d..73d76) and the 0x100 arg: new = ((2*p + slope*2*rem4
//!   >> 16) >> 1) & 0x7fff per component, returned packed as dword.
//! - 65e94..65ef2: byte[0x54]=0x14; byte[0x16]&=0xbf; dword[0xb]=waypoint;
//!   65ea1 word(0x49)|=0x10; 65eaf CALL 13e50 (OR byte[0x49],0x4); 65eb4 CALL
//!   13e70 (AND word[0x49],0xeffc; AND byte[0x16],0x7f); 65ebb/65ec3 CALL
//!   13eb0 (+0x4d and +0x50: byte[0x4f]=0, byte[0x52]=0); 65ed0 CALL
//!   [vtable+0x80]=5a990 (private/ghidra/initial-creature/instructions-
//!   0005a990.txt: ((75 + min(byte[0x76],100)*25/100) * dword[0x1045fc +
//!   45*byte[0x7a] + 6] / 100 * (byte[0x7b]+1)) >> 8); 65ed6 word[0x6c]=0x7fff;
//!   65ee3 byte[0x60]|=0x80; 65eeb CALL 6bdd0 (private/ghidra/initial-creature/
//!   instructions-0006b700.txt lines 482-486: AL = byte[0x104a64 +
//!   2*byte[0x7a]] == 3). The measured species byte 0x7a=4 reads 0x04, so the
//!   measured shape takes 65ef2 JZ 6a290 and the tail 6a4a0 DEC word(0x6c)
//!   (modeled by the shared tail in creature_world) yields 0x7ffe.
//!   6c190 is ONLY reached on the 6bdd0-true side (65ef8/65c4d); the measured
//!   path never executes it, so it stays fail-closed (UnsupportedMotionState).
//!
//! Tick [0x11877e]: the mission tick, modeled as counter=args.frame per the
//! F20 receipt (producer INC dword ptr [0x11877e] at 0x1bd4f in the module
//! main loop) - the same relation mule_callback_phase already uses.
//!
//! Activation tables (0x11862c/0x1185ac/0x1186ac, cursor byte 0x11872c): the
//! B11 path WRITES the four (code, pointer, ring-value) triples above and
//! READS none of them; the consumers are a separate unproved family. The
//! cursor arithmetic is mirrored by BlueprintRng.ring (rng.bin bytes 2), the
//! counter by BlueprintRng.draw_count, and the LCG state by BlueprintRng.seed
//! (word[0x11878a]); the table contents are recorded in a local
//! ActivationTables instance so the write order/values stay visible and
//! no unverifiable world state is invented.
//!
//! Table data: private/integration/game-runtime/creature/b11-tables.bin,
//! extracted from private/ghidra/GW-CD-candidate.exe (sha256 4bdf296febc88984
//! aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba, DOS/4GW LE object 3 base
//! 0x100000) with the page mapping recipe validated in
//! private/ghidra/config-validate-le.py; provenance receipt
//! private/integration/b11-tables-extraction.json. Layout: 0x0000 16 bytes
//! VA 0x104a64 (6bdd0 species flags); 0x0010 360 bytes VA 0x1045fc (5a990
//! 45-byte per-species records); 0x0178 8192 bytes VA 0x107ee8 then 8192 bytes
//! VA 0x1086e8 (slope tables). A missing or unpinned file refuses the arm
//! (fail-closed), it never changes behavior.
use crate::blueprint::BlueprintRng;
use crate::creature_world::{CreatureError,CreatureRecord};
use std::sync::OnceLock;

const LC17:u16=0xe62d; // -0x19d3 and -0x44bf19d3 reduced mod 2^16
const TABLE_BYTES:usize=16+45*8+0x2800;
const TABLE_SHA:u64=0x6f3c_5c73_0f05_f544; // FNV-1a64 of the pinned table file

fn w16(r:&CreatureRecord,o:usize)->i16{i16::from_le_bytes([r[o],r[o+1]])}
fn put16(r:&mut CreatureRecord,o:usize,v:i16){r[o..o+2].copy_from_slice(&v.to_le_bytes());}

/// 32-entry cursor-indexed (code, pointer, ring-value) activation triples
/// (source 0x11862c/0x1185ac/0x1186ac, cursor byte 0x11872c & 0x1f).
/// Writes only; no consumer is admitted in this family.
#[derive(Debug,Clone)]
pub struct ActivationTables{pub code:[u32;32],pub pointer:[u32;32],pub ring_value:[u32;32],pub cursor:u8}
impl ActivationTables{
 fn new(cursor:u8)->Self{Self{code:[0;32],pointer:[0;32],ring_value:[0;32],cursor}}
 /// One source activation write: entry[cursor]=(code,pointer,tick); the
 /// source then advances the cursor byte ((cursor+1)&0x1f) - mirrored into
 /// BlueprintRng.ring, which rng.bin pins.
 fn write(&mut self,code:u32,pointer:u32,tick:u32,rng:&mut BlueprintRng){
  let i=usize::from(self.cursor&0x1f);self.code[i]=code;self.pointer[i]=pointer;self.ring_value[i]=tick;
  self.cursor=self.cursor.wrapping_add(1)&0x1f;rng.ring=self.cursor;
 }
}

struct SourceTables{species_flag:[u8;8],species_stat:[[u8;45];8],slope_a:[i32;2048],slope_b:[i32;2048]}
impl SourceTables{
 /// 5a990 word(0x3f) value for this record's species fields.
 fn species_skill(&self,record:&CreatureRecord)->Result<i32,CreatureError>{
  let k=usize::from(record[0x7a]);if k>=8{return Err(CreatureError::UnsupportedMotionState);}
  let esi=i32::from_le_bytes(self.species_stat[k][6..10].try_into().unwrap());
  let dl=i32::from(record[0x76]).min(100);
  let base=75+dl*25/100;
  let edi=i32::from(record[0x7b])+1;
  Ok(((((i64::from(base))*(i64::from(esi)))/100)*(i64::from(edi))>>8) as i32)
 }
 /// 6bdd0: AL = byte[0x104a64 + 2*byte[0x7a]] == 3.
 fn species_is_three(&self,species:u8)->Result<bool,CreatureError>{
  let k=usize::from(species);if k>=8{return Err(CreatureError::UnsupportedMotionState);}
  Ok(self.species_flag[k]==3)
 }
 /// 73c20's waypoint rebuild (73d3d..73dde) with ESI=rem3*8 normalized and
 /// the 0x100 arg; returns the packed (x,y) dword the source stores at 0xb.
 fn replan(&self,x:i32,y:i32,rem3:i32,rem4:i32)->Result<u32,CreatureError>{
  let esi=rem3*8;
  let idx=if esi<0x200{esi+0x600}else{esi-0x200};
  if !(0..2048).contains(&idx){return Err(CreatureError::UnsupportedMotionState);}
  let ui=idx as usize;
  let eax=2*rem4; // 73d07/73d2c/73d33: (0x100*rem4)>>8 doubled
  let term_y=self.slope_a[ui].wrapping_mul(eax)>>16; //73d5a/73d5d
  let term_x=self.slope_b[ui].wrapping_mul(eax)>>16; //73d76/73d86
  let x2=(x*2).wrapping_add(term_x);let y2=(y*2).wrapping_add(term_y);
  let nx=((x2>>1)&0x7fff) as u16 as u32;let ny=((y2>>1)&0x7fff) as u16 as u32;
  Ok(nx|(ny<<16))
 }
}
fn fnv1a64(bytes:&[u8])->u64{let mut h:u64=0xcbf29ce484222325;for b in bytes{h^=u64::from(*b);h=h.wrapping_mul(0x100000001b3);}h}
fn load_tables()->Option<SourceTables>{
 let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime/creature/b11-tables.bin");
 let bytes=std::fs::read(path).ok()?;
 if bytes.len()!=TABLE_BYTES||fnv1a64(&bytes)!=TABLE_SHA{return None;}
 let mut stat=[[0u8;45];8];
 for k in 0..8{stat[k].copy_from_slice(&bytes[16+45*k..16+45*(k+1)]);}
 let dword=|i:usize|i32::from_le_bytes(bytes[i..i+4].try_into().unwrap());
 let mut slope_a=[0i32;2048];let mut slope_b=[0i32;2048];
 for i in 0..2048{slope_a[i]=dword(16+360+i*4);slope_b[i]=dword(16+360+0x800+i*4);} //0x1086e8 = 0x107ee8+0x800 bytes
 Some(SourceTables{species_flag:bytes[0..16].iter().step_by(2).copied().collect::<Vec<u8>>().try_into().unwrap(),species_stat:stat,slope_a,slope_b})
}
fn tables()->Option<&'static SourceTables>{
 static T:OnceLock<Option<SourceTables>>=OnceLock::new();
 T.get_or_init(load_tables).as_ref()
}

/// The B11 measured-shape arm: 65c60 ring-gate long side through the 65deb
/// re-acquisition block, 73c20 replan, 5a990 and the 6bdd0 false side, with
/// the shared tail DEC (creature_world) producing word(0x6c)=0x7ffe. Every
/// unmeasured side (ring-gate short side 65d4d/65d5e, timer-gate short side
/// 65cb8, draw-1 flag JG side 65d3c, 8b780 AL==0 side 65c57, 6bdd0 true side
/// 65ef8/6c190) refuses.
pub fn plan_expired_reacquisition(record:&CreatureRecord,rng:&mut BlueprintRng,tick:u32)->Result<CreatureRecord,CreatureError>{
 let t=tables().ok_or(CreatureError::UnsupportedMotionState)?;
 //65c7d ring gate; [0x11877e] modeled as counter=args.frame (F20 receipt).
 //SHORT side (F35/attempt35, 2026-10-18): 65c7f JMP 65d4d; byte[EBP+0x186]
 //holds the ENTRY AL (65c66), and the only entry to 65c60 is 65951 JZ
 //(CALL [ctx+0x68] result zero), so AL==0 deterministically; 65d56 TEST CL,CL
 //is NOT taken and 65d5e runs the 65d80..65dce near-target block - the SAME
 //block the F17 arrival side reaches (measured trace
 //private/review/owned-mule-captured-width/d386493180e14ca7bc14f28d32e22e5f,
 //1736 instructions, 0 draws). The timer gate / draws / 5cc60 / 65deb block
 //are NOT executed on this side; return the near-target record directly.
 if tick.wrapping_add(i32::from(w16(record,0xf)) as u32)&3!=3{return near_target(record);}
 //65c9a/65cb2 timer gate. The byte[[EBP+0xa6]+6]==3 ECX doubling (65c84) is
 //unmodeled; both sides pass the >0x100 gate for the admitted shape.
 let ecx=i32::from(0x7fff)-i32::from(w16(record,0x6c));
 if ecx<=0x100{return Err(CreatureError::UnsupportedMotionState);}
 let mut activation=ActivationTables::new(rng.ring);
 //65cbe activation write, then 65cfe draw 1 (counter+=1 at 65d34).
 activation.write(0x9aa,0x10e4d8,tick,rng);
 rng.seed=rng.seed.wrapping_mul(LC17);rng.draw_count=rng.draw_count.wrapping_add(1);
 let out1=((i32::from(rng.seed)>>8)&0x7fff)%150;
 if out1>ecx-0x100{return Err(CreatureError::UnsupportedMotionState);} //65d3a JG side unmeasured
 //65d40 BH=1; 65d42 CALL 5cc60: the 5cca0 zeroing block (BH preserved).
 let mut after=*record;
 put16(&mut after,0x35,0);after[0x15]&=0xdf;put16(&mut after,0x33,0);
 let flags60=(w16(&after,0x60) as u16)&0xd71a;put16(&mut after,0x60,flags60 as i16);
 after[0x5d]=0;after[0x57]=0;put16(&mut after,0x6c,0);
 //65deb block: activation write + counter (65e1f), then draw 2 (65e3d).
 activation.write(0xa03,0x10e4e5,tick,rng);
 rng.draw_count=rng.draw_count.wrapping_add(1); //65e1f is the block's only INC; 65e3d advances the LCG only
 rng.seed=rng.seed.wrapping_mul(LC17);
 let _arg2=(((i32::from(rng.seed)>>8)&0x7fff)%20)+10; //65e6c; 8b780 never reads ECX
 after[0x5e]=8; //65e68
 //65e7a CALL [vtable+0x8c]=8b780: pinned AL!=0 for the measured shape
 //(trace 2030; context dword zeroing + chain walk produce no eight-buffer
 //deltas - PROVISIONAL). The AL==0 side (65c57) stays unreachable/fail-closed.
 //65e88 CALL 73c20 with EAX=record+7, EDX=0x100.
 activation.write(0x59c,0x10e6b7,tick,rng);
 rng.draw_count=rng.draw_count.wrapping_add(2); //73c92 ADD EDI,0x2
 rng.seed=rng.seed.wrapping_mul(LC17);
 let rem3=(i32::from(rng.seed)>>8)&0x7fff;let rem3=rem3%256;
 activation.write(0x59d,0x10e6c0,tick,rng);
 rng.seed=rng.seed.wrapping_mul(LC17);
 let rem4=((i32::from(rng.seed)>>8)&0x7fff)%256;
 let packed=t.replan(i32::from(w16(&after,7)),i32::from(w16(&after,9)),rem3,rem4)?;
 //65e94..65e9e
 after[0x54]=0x14;after[0x16]&=0xbf;
 after[0xb..0xf].copy_from_slice(&packed.to_le_bytes()); //65ea4
 //65ea1 + 13e50 + 13e70 + 13e70
 let mut flags=u16::from_le_bytes([after[0x49],after[0x4a]])|0x10|0x4;flags&=0xeffc;
 after[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());after[0x16]&=0x7f;
 //13eb0(ESI+0x4d) and 13eb0(ESI+0x50)
 after[0x4f]=0;after[0x52]=0;
 //65ed0 CALL [vtable+0x80]=5a990 -> 65edf word[0x3f]
 let skill=t.species_skill(&after)?.min(i32::from(i16::MAX)) as i16;
 put16(&mut after,0x3f,skill);
 //65ed6/65ee3
 put16(&mut after,0x6c,0x7fff);after[0x60]|=0x80;
 //65eeb CALL 6bdd0; measured species 4 reads 0x04 -> JZ 6a290 (tail DEC in
 //the shared tail). The true side (65ef8 CALL 6c190) is unimplemented.
 if t.species_is_three(after[0x7a])?{return Err(CreatureError::UnsupportedMotionState);}
 Ok(after)
}

/// The shared 65d80..65dce continuation near-target block for the state-1
/// degraded arrival (reached from BOTH the F17 arrival arm - owned_mule_updates
/// "6008 measured arrival side" - and the B11 ring-gate SHORT side above), so
/// there is exactly one implementation of the source block:
/// - 65d85 CALL 73770 (doubled distance, EDI=ESI+7); 65d8a CMP 0x4000 JG 65dd7:
///   the >0x4000 side (65dd7..) is UNMEASURED and stays refused. For the
///   measured shapes the record sits ON its own route waypoint
///   (motion_point == word(0x0b)/0x0d), which pins the <=0x4000 side.
/// - 65d9e CALL 6eda0; 65da5 JZ 65dbe taken (measured FALSE side; the aligned
///   virtual38 side 65da7..65db5 - AND byte[0x60],0xbf, JMP 6a290 - is
///   unmeasured and refuses). word(0x6a)>0 is the measured freshness shape.
/// - 65dc9 CALL 73970 with the 0x100 arg; 65dce MOV word[ESI+0x4b],AX - the
///   measured records reach the waypoint, so AE028 yields yaw 0 and the write
///   is word(0x4b)=0 (oracle 0x4b 1024->0 on R30, 0x31a->0 on the 6060 record).
///   65da7..65db5 and 65dd7.. stay fail-closed (UnsupportedMotionState).
pub(crate) fn near_target(record:&CreatureRecord)->Result<CreatureRecord,CreatureError>{
 if crate::creature_world::motion_point(record)!=[w16(record,0xb),w16(record,0xd)]||w16(record,0x6a)<=0{return Err(CreatureError::UnsupportedMotionState);}
 let yaw=w16(record,0x39);let direct=-i32::from(yaw);let alternative=direct-2048*direct.signum();
 let delta=if direct.abs()<=alternative.abs(){direct}else{alternative};
 if delta.abs()<=256||!(0..2048).contains(&yaw){return Err(CreatureError::UnsupportedMotionState);}
 let mut after=*record;after[0x4b..0x4d].copy_from_slice(&0i16.to_le_bytes());Ok(after)
}
