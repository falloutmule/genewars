use std::io::{self, Read, Write};
#[path="building.rs"] mod building;
#[path="building_pool.rs"] mod building_pool;
use building::BuildingAnimationBinding;
use building_pool::{BuildingPool, BuildingPoolError};

fn take<'a>(b: &'a [u8], p: &mut usize, n: usize) -> &'a [u8] {
    let e = p.checked_add(n).expect("input offset overflow");
    let s = b.get(*p..e).expect("truncated harness input"); *p = e; s
}
fn u32le(b: &[u8], p: &mut usize) -> usize {
    u32::from_le_bytes(take(b,p,4).try_into().unwrap()) as usize
}
fn main() {
    let mut input=Vec::new(); io::stdin().read_to_end(&mut input).unwrap();
    let mut p=0usize; let cases=u32le(&input,&mut p); let mut out=io::BufWriter::new(io::stdout().lock());
    for _ in 0..cases {
        let plen=u32le(&input,&mut p); let packed=take(&input,&mut p,plen);
        let prior_bytes=take(&input,&mut p,256*67);
        let bindings_bytes=take(&input,&mut p,256);
        let prior:[[u8;67];256]=std::array::from_fn(|i| prior_bytes[i*67..(i+1)*67].try_into().unwrap());
        let bindings:[Option<BuildingAnimationBinding>;256]=std::array::from_fn(|i| {
            if bindings_bytes[i]==0 {None} else {Some(BuildingAnimationBinding{selector:bindings_bytes[i]})}
        });
        match BuildingPool::from_packed_inactive(packed,&prior,bindings) {
            Ok(pool) => { out.write_all(&[0]).unwrap(); for r in pool.records() {out.write_all(r).unwrap();} }
            Err(BuildingPoolError::PackedLength{..}) => out.write_all(&[1]).unwrap(),
            Err(BuildingPoolError::ActivePackedRecord(i)) => {out.write_all(&[2]).unwrap();out.write_all(&(i as u16).to_le_bytes()).unwrap();}
            Err(BuildingPoolError::InvalidBinding(i)) => {out.write_all(&[3]).unwrap();out.write_all(&(i as u16).to_le_bytes()).unwrap();}
            Err(_) => out.write_all(&[4]).unwrap(),
        }
    }
    assert_eq!(p,input.len(),"unused harness input"); out.flush().unwrap();
}
