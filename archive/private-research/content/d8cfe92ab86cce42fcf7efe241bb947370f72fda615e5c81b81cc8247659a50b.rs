use genewars_runtime::actor_activation::{
    plan_associated_actor_activation, sample_placement_candidate, ActivationPlanningOutcome,
    CandidateEnvironment, ResolvedCellHead, TrigTables,
};
use std::{env, fs};

fn take<'a>(b: &'a [u8], p: &mut usize, n: usize) -> &'a [u8] {
    let s = *p; let e = s.checked_add(n).expect("input cursor overflow");
    let v = b.get(s..e).expect("truncated case input"); *p = e; v
}
fn u8v(b: &[u8], p: &mut usize) -> u8 { take(b,p,1)[0] }
fn u16v(b: &[u8], p: &mut usize) -> u16 { u16::from_le_bytes(take(b,p,2).try_into().unwrap()) }
fn i16v(b: &[u8], p: &mut usize) -> i16 { i16::from_le_bytes(take(b,p,2).try_into().unwrap()) }
fn i32v(b: &[u8], p: &mut usize) -> i32 { i32::from_le_bytes(take(b,p,4).try_into().unwrap()) }
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
fn head_cell(point: [u16;2]) -> usize { usize::from((point[1] >> 8) & 127) * 128 + usize::from((point[0] >> 8) & 127) }

fn main() {
    let a: Vec<String> = env::args().collect();
    let input = fs::read(&a[1]).unwrap();
    let sine = fs::read(&a[2]).unwrap(); let cosine = fs::read(&a[3]).unwrap();
    let tables = TrigTables::from_mapped_bytes(&sine,&cosine).unwrap();
    let mut p=0usize; let count=u16v(&input,&mut p) as usize;
    let mut lines=Vec::new();
    for ci in 0..count {
        let actor_id=i16v(&input,&mut p); let record_id=i16v(&input,&mut p); let slot=u8v(&input,&mut p) as usize;
        let center=[u16v(&input,&mut p),u16v(&input,&mut p)]; let seed=u16v(&input,&mut p);
        let obstacle=u8v(&input,&mut p)!=0; let flags71=u8v(&input,&mut p);
        let flags57=u8v(&input,&mut p); let state82=u8v(&input,&mut p);
        let avoid=u8v(&input,&mut p); let category=u8v(&input,&mut p);
        let bp_count=u32::from_le_bytes(take(&input,&mut p,4).try_into().unwrap());
        let head_id=i16v(&input,&mut p); let head_kind=u8v(&input,&mut p);
        let nenv=u16v(&input,&mut p) as usize;
        let mut envs=Vec::with_capacity(nenv);
        for _ in 0..nenv {
            let terrain=u8v(&input,&mut p); let blocker=i32v(&input,&mut p);
            let has=u8v(&input,&mut p)!=0; let resolved_id=i16v(&input,&mut p); let resolved_kind=u8v(&input,&mut p);
            envs.push(CandidateEnvironment { terrain_class:terrain, blocker_weight:blocker,
                resolved_cell_head: if has {Some(ResolvedCellHead {record_id:resolved_id,record_type:resolved_kind})} else {None} });
        }
        let mut actor=core::array::from_fn::<_,200,_>(|i| (i.wrapping_mul(17).wrapping_add(ci.wrapping_mul(7))) as u8);
        actor[6]=3; actor[0x0f..0x11].copy_from_slice(&record_id.to_le_bytes());
        actor[0x53]=avoid; actor[0x70..0x72].copy_from_slice(&0x0400u16.to_le_bytes());
        actor[0x71]=0x04 | flags71; actor[0x57]=flags57; actor[0x82]=state82; actor[0x74]=0;
        let mut blueprints=vec![0u8;32*36]; let bstart=0usize;
        blueprints[bstart+0x16]=category; blueprints[bstart+0x18]=9;
        blueprints[bstart+0x1c..bstart+0x20].copy_from_slice(&bp_count.to_le_bytes());
        let mut map=vec![0u8;128*128*14];
        let center_cell=head_cell(center); let headoff=center_cell*14+6;
        map[headoff..headoff+2].copy_from_slice(&head_id.to_le_bytes()); map[headoff+2]=head_kind;
        // Destination cells carry a decoy head to prove the plan uses the
        // associated object's center cell for insertion.
        for i in 0..nenv {
            if let Ok(c)=sample_placement_candidate(seed,center,&tables) {
                let cell=head_cell(c.position_u15); let off=cell*14+6;
                if cell!=center_cell { map[off..off+3].copy_from_slice(&[0x63,0,4]); }
            }
        }
        let result=plan_associated_actor_activation(actor_id,slot,&actor,&blueprints,&map,center,seed,&tables,&envs,obstacle);
        match result {
            Err(e) => lines.push(format!("{ci}|ERR|{e:?}")),
            Ok(ActivationPlanningOutcome::Ineligible {..}) => lines.push(format!("{ci}|INELIGIBLE")),
            Ok(ActivationPlanningOutcome::Planned(plan)) => {
                let attempts=plan.candidate_attempts.iter().map(|r| format!("{},{},{},{},{},{}",
                    r.candidate.seed_before,r.candidate.seed_after,r.candidate.angle_index,
                    r.candidate.position_u15[0],r.candidate.position_u15[1],u8::from(r.accepted)))
                    .collect::<Vec<_>>().join(";");
                let linkprev=plan.link.old_head_previous_after.map(|v|hex(&v)).unwrap_or_else(||"-".into());
                lines.push(format!("{ci}|OK|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                    plan.seed_after,plan.link.cell_index,attempts,hex(&plan.actor_after),hex(&plan.blueprint_after),
                    hex(&plan.link.expected_old_head),hex(&plan.link.new_cell_head),linkprev,
                    hex(&plan.link.actor_next_after)));
            }
        }
    }
    assert_eq!(p,input.len(),"unconsumed case input");
    fs::write(&a[4],lines.join("\n")+"\n").unwrap();
}
