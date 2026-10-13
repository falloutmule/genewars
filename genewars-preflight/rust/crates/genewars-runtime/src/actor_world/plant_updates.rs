//! Neutral plant factory6C4B0 over the owned world. Scheduling is separate.
use super::*;
use crate::plant_spread::{plant_factory_position,plan_neutral_plant_initialization};

impl ActorWorld {
    /// Allocate the first inactive kind2 slot, initialize it from explicit
    /// source position/selector, link it at the cell head, and update count.
    /// Parent reproduction flags and the newborn pass are not run here.
    pub fn create_neutral_plant(&mut self,selector:u8,requested:[i16;2])->Result<Option<u16>,ActorWorldError> {
        if self.plants.is_none(){return Err(ActorWorldError::PlantsNotAdmitted);}
        self.validate_links()?;
        let layout=self.layouts[1];let count=word(&self.backing,0x3552);
        let active=(0..layout.count).filter(|&i|self.backing[layout.file_offset+i*layout.packed_stride+6]!=0).count();
        if count<0||count as usize!=active {
            return Err(RuntimeError::InvalidPlantCount{declared:count as u16,active}.into());
        }
        if count>=512{return Ok(None);}
        let slot=(0..layout.count).find(|&i|self.backing[layout.file_offset+i*layout.packed_stride+6]==0)
            .ok_or(RuntimeError::StorageInvariant("plant count below capacity without inactive slot"))?;
        let at=layout.file_offset+slot*layout.packed_stride;
        let mut prior=[0;49];prior[..31].copy_from_slice(&self.backing[at..at+31]);prior[39..].copy_from_slice(&self.backing[at+31..at+41]);
        let point=plant_factory_position(requested);
        let height=genewars_core::cd_candidate::interpolate_height(self.map(),point[0],point[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
        let shade=genewars_core::cd_candidate::interpolate_entity_shade(self.map(),point[0],point[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
        let mut after=plan_neutral_plant_initialization(&prior,slot as u16,selector,requested,height,shade)
            .map_err(RuntimeError::StorageInvariant)?.after;
        let cell=floor_cell(point);let head_at=MAP_START+cell*14+6;let head=reference(&self.backing,head_at);
        let raw=[slot as u8,(slot>>8)as u8,2];
        after[..2].copy_from_slice(&(cell as u16).to_le_bytes());after[2]=0;
        if head[2]!=0{after[3..6].copy_from_slice(&head);}else{after[5]=0;}
        let mut staged=self.clone();
        if head[2]!=0{staged.replace_previous(head,raw)?;}
        staged.backing[at..at+31].copy_from_slice(&after[..31]);staged.backing[at+31..at+41].copy_from_slice(&after[39..]);
        staged.backing[head_at..head_at+3].copy_from_slice(&raw);
        staged.backing[0x3552..0x3554].copy_from_slice(&(count+1).to_le_bytes());
        staged.validate_links()?;*self=staged;Ok(Some(slot as u16))
    }
}
