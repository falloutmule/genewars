
impl ActorWorld {
 pub fn review_sparse_buildings(&mut self) {
  fn target(slot:usize)->usize{slot/64*64+[1,3,7,15,23,31,47,63][slot%64]}
  fn remap(raw:&mut[u8]){if raw[2]==4{let slot=u16::from_le_bytes([raw[0],raw[1]])as usize;raw[..2].copy_from_slice(&(target(slot)as u16).to_le_bytes());}}
  let pool=self.buildings.as_ref().unwrap();let old=*pool.records();let mut records=old;let mut bindings=[None;256];
  for r in &mut records{if r[6]!=0{r[6]=0;}}
  for(slot,r)in old.iter().enumerate(){if r[6]==0{continue}let next=target(slot);let mut copy=*r;
   copy[15..17].copy_from_slice(&(next as u16).to_le_bytes());remap(&mut copy[..3]);remap(&mut copy[3..6]);
   records[next]=copy;bindings[next]=pool.binding(slot).unwrap();}
  self.buildings=Some(Box::new(crate::building_pool::BuildingPool::admit(records,bindings).unwrap()));
  for cell in self.backing[MAP_START..MAP_START+MAP_BYTES].chunks_exact_mut(14){remap(&mut cell[6..9]);}
  for(kind,layout)in self.layouts.iter().enumerate(){if matches!(kind+1,1|3|4){continue}
   for row in self.backing[layout.file_offset..layout.file_offset+layout.count*layout.packed_stride].chunks_exact_mut(layout.packed_stride){remap(&mut row[..3]);remap(&mut row[3..6]);}}
  for slot in 0..20{let before=*self.actors.record(slot).unwrap();let mut after=before;remap(&mut after[..3]);remap(&mut after[3..6]);self.actors.replace_record_checked(slot,&before,&after).unwrap();}
  for slot in 0..300{let pool=self.creatures.as_mut().unwrap();let before=*pool.record(slot).unwrap();let mut after=before;remap(&mut after[..3]);remap(&mut after[3..6]);pool.replace_record_checked(slot,&before,&after).unwrap();}
  self.validate_building_counts().unwrap();self.validate_links().unwrap();
 }
}
