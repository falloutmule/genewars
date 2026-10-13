//! Numeric 75B00 -> 84B40 -> 8B800 visibility update for radius4096.
//! Viewport invalidation/minimap callbacks remain explicit presentation work.
use super::*;
use genewars_core::cd_candidate::{plan_terrain_factor_raise_4096,interpolate_entity_shade,TerrainFactorChange};

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct RevealReceipt {
    pub changes:Vec<TerrainFactorChange>,
    /// Ordered 84B40 notification positions. Numeric dirty bits and entity
    /// shades are committed, but its viewport/minimap work is still pending.
    pub pending_presentation_vertices:Vec<[i16;2]>,
    /// Counts visits, including repeated visits after adjacent vertices.
    pub cell_visits:usize,
    pub plant_shade_writes:usize,
}

impl ActorWorld {
    pub fn apply_reveal_4096(&mut self,target:[i16;2])->Result<RevealReceipt,ActorWorldError> {
        self.validate_links()?;
        let changes=plan_terrain_factor_raise_4096(self.map(),target)
            .map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
        let mut staged=self.clone();
        let mut receipt=RevealReceipt{changes,pending_presentation_vertices:Vec::new(),cell_visits:0,plant_shade_writes:0};
        for change in &receipt.changes {
            let [x,y]=change.source_cell;
            let index=usize::from(y)*128+usize::from(x);
            if staged.backing[MAP_START+index*14+12]!=change.previous {return Err(ActorWorldError::StaleLink);}
            staged.backing[MAP_START+index*14+12]=change.next;
            receipt.pending_presentation_vertices.push(change.grid_position);
            // The callback updates these four cells in this precise order.
            for [cx,cy] in [[x,y],[x.wrapping_sub(1)&127,y],
                [x.wrapping_sub(1)&127,y.wrapping_sub(1)&127],[x,y.wrapping_sub(1)&127]] {
                let cell=usize::from(cy)*128+usize::from(cx);
                staged.backing[MAP_START+cell*14+11]|=0x0c;
                receipt.cell_visits+=1;
                let mut raw=reference(staged.map(),cell*14+6);
                while raw[2]!=0 {
                    let record=staged.entity(raw)?;
                    let next=reference(record,3);
                    // vtable+6C: plants return true (1F280); creature,
                    // actor and effect return false (1F240).
                    match raw[2] {
                        1|3|5=>{},
                        2=>{
                            if record[0x15]&2==0 {
                                let shade=interpolate_entity_shade(staged.map(),word(record,7),word(record,9))
                                    .map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
                                let layout=staged.layouts[1];
                                let start=layout.file_offset+usize::from(u16::from_le_bytes([raw[0],raw[1]]))*layout.packed_stride;
                                staged.backing[start+0x17]=shade;
                                receipt.plant_shade_writes+=1;
                            }
                        },
                        kind=>return Err(ActorWorldError::UnsupportedRevealFamily(kind)),
                    }
                    raw=next;
                }
            }
        }
        staged.validate_links()?;
        *self=staged;
        Ok(receipt)
    }
}
