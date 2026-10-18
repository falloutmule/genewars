use genewars_runtime::{target_resolver::{resolve_target,TargetResolverError},actor_activation::TrigTables,physics_tables::PhysicsTables};
use std::{fs,path::Path,io::Write};
fn word(b:&[u8],p:usize)->i16{i16::from_le_bytes([b[p],b[p+1]])}
fn main(){
    let root=std::env::args().nth(1).unwrap();let root=Path::new(&root);let work=root.join("private/review/target-resolver");
    let runtime=root.join("private/integration/game-runtime/physics");
    let physics=PhysicsTables::from_mapped_bytes(&fs::read(runtime.join("angle.bin")).unwrap(),&fs::read(runtime.join("bounce.bin")).unwrap()).unwrap();
    let trig=TrigTables::generate_pinned().unwrap();let mut out=fs::File::create(work.join("actual.bin")).unwrap();
    for case in fs::read(work.join("input.bin")).unwrap().chunks_exact(11) {
        let current=[word(case,0),word(case,2)];let requested=[word(case,4),word(case,6)];let mode=case[8];let limit=word(case,9)as u16;
        let mut queries=Vec::new();
        let result=resolve_target(current,requested,&physics,&trig,|point| {
            queries.push(point);
            Ok::<_,()>(match mode {0=>true,1=>queries.len()>usize::from(limit),2=>false,3=>((i32::from(point[0])^i32::from(point[1]))&255)<32,_=>unreachable!()})
        });
        let (status,point)=match result {Ok(result)=>{assert_eq!(result.queried_points as usize,queries.len());(0,result.point)},Err(TargetResolverError::IterationLimit)=>(1,[0,0]),Err(_)=>unreachable!()};
        out.write_all(&[status]).unwrap();out.write_all(&(queries.len()as u16).to_le_bytes()).unwrap();
        for point in std::iter::once(point).chain(queries) {for value in point {out.write_all(&value.to_le_bytes()).unwrap();}}
    }
}
