use genewars_runtime::bookkeeping::{plan_empty_building_player,plan_empty_family8_countdown};
use std::{fs,path::Path,io::Write};
fn main(){
    let arg=std::env::args().nth(1).unwrap();let p=Path::new(&arg);
    let input=fs::read(p.join("players-input.bin")).unwrap();let mut out=fs::File::create(p.join("players-actual.bin")).unwrap();
    for case in input.chunks_exact(3+4*1805) {
        let period=u16::from_le_bytes([case[1],case[2]]);
        for raw in case[3..].chunks_exact(1805) {out.write_all(&plan_empty_building_player(raw.try_into().unwrap(),case[0],period).unwrap().after).unwrap();}
    }
    let mut out=fs::File::create(p.join("countdown-actual.bin")).unwrap();
    for c in fs::read(p.join("countdown-input.bin")).unwrap().chunks_exact(7) {
        let r=plan_empty_family8_countdown(u16::from_le_bytes([c[0],c[1]]),u16::from_le_bytes([c[2],c[3]]),c[4]as i8,c[5],c[6]);
        match r {Ok(r)=>out.write_all(&[0,r.after as u8,(r.after>>8)as u8,u8::from(r.started)]).unwrap(),Err(_)=>out.write_all(&[1,0,0,0]).unwrap()}
    }
}
