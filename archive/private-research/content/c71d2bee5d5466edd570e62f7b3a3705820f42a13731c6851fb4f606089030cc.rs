#[cfg(test)]mod bootstrap_visibility_review{
 use super::*;
 #[test]fn captures(){
  let root=std::path::PathBuf::from(std::env::var("GENEWARS_REVIEW_ROOT").unwrap());let out=std::path::PathBuf::from(std::env::var("GENEWARS_REVIEW_OUT").unwrap());
  let mission=std::fs::read(root.join("private/ghidra/campaign-resources/152.dat")).unwrap();
  let text=std::fs::read(root.join("private/reference-run/run-006/drive-c/GAME/DATA/TEXT.DAT")).unwrap();let strings=genewars_assets::TextStringTable::parse(&text).unwrap();let names=BlueprintNames::new(&strings.strings()[..128]).unwrap();
  for case in 0..4{
   let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
   let mut w=ActorWorld::from_parts(&ConqMissionSource::parse(&mission).unwrap(),&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:0,draw_count:16855}).unwrap();
   w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0;67];256],[None;256]).unwrap();w.admit_effects(&[[0;81];200],[None;200]).unwrap();w.admit_family8(&[[0;102];16],[None;16]).unwrap();
   if case&1!=0{
    w.regenerate_player_blueprints(0,[0;36],&names,true).unwrap();let origin=ActorOriginSample{position:[0,0],height:genewars_core::cd_candidate::interpolate_height(w.map(),0,0).unwrap(),shade:interpolate_entity_shade(w.map(),0,0).unwrap()};w.prepare_fresh_player(0,5,origin,[20736,13568]).unwrap();w.create_landing_core(0,0).unwrap();
   }
   if case&2!=0{for cell in 0..16384{w.backing[MAP_START+cell*14+12]=63;}}
   let snap=|w:&ActorWorld|{let mut b=w.backing.clone();for r in w.actors.records(){b.extend(r);}for r in w.creatures.as_ref().unwrap().records(){b.extend(r);}for r in w.buildings.as_ref().unwrap().records(){b.extend(r);}for r in w.effects.as_ref().unwrap().records(){b.extend(r);}for r in w.family8.as_ref().unwrap().records(){b.extend(r);}for p in &w.players{b.extend(p.bytes());}b.extend(w.rng.seed.to_le_bytes());b.push(w.rng.ring);b.extend(w.rng.draw_count.to_le_bytes());b};
   std::fs::write(out.join(format!("case{case}-before.bin")),snap(&w)).unwrap();let receipt=w.initialize_mission_visibility(0).unwrap();assert_eq!(receipt.reset_cells,16384);std::fs::write(out.join(format!("case{case}-after.bin")),snap(&w)).unwrap();let mut events=Vec::new();for p in receipt.reveal.pending_presentation_vertices{for v in p{events.extend(v.to_le_bytes());}}std::fs::write(out.join(format!("case{case}-events.bin")),events).unwrap();
  }
 }
}
