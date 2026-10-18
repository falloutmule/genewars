//! Transactional owned-world opcode16 placement, with two original target searches.
use super::*;
use crate::actor_construction::{ArchitectAssignmentPlan,plan_architect_assignment};
use super::building_updates::ConstructionWorldError;

impl ActorWorld {
    pub fn place_architect_construction(&mut self,player:u8,id:u16,requested:[i16;2],selector:u8,
        physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables,
        owner_selected:bool,actor_selected:bool)->Result<ArchitectAssignmentPlan,ConstructionWorldError>{
        self.player(player)?;
        let before=*self.actors.record(usize::from(id)).map_err(ActorWorldError::from)?;
        if before[6]!=3||before[24]!=player||id/5!=u16::from(player){return Err(ActorWorldError::ActorOwner(id as i16).into());}
        if before[0x70]&1==0{return Err(ActorWorldError::ActorNotCommandable(id as i16).into());}
        if !self.validate_links()?.contains(&(3,id as i16)){return Err(ActorWorldError::MissingActorLink(id as i16).into());}
        let handle=self.actors.bindings(usize::from(id)).map_err(ActorWorldError::from)?.blueprint.ok_or(ActorWorldError::MissingBlueprintBinding(id as i16))?;
        if handle.player_id!=player{return Err(ActorWorldError::ActorOwner(id as i16).into());}
        let blueprint=*self.player(player)?.blueprint(usize::from(handle.blueprint_index)).map_err(ActorWorldError::from)?;
        plan_architect_assignment(&before,&blueprint,10,u16::from(selector)<<8,owner_selected,actor_selected).map_err(ConstructionWorldError::Assignment)?;
        let resolve=|requested|{
            crate::target_resolver::resolve_target([word(&before,7),word(&before,9)],requested,physics,trig,|point|{
                let obstacle=self.move_obstacle_at(point)?;let class=self.terrain_class_at(point)?;
                if before[0x53]&(1<<class)!=0||obstacle{return Ok(false);}
                Ok(self.cell_plant_blocker(id as i16,point)?<10)
            }).map(|r|r.point).map_err(|e|match e{
                crate::target_resolver::TargetResolverError::Query(e)=>e,
                crate::target_resolver::TargetResolverError::IterationLimit=>ActorWorldError::TargetSearchLimit,
            })
        };
        let first=resolve(requested)?;let second=resolve(first)?;
        let plan=crate::actor_construction_placement::plan_architect_placement(&before,&blueprint,10,u16::from(selector)<<8,[first,second],owner_selected,actor_selected).map_err(ConstructionWorldError::Assignment)?;
        self.actors.replace_record_checked(usize::from(id),&before,&plan.after).map_err(ActorWorldError::from)?;
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queues_movement_without_inventing_speed_and_rejects_reassignment_atomically(){
        let mut bytes=vec![0;343_958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());bytes[337_054..337_058].copy_from_slice(&0x6beu32.to_le_bytes());
        for c in 0..16384 {let p=MAP_START+c*14;bytes[p..p+2].copy_from_slice(&8192u16.to_le_bytes());bytes[p+2..p+4].copy_from_slice(&3136i16.to_le_bytes());bytes[p+5]=7;}
        let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[4096,4096],b"fixture\0").unwrap());
        let mut world=ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(),&[[0;200];20],core::array::from_fn(|_|ActorHandleBindings::default()),players,BlueprintRng{seed:5428,ring:0,draw_count:0}).unwrap();
        let strings:Vec<_>=(0..128).map(|i|format!("name{i}")).collect();let borrowed:Vec<_>=strings.iter().map(|s|s.as_bytes()).collect();let names=crate::blueprint::BlueprintNames::new(&borrowed).unwrap();
        let trig=TrigTables::generate_pinned().unwrap();let physics=crate::physics_tables::PhysicsTables::fixture();
        world.regenerate_player_blueprints(0,[0;36],&names,true).unwrap();world.prepare_fresh_player(0,5,ActorOriginSample{position:[19839,12927],height:6272,shade:0},[19839,12927]).unwrap();
        world.activate_without_associated_unit(0,&trig,|_,_,_|Ok(PlacementEnvironment{terrain_class:7,blocker_weight:0})).unwrap();
        let before=*world.actors.record(0).unwrap();let mut ready=before;ready[0x55]=1;ready[0x3f..0x41].fill(0);world.actors.replace_record_checked(0,&before,&ready).unwrap();
        let plan=world.place_architect_construction(0,0,[21119,13439],3,&physics,&trig,true,true).unwrap();
        assert_eq!(word(&plan.after,0x3f),0);assert_eq!([word(&plan.after,0x5c),word(&plan.after,0x5e)],[21119,13439]);
        assert_eq!(plan.after[0x57]&3,3);assert_eq!(plan.after[0x58]&2,2);assert_eq!(plan.after[0x83],3);
        let saved=world.clone();assert!(world.place_architect_construction(0,0,[21119,13439],3,&physics,&trig,true,true).is_err());assert_eq!(world,saved);
    }
}
