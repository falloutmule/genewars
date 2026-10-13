#[path="../rust/crates/genewars-runtime/src/countdown_expiry.rs"]mod countdown_expiry;
use countdown_expiry::*;
use std::{fs,path::Path};
fn main(){
    let arg=std::env::args().nth(1).unwrap();let work=Path::new(&arg);let input=fs::read(work.join("input.bin")).unwrap();let mut result=Vec::new();
    for row in input.chunks_exact(8){
        let count=u16::from_le_bytes(row[0..2].try_into().unwrap());let period=u16::from_le_bytes(row[2..4].try_into().unwrap());
        let plan=plan_countdown_transition(count,period,row[4]as i8,row[5],row[6],row[7]==1);
        result.extend(plan.after.to_le_bytes());result.push(plan.started as u8);result.push(plan.expired as u8);
        result.push(transition_requests(plan,0).len()as u8);
    }
    assert_eq!(animation_requests().len(),8);fs::write(work.join("actual.bin"),result).unwrap();
}
