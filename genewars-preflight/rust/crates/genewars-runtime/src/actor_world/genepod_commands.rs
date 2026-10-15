//! Shared human producer -> opcode12 transaction. Input capture is supplied by
//! the real control adapter; this API never spawns a creature or advances work.
use super::*;
use crate::genepod_human_producer::{self,ProducerInput,ProducerOutcome,ProducerPlan};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum GenepodInputError {
    World(ActorWorldError),
    Producer(genepod_human_producer::ProducerError),
    Command(crate::genepod_command::GenepodCommandError),
}

#[cfg(test)] mod tests {
    use super::*;
    use crate::{actor_pool::ActorHandleBindings,genepod_capture::{self,CaptureState}};
    fn fixture()->(ActorWorld,u16){
        let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime/terrain/CONQ001.DAT");
        let bytes=std::fs::read(path).unwrap();let mission=genewars_assets::ConqMissionSource::parse(&bytes).unwrap();
        let players=core::array::from_fn(|p|{
            let fresh=PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap();
            let mut bytes=*fresh.bytes();bytes[0x4a3..0x4a7].copy_from_slice(&2000i32.to_le_bytes());
            bytes[0x6e0]=1;bytes[0x6ae]=4;bytes[0x62a..0x62e].copy_from_slice(&71680i32.to_le_bytes());
            PlayerState::from_record(&bytes).unwrap()
        });
        let mut world=ActorWorld::from_mission(&mission,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:0}).unwrap();
        world.admit_creatures(&[[0;124];300]).unwrap();world.admit_buildings(&[[0;67];256],[None;256]).unwrap();
        let slot=world.create_building(0,0,[19839,12927]).unwrap().unwrap().slot;
        assert!(world.progress_admitted_grade0_building(slot,280,false).unwrap().completed);
        (world,slot)
    }
    fn input(state:u8,event:u16)->ProducerInput{ProducerInput{selected_player_index:0,panel_state:state,species_list_index:0,mouse_flags:event,captured_opaque_hit:true}}
    #[test] fn captured_release_starts_and_cancels_through_shared_world_dispatch(){
        let(mut world,slot)=fixture();let initial=world.clone();
        let down=genepod_capture::plan_capture(CaptureState::default(),1,true).unwrap();
        let receipt=world.dispatch_genepod_mule_input(slot,input(3,down.after.event),true).unwrap();
        assert_eq!(world,initial);assert_eq!(receipt.producer.panel_state_after,2);
        let up=genepod_capture::plan_capture(down.after,2,true).unwrap();
        let receipt=world.dispatch_genepod_mule_input(slot,input(receipt.producer.panel_state_after,up.after.event),true).unwrap();
        assert!(matches!(receipt.producer.outcome,ProducerOutcome::Issued{..}));assert!(receipt.panel_refresh_requested);
        let building=world.buildings().unwrap().record(slot as usize).unwrap();
        assert_eq!(i32::from_le_bytes(building[54..58].try_into().unwrap()),75);
        assert_eq!(world.player(0).unwrap().bytes(),initial.player(0).unwrap().bytes());
        assert_eq!(world.creatures(),initial.creatures());assert_eq!(world.rng(),initial.rng());
        // Cancellation uses the same captured release and refunds only paid work.
        world.dispatch_genepod_mule_input(slot,input(1,up.after.event),true).unwrap();
        assert_eq!(i32::from_le_bytes(world.buildings().unwrap().record(slot as usize).unwrap()[54..58].try_into().unwrap()),0);
        assert_eq!(world.player(0).unwrap().bytes(),initial.player(0).unwrap().bytes());
    }
    #[test] fn canceled_capture_and_invalid_selection_cannot_mutate_world(){
        let(mut world,slot)=fixture();let initial=world.clone();
        let down=genepod_capture::plan_capture(CaptureState::default(),1,true).unwrap();
        let up=genepod_capture::plan_capture(down.after,2,false).unwrap();
        let receipt=world.dispatch_genepod_mule_input(slot,input(2,up.after.event),true).unwrap();
        assert!(matches!(receipt.producer.outcome,ProducerOutcome::Ignored(_)));assert_eq!(world,initial);
        let mut invalid=input(3,8);invalid.species_list_index=1;
        assert!(world.dispatch_genepod_mule_input(slot,invalid,true).is_err());assert_eq!(world,initial);
    }
}
impl From<ActorWorldError> for GenepodInputError {
    fn from(error:ActorWorldError)->Self{Self::World(error)}
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct GenepodInputReceipt {
    pub producer:ProducerPlan,
    pub panel_refresh_requested:bool,
}
impl ActorWorld {
    /// The same producer and handler serve native and automated gameplay.
    /// Caller proves the original selected-panel refresh conjunction separately.
    pub fn dispatch_genepod_mule_input(&mut self,slot:u16,input:ProducerInput,
        selected_panel_refresh_eligible:bool)->Result<GenepodInputReceipt,GenepodInputError>{
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::EnvironmentUnavailable)?;
        let before=*pool.record(usize::from(slot)).map_err(ActorWorldError::from)?;
        let caller=*self.player(input.selected_player_index)?.bytes();
        let producer=genepod_human_producer::plan_mule_producer(&before,&caller,input)
            .map_err(GenepodInputError::Producer)?;
        let mut panel_refresh_requested=false;
        if let ProducerOutcome::Issued{request,..}=producer.outcome{
            let identity=self.player(caller[0x4a1])?.bytes()[2];
            let plan=crate::genepod_command::plan_mule_command(&before,&caller,request,
                crate::genepod_command::GenepodCommandContext{
                    funds_identity_player_type:identity,selected_panel_refresh_eligible,
                }).map_err(GenepodInputError::Command)?;
            let mut staged=self.clone();
            staged.buildings.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&plan.building_after)
                .map_err(ActorWorldError::from)?;
            staged.players[usize::from(input.selected_player_index)]
                .replace_bookkeeping_checked(&caller,&plan.caller_after).map_err(ActorWorldError::from)?;
            staged.validate_links()?;staged.validate_building_counts()?;
            panel_refresh_requested=plan.panel_refresh_requested;
            *self=staged;
        }
        Ok(GenepodInputReceipt{producer,panel_refresh_requested})
    }
}
