use genewars_runtime::plant_spread::{plan_neutral_plant_initialization,plant_factory_position};
use std::{fs,path::Path,io::Write};
fn main(){
    let path=std::env::args().nth(1).unwrap();let work=Path::new(&path);let mut out=fs::File::create(work.join("actual.bin")).unwrap();
    for c in fs::read(work.join("input.bin")).unwrap().chunks_exact(61){
        let prior=c[..49].try_into().unwrap();let slot=u16::from_le_bytes(c[49..51].try_into().unwrap());let selector=c[51];
        let requested=[i16::from_le_bytes(c[52..54].try_into().unwrap()),i16::from_le_bytes(c[54..56].try_into().unwrap())];
        let height=i32::from_le_bytes(c[56..60].try_into().unwrap());let result=plan_neutral_plant_initialization(&prior,slot,selector,requested,height,c[60]).unwrap();
        assert_eq!(result.position,plant_factory_position(requested));out.write_all(&result.after).unwrap();
        for _ in 0..2 {for v in result.position{out.write_all(&v.to_le_bytes()).unwrap();}}
    }
}
