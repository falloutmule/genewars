//! Reproduction components shared by packed and owned-overlay plant storage.
use super::*;
use crate::blueprint::BlueprintRng;
use crate::plant_spread::{SpreadDirections,SpreadQuery,SpreadError,plan_spread_target,
    plant_factory_position,plan_neutral_plant_initialization};

pub(crate) struct PlantReproductionContext<'a> {
    pub rng:&'a mut BlueprintRng,
    pub directions:&'a SpreadDirections,
    pub height_threshold:i16,
}

impl<S:PlantStorage> MissionWorld<S> {
    pub(crate) fn create_neutral_plant_inner(&mut self,selector:u8,requested:[i16;2])->Result<Option<u16>,RuntimeError> {
        let layout=self.layouts()[PLANT_GROUP];let count=self.plant_count();
        let active=self.active_plant_indices()?.len();
        if usize::from(count)!=active{return Err(RuntimeError::InvalidPlantCount{declared:count,active});}
        if count>=512{return Ok(None);}
        let slot=(0..layout.count).find(|&i|self.bytes[layout.offset+i*layout.stride+6]==0)
            .ok_or(RuntimeError::StorageInvariant("plant count below capacity without inactive slot"))?;
        let at=layout.offset+slot*layout.stride;
        let mut prior=[0;49];prior[..31].copy_from_slice(&self.bytes[at..at+31]);prior[39..].copy_from_slice(&self.bytes[at+31..at+41]);
        let point=plant_factory_position(requested);
        let height=genewars_core::cd_candidate::interpolate_height(self.map_cells(),point[0],point[1])
            .map_err(|_|RuntimeError::StorageInvariant("plant height sample failed"))?;
        let shade=genewars_core::cd_candidate::interpolate_entity_shade(self.map_cells(),point[0],point[1])
            .map_err(|_|RuntimeError::StorageInvariant("plant shade sample failed"))?;
        let mut after=plan_neutral_plant_initialization(&prior,slot as u16,selector,requested,height,shade)
            .map_err(RuntimeError::StorageInvariant)?.after;
        let cell=usize::from((point[1]as u16>>8)&127)*128+usize::from((point[0]as u16>>8)&127);
        let head_at=MAP_OFFSET+cell*14+6;let head=[self.bytes[head_at],self.bytes[head_at+1],self.bytes[head_at+2]];
        let raw=[slot as u8,(slot>>8)as u8,2];after[..2].copy_from_slice(&(cell as u16).to_le_bytes());after[2]=0;
        if head[2]!=0 {
            let old=read_reference(&head,0).ok_or(RuntimeError::StorageInvariant("invalid plant cell head"))?;
            self.bytes.write_common_reference(old.kind,old.index,0,raw)?;after[3..6].copy_from_slice(&head);
        }else{after[5]=0;}
        self.bytes[at..at+31].copy_from_slice(&after[..31]);self.bytes[at+31..at+41].copy_from_slice(&after[39..]);
        self.bytes[head_at..head_at+3].copy_from_slice(&raw);write_u16(&mut self.bytes,PLANT_COUNT_OFFSET,count+1);
        Ok(Some(slot as u16))
    }

    pub(crate) fn reproduce_neutral_plant(&mut self,parent:usize,params:PlantParameters,
        context:&mut PlantReproductionContext<'_>)->Result<Option<u16>,RuntimeError> {
        let layout=self.layouts()[PLANT_GROUP];let at=layout.offset+parent*layout.stride;
        if self.bytes[at+24]!=4{return Err(RuntimeError::UnsupportedOwnerCounter{index:parent,owner:self.bytes[at+24]});}
        let position=[read_i16(&self.bytes,at+7),read_i16(&self.bytes,at+9)];let selector=self.bytes[at+35];
        let result=plan_spread_target(position,params.random_walk_steps_u8,params.growth_deltas_i8,
            context.height_threshold,context.directions,*context.rng,|query|match query {
                SpreadQuery::Terrain(point)=>Ok(i32::from(self.terrain_class(point))),
                SpreadQuery::PlantCount(cell)=>self.positive_growth_cell_count(cell).map(i32::from),
                SpreadQuery::CellFlags(cell)=>Ok(i32::from(self.map_cells()[(usize::from(cell[1])*128+usize::from(cell[0]))*14+11])),
                SpreadQuery::Height(point)=>genewars_core::cd_candidate::interpolate_height(self.map_cells(),point[0],point[1])
                    .map_err(|_|RuntimeError::StorageInvariant("plant spread height sample failed")),
            }).map_err(|error|match error {
                SpreadError::Environment(error)=>error,
                SpreadError::InvalidRing(_)=>RuntimeError::StorageInvariant("invalid reproduction RNG ring"),
                SpreadError::InvalidTerrain(_)=>RuntimeError::StorageInvariant("invalid reproduction terrain selector"),
            })?;
        *context.rng=result.rng_after;
        let Some(point)=result.factory_point else{return Ok(None);};
        let slot=self.create_neutral_plant_inner(selector,point)?;
        if let Some(slot)=slot{self.bytes[layout.offset+usize::from(slot)*layout.stride+21]|=1;}
        Ok(slot)
    }

    fn positive_growth_cell_count(&self,cell:[u8;2])->Result<u16,RuntimeError> {
        let index=usize::from(cell[1])*128+usize::from(cell[0]);let mut reference=read_reference(self.map_cells(),index*14+6);
        let mut count=0u16;let mut seen=HashSet::new();
        while let Some(entity)=reference {
            if !seen.insert(entity){return Err(RuntimeError::ReferenceCycle{cell:index});}
            let common=self.bytes.common_record(entity.kind,entity.index)?;
            if common[6]==2 {
                let layout=self.layouts()[PLANT_GROUP];let at=layout.offset+usize::from(entity.index)*layout.stride;
                if read_i16(&self.bytes,at+36)>0{count+=1;}
            }
            reference=read_reference(common,3);
        }
        Ok(count)
    }
}
