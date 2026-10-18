//! Source8C550 height assignment and ordered8BEE0 repairs.
//! Numeric planning does not resolve map shading, viewport dirtiness or live
//! object height refresh. Execute every step in order inside one transaction.
use crate::construction_terrain::{self,TerrainHeightMutation,TerrainJobError};
use crate::blueprint::BlueprintRng;
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum TerrainHeightError{MapLength,Cell(u16),Ring(u8),Slope(TerrainJobError)}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum TerrainHeightStep{
    SetHeight(TerrainHeightMutation),
    ///84960: its viewport work and four8B800 corner shade refreshes.
    RefreshVertex{position:[i16;2]},
    RngDraw{journal_line:u16,seed_before:u16,seed_after:u16,ring_slot:u8},
    ///8C290, before84B40 and before the next slope mutation.
    RefreshSlopeShade{cell:u16,source_index:i32},
    ///84B40, after this cell's8C290.
    RefreshSlopeViewport{position:[i16;2]},
    ///8B840 on the actual linked cell; never substitute an empty list.
    RefreshLinkedObjects{cell:u16},
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TerrainHeightPlan{pub requested:i16,pub assigned:i16,pub steps:Vec<TerrainHeightStep>,pub rng_after:BlueprintRng,pub returned_height_delta:i16}
pub fn plan_height_change(map:&[u8],index:u16,requested:i16,rng:BlueprintRng)->Result<TerrainHeightPlan,TerrainHeightError>{
    if map.len()!=229376{return Err(TerrainHeightError::MapLength);}if index>=16384{return Err(TerrainHeightError::Cell(index));}if rng.ring>=32{return Err(TerrainHeightError::Ring(rng.ring));}
    let position=|cell:u16|[((cell&127)<<8)as i16,((cell>>7)<<8)as i16];
    let mut work=map.to_vec();let at=usize::from(index)*14+2;let before=i16::from_le_bytes([map[at],map[at+1]]);let assigned=requested.max(0);
    work[at..at+2].copy_from_slice(&assigned.to_le_bytes());
    let mut steps=vec![TerrainHeightStep::SetHeight(TerrainHeightMutation{cell:index,before,after:assigned}),TerrainHeightStep::RefreshVertex{position:position(index)}];
    let slope=construction_terrain::plan_terrain_slope_repair(&work,index,rng.seed).map_err(TerrainHeightError::Slope)?;
    let mut next=rng;let mut previous_line=0;let mut diagonal_step=0i32;
    for (change,line)in slope.heights.into_iter().zip(slope.rng_journal_lines){
        if line!=previous_line{diagonal_step=0;previous_line=line;}diagonal_step+=1;
        let direction=if line==0x3c1{-1}else{1};let sx=i32::from(index&127)+direction*diagonal_step;let sy=i32::from(index>>7)+direction*diagonal_step;
        let old=next.seed;next.seed=old.wrapping_mul(0xe62d);
        steps.push(TerrainHeightStep::RngDraw{journal_line:line,seed_before:old,seed_after:next.seed,ring_slot:next.ring});
        next.ring=(next.ring+1)&31;next.draw_count=next.draw_count.wrapping_add(1);
        steps.push(TerrainHeightStep::SetHeight(change));steps.push(TerrainHeightStep::RefreshSlopeShade{cell:change.cell,source_index:sy*128+sx});steps.push(TerrainHeightStep::RefreshSlopeViewport{position:[(sx*256)as i16,(sy*256)as i16]});
        let at=usize::from(change.cell)*14+2;work[at..at+2].copy_from_slice(&change.after.to_le_bytes());
    }
    let x=i32::from(index&127);let y=i32::from(index>>7);
    for(dx,dy)in[(0,0),(-1,0),(-1,-1),(0,-1)]{steps.push(TerrainHeightStep::RefreshLinkedObjects{cell:(((y+dy)&127)*128+((x+dx)&127))as u16});}
    let final_height=i16::from_le_bytes([work[at],work[at+1]]);
    Ok(TerrainHeightPlan{requested,assigned,steps,rng_after:next,returned_height_delta:final_height.wrapping_sub(assigned)})
}
