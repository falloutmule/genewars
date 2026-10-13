use genewars_runtime::{blueprint::BlueprintRng,plant_spread::{SpreadDirections,SpreadQuery,SpreadRejection,plan_spread_target}};
use std::{fs,path::Path,io::Write};
fn word(b:&[u8],p:usize)->i16{i16::from_le_bytes([b[p],b[p+1]])}
fn main(){
    let root=std::env::args().nth(1).unwrap();let root=Path::new(&root);
    let work=std::env::args().nth(2).unwrap();let work=Path::new(&work);
    let mapped=fs::read(root.join("private/ghidra/validated/memory-.object3.bin")).unwrap();
    let directions=SpreadDirections::from_mapped_bytes(&mapped[0x4564..0x4588]).unwrap();
    let mut output=fs::File::create(work.join("actual.bin")).unwrap();
    for c in fs::read(work.join("input.bin")).unwrap().chunks_exact(23) {
        let mode=c[5];let threshold=word(c,13);let mut queries=Vec::new();
        let result=plan_spread_target([word(c,0),word(c,2)],c[4],core::array::from_fn(|i|c[15+i]as i8),threshold,&directions,
            BlueprintRng{seed:word(c,6)as u16,ring:c[8],draw_count:u32::from_le_bytes(c[9..13].try_into().unwrap())},|q|{
                let (tag,point,value)=match q {
                    SpreadQuery::Terrain(p)=>(0,p,if mode==1{1}else{0}),
                    SpreadQuery::PlantCount(p)=>(1,p.map(i16::from),if mode==2{1}else if mode==6{-1}else{0}),
                    SpreadQuery::CellFlags(p)=>(2,p.map(i16::from),if mode==3{1}else{0}),
                    SpreadQuery::Height(p)=>(3,p,i32::from(threshold)+if mode==4{512}else if mode==5{511}else{513}),
                };queries.push((tag,point));Ok::<i32,()>(value)
            }).unwrap();
        output.write_all(&[u8::from(result.factory_point.is_some()),match result.rejection{None=>0,Some(SpreadRejection::Terrain)=>1,Some(SpreadRejection::Occupied)=>2,Some(SpreadRejection::Obstacle)=>3,Some(SpreadRejection::Height)=>4}]).unwrap();
        for p in [result.walked_point,result.factory_point.unwrap_or([0,0])] {for v in p{output.write_all(&v.to_le_bytes()).unwrap();}}
        output.write_all(&result.rng_after.seed.to_le_bytes()).unwrap();output.write_all(&[result.rng_after.ring]).unwrap();output.write_all(&result.rng_after.draw_count.to_le_bytes()).unwrap();
        output.write_all(&[queries.len()as u8]).unwrap();
        for (tag,point)in queries{output.write_all(&[tag]).unwrap();for v in point{output.write_all(&v.to_le_bytes()).unwrap();}}
    }
}
