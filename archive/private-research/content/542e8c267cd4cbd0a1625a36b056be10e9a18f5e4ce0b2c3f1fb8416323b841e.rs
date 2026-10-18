// Bounded numeric adapter for child20 hitting an unlinked, neutral, idle Mule.
// The factory and cell-link operation remain caller responsibilities.
fn word(a:&[u8],i:usize)->u16{u16::from_le_bytes([a[i],a[i+1]])}
fn put(a:&mut[u8],i:usize,v:u16){a[i..i+2].copy_from_slice(&v.to_le_bytes())}
fn cancel(a:&mut[u8]){put(a,51,0);put(a,53,0);a[21]&=!32;put(a,96,word(a,96)&0xd71a);a[93]=0;a[87]=0;put(a,108,0)}
fn step(seed:&mut u16,ring:&mut u8,count:&mut u32)->u16{*seed=seed.wrapping_mul(0xe62d);*ring=ring.wrapping_add(1)&31;*count=count.wrapping_add(1);*seed>>8}
fn impact(before:&[u8],mut seed:u16,mut ring:u8)->Result<Vec<u8>,String>{
 if before.len()!=124||before[6]!=1||before[24]!=4||before[122]!=4||before[94]!=8||word(before,96)&2!=0||(word(before,106)as i16)<=1{return Err("unsupported child20 recipient".into())}
 let mut a=before.to_vec();let mut count=0u32;
 // Query setup consumes two draws, whether or not a recipient exists.
 step(&mut seed,&mut ring,&mut count);step(&mut seed,&mut ring,&mut count);
 // Caller inserts a slot0 effect at this cell's head; this is the only link edit.
 a[0]=0;a[1]=0;a[2]=5;
 cancel(&mut a);put(&mut a,106,word(before,106).wrapping_sub(1));a[94]=20;put(&mut a,108,32767);put(&mut a,85,0);a[87]=5;
 let audio=if step(&mut seed,&mut ring,&mut count)%2!=0{8}else{28+(step(&mut seed,&mut ring,&mut count)%3)as u8};
 // 5C7E0 cancels the just-recorded hit target, resets movement flags, and
 // enters the source reaction with the building-origin duration of64.
 cancel(&mut a);a[97]&=127;put(&mut a,63,0);let flags=word(&a,73)&0xefe8;put(&mut a,73,flags);a[84]=0;a[22]&=63;a[94]=18;put(&mut a,108,64);a[74]&=0x3b;a[21]&=!32;
 a.extend_from_slice(&seed.to_le_bytes());a.push(ring);a.extend_from_slice(&count.to_le_bytes());a.extend_from_slice(&[3,audio,57,57,0]);Ok(a)
}
fn main(){let p=std::env::args().nth(1).unwrap();let input=std::fs::read(format!("{p}/damage-input.bin")).unwrap();let mut out=Vec::new();for row in input.chunks_exact(127){out.extend(impact(&row[..124],word(row,124),row[126]).unwrap())}std::fs::write(format!("{p}/damage-actual.bin"),out).unwrap();}
