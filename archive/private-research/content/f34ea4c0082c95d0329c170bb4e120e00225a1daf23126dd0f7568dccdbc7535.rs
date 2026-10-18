//! Source72E50 projection through the world's shared8ACC0 entity registry.
//! The returned carry data is for a new arena; never resume the old arena.
use super::*;
use crate::result_carry::{self,CampaignCarryPlan,CarryError};

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum OwnedCampaignCarryError { Source(CarryError), World(ActorWorldError) }
impl From<ActorWorldError> for OwnedCampaignCarryError {
    fn from(e:ActorWorldError)->Self{Self::World(e)}
}
#[cfg(test)]mod tests {
    use super::*;
    fn fixture(order:&[[u8;3]])->ActorWorld{
        let mut bytes=vec![0;343958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());
        bytes[337054..337058].copy_from_slice(&0x6beu32.to_le_bytes());
        let players=core::array::from_fn(|id|PlayerState::fresh_campaign(id as u8,[4096,4096],b"fixture\0").unwrap());
        let mut w=ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(),&[[0;200];20],[ActorHandleBindings::default();20],players,BlueprintRng{seed:1,ring:0,draw_count:9}).unwrap();
        let names:Vec<_>=(0..128).map(|i|format!("name{i}")).collect();let refs:Vec<_>=names.iter().map(|s|s.as_bytes()).collect();
        w.regenerate_player_blueprints(0,[0xa5;36],&BlueprintNames::new(&refs).unwrap(),true).unwrap();
        w.prepare_fresh_player(0,5,ActorOriginSample{position:[0,0],height:0,shade:0},[4096,4096]).unwrap();
        w.activate_without_associated_unit(0,&TrigTables::generate_pinned().unwrap(),|_,_,_|Ok(PlacementEnvironment{terrain_class:1,blocker_weight:0})).unwrap();
        // Fixture relinks the two prepared actors into one mixed entity cell.
        for cell in 0..16384{w.backing[MAP_START+cell*14+8]=0;}
        let mut records=[[0;67];256];records[0][6]=4;records[0][45]=11;let mut building_bindings=[None;256];building_bindings[0]=Some(crate::building::BuildingAnimationBinding{selector:11});
        w.buildings=Some(Box::new(BuildingPool::admit(records,building_bindings).unwrap()));
        let cell=floor_cell([4096,4096]);
        for (i,&raw) in order.iter().enumerate(){
            let mut common=[0;31];common[..3].copy_from_slice(&if i==0{[cell as u8,(cell>>8)as u8,0]}else{order[i-1]});
            common[3..6].copy_from_slice(&order.get(i+1).copied().unwrap_or([0xa5,0x5a,0]));
            common[6]=raw[2];common[7..9].copy_from_slice(&4096i16.to_le_bytes());common[9..11].copy_from_slice(&4096i16.to_le_bytes());common[15..17].copy_from_slice(&[raw[0],raw[1]]);
            match raw[2]{
                3=>{let before=*w.actors.record(raw[0]as usize).unwrap();let mut after=before;after[..31].copy_from_slice(&common);after[112]=1;w.actors.replace_record_checked(raw[0]as usize,&before,&after).unwrap();},
                4=>{let pool=w.buildings.as_mut().unwrap();let before=*pool.record(raw[0]as usize).unwrap();let mut after=before;after[..31].copy_from_slice(&common);pool.replace_record_checked(raw[0]as usize,&before,&after).unwrap();},
                _=>{let layout=w.layouts[usize::from(raw[2]-1)];let start=layout.file_offset+raw[0]as usize*layout.packed_stride;w.backing[start..start+31].copy_from_slice(&common);},
            }
        }
        w.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&order[0]);w.validate_links().unwrap();w
    }
    fn emit(w:&ActorWorld,q:&CampaignCarryPlan,label:usize){
        let Some(root)=std::env::var_os("GENEWARS_CARRY_FIXTURES")else{return};
        let d=std::path::PathBuf::from(root).join(label.to_string());std::fs::create_dir_all(&d).unwrap();
        let staged=w.stage_campaign_unlinks(&q.retired_actors).unwrap();
        let bindings:Vec<_>=(0..20).flat_map(|id|w.actors.bindings(id).unwrap().blueprint.map_or([0,0,0],|b|[1,b.player_id,b.blueprint_index])).collect();std::fs::write(d.join("bindings.bin"),bindings).unwrap();
        for (name,bytes) in [("players",w.players.iter().flat_map(|p|p.bytes().iter().copied()).collect::<Vec<_>>()),("actors",w.actors.records().concat()),("map",w.map().to_vec()),("buildings",w.buildings().unwrap().records().concat()),("backing",w.backing.clone()),("building-after",staged.buildings().unwrap().records().concat()),("backing-after",staged.backing.clone())]{std::fs::write(d.join(format!("{name}.bin")),bytes).unwrap();}
        let mut out=q.players.concat();out.extend(q.actors.concat());out.extend(&q.map);out.extend(q.records_184d7c);out.push(q.flags_after);out.extend(q.frame_after.to_le_bytes());out.push(u8::from(q.selected_actor_reset_required));out.extend(&q.retired_actors);std::fs::write(d.join("actual.bin"),out).unwrap();
    }
    #[test]fn campaign_carry_shared_cells_preserve_original_world(){
        let a=[0,0,3];let b=[1,0,3];let building=[0,0,4];let plant=[8,0,2];
        for (label,order) in [[a,b,building,plant],[building,a,plant,b],[plant,b,a,building],[building,plant,a,b],[a,building,b,plant],[b,plant,building,a]].iter().enumerate(){
            let w=fixture(order);let before=w.clone();let q=w.plan_campaign_carry(&[0xa5;600],0x7e,2,0,Some(0)).unwrap();
            assert_eq!(w,before);assert_eq!(q.retired_actors,[0,1]);assert!(q.selected_actor_reset_required);
            assert_eq!(q.actors[0][6],0);assert_eq!(q.actors[1][6],0);assert_eq!(q.players[0][3],0);
            let remaining=order.iter().find(|r|r[2]!=3).unwrap();let cell=floor_cell([4096,4096]);assert_eq!(&q.map[cell*14+6..cell*14+9],remaining);emit(&w,&q,label);
        }
    }
    #[test]fn campaign_carry_bad_links_and_late_validation_preserve_world(){
        let original=fixture(&[[0,0,3],[0,0,4],[1,0,3],[8,0,2]]);
        for variant in 0..4{
            let mut w=original.clone();
            if variant<2{let before=*w.actors.record(1).unwrap();let mut after=before;after[if variant==0{0}else{5}]=255;w.actors.replace_record_checked(1,&before,&after).unwrap();}
            else{let mut p=*w.players[0].bytes();p[5]=if variant==2{0}else{20};w.players[0]=PlayerState::from_record(&p).unwrap();}
            let before=w.clone();assert!(w.plan_campaign_carry(&[0xa5;600],0x7e,2,0,Some(0)).is_err());assert_eq!(w,before);
        }
        assert!(original.plan_campaign_carry(&[0;600],0x7e,3,0,None).is_err());
    }
}
impl From<CarryError> for OwnedCampaignCarryError {
    fn from(e:CarryError)->Self{Self::Source(e)}
}
impl ActorWorld {
    /// Pure projection: validate source list/bindings, unlink ready actors in
    /// retirement order on a disposable clone, then apply the existing numeric
    /// carry planner. All non-actor neighbours resolve through the shared store.
    pub fn plan_campaign_carry(&self,records:&[u8;600],flags:u8,mode:u8,
        selected_player:u8,selected_actor:Option<u8>)
        ->Result<CampaignCarryPlan,OwnedCampaignCarryError>{
        let players=core::array::from_fn(|id|*self.players[id].bytes());
        let bindings=core::array::from_fn(|id|self.actors.bindings(id).unwrap().blueprint
            .map(|b|(b.player_id,b.blueprint_index)));
        // This preflight only validates existing scalar/list constraints. It
        // does not commit or replace the real link traversal below.
        let mut unlinked=*self.actors.records();
        for a in &mut unlinked{a[112]&=!1;}
        let checked=result_carry::plan_campaign_carry(&players,&unlinked,&bindings,
            self.map(),records,flags,mode,selected_player,selected_actor)?;
        let staged=self.stage_campaign_unlinks(&checked.retired_actors)?;
        Ok(result_carry::plan_campaign_carry(&players,staged.actors.records(),&bindings,
            staged.map(),records,flags,mode,selected_player,selected_actor)?)
    }
    fn stage_campaign_unlinks(&self,order:&[u8])->Result<Self,ActorWorldError>{
        let mut staged=self.clone();
        for &id in order {
            let before=*staged.actors.record(usize::from(id))?;
            if before[112]&1!=0 {
                staged.unlink_actor_for_motion(u16::from(id))?;
                let before=*staged.actors.record(usize::from(id))?;
                let mut after=before;after[112]&=!1;
                staged.actors.replace_record_checked(usize::from(id),&before,&after)?;
            }
        }
        Ok(staged)
    }
}
