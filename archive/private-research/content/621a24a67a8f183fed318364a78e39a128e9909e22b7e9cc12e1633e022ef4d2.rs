extern crate genewars_runtime;pub use genewars_runtime::blueprint;use genewars_runtime::creature_world::{CreatureError,InitialMuleChoice,CreatureRngPurpose,CreatureRngDraw};type CreatureRecord=[u8;124];fn motion_word(r:&[u8],p:usize)->i16 {i16::from_le_bytes([r[p],r[p+1]])}fn motion_put(r:&mut[u8],p:usize,v:i16){r[p..p+2].copy_from_slice(&v.to_le_bytes());}fn draw(rng:&mut crate::blueprint::BlueprintRng,purpose:CreatureRngPurpose,trace:&mut Vec<CreatureRngDraw>)->u8 {
    let before=rng.seed;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
    let value=(rng.seed>>8)as u8;trace.push(CreatureRngDraw{purpose,seed_before:before,seed_after:rng.seed,value});value
}pub fn plan_mule_feeding_query_prefix(before:&CreatureRecord,nutrition:u8,rng_before:crate::blueprint::BlueprintRng,retired:bool)->Result<InitialMuleChoice,CreatureError>{
    plan_mule_plant_feed_inner(before,nutrition,rng_before,retired,true)
}fn plan_mule_plant_feed_inner(before:&CreatureRecord,nutrition:u8,rng_before:crate::blueprint::BlueprintRng,retired:bool,defer_fallback:bool)->Result<InitialMuleChoice,CreatureError> {
    if rng_before.ring>=32||before[6]!=1||before[0x18]!=4||before[0x5e]!=7||before[0x7a]!=4||before[0x76]>90||before[0x61]!=0x40||before[0x60]!=8
       ||before[0x57]!=2||motion_word(before,0x6c)&15!=0{return Err(CreatureError::UnsupportedMotionState);}
    let gain=i16::from(nutrition)*2;let reserve=motion_word(before,0x68).wrapping_add(gain);let energy=if before[0x77]==255 {motion_word(before,0x66)}else{motion_word(before,0x66).wrapping_sub(gain*2)};
    if energy<=2{return Err(CreatureError::UnsupportedMotionState);}
    let full=reserve>motion_word(before,0x62);
    let mut after=*before;motion_put(&mut after,0x68,if full{motion_word(before,0x62)}else{reserve});motion_put(&mut after,0x66,energy);if full{after[0x61]&=0xbf;}
    let mut rng=rng_before;let mut draws=Vec::new();draw(&mut rng,CreatureRngPurpose::NeighborhoodX,&mut draws);draw(&mut rng,CreatureRngPurpose::NeighborhoodY,&mut draws);
    if retired{after[0x57]=0;}
    if (retired||full)&&!defer_fallback {
        //724C0 clears the selected target kind.5CC60 then resets the
        //state7 action and69500 consumes one idle-delay draw.
        after[0x57]=0;after[0x15]&=0xdf;after[0x33..0x37].fill(0);let flags=motion_word(&after,0x60)as u16&0xd71a;motion_put(&mut after,0x60,flags as i16);after[0x5d]=0;
        let delay=i16::from(draw(&mut rng,CreatureRngPurpose::IdleDelay,&mut draws)%20)+10;after[0x5e]=8;motion_put(&mut after,0x6c,delay);
    }
    Ok(InitialMuleChoice{before:*before,after,rng_before,rng_after:rng,draws,sampled_points:Vec::new(),chosen_target:None,queue_requested:true})
}use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(131){let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[124..126].try_into().unwrap()),ring:c[126],draw_count:u32::from_le_bytes(c[127..131].try_into().unwrap())};let q=plan_mule_feeding_query_prefix(c[..124].try_into().unwrap(),8,rng,true).unwrap();out.extend(q.after);out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());}std::io::stdout().write_all(&out).unwrap();}