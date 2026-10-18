//! Source791C0/77B80/77960 terraform allocation and78E30 numeric work.
//! Height mutations and ordered terrain maintenance remain explicit requests;
//! a world adapter must execute them atomically before committing the job.
pub type TerrainJob=[u8;176];
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum TerrainJobError{MapLength,InvalidFootprint,InvalidStep,InvalidHeight,CountMismatch,DurationOverflow,Inactive}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TerrainJobAllocation{pub slot:u8,pub before:TerrainJob,pub after:TerrainJob,pub count_after:u8}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct TerrainHeightMutation{pub cell:u16,pub before:i16,pub after:i16}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum TerrainMaintenanceRequest{ClassifyCell(u16),BorderCell(u16),BorderRectangle{min:[i16;2],max:[i16;2]},RefreshRectangle{min:[i16;2],max:[i16;2],completed:bool}}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TerrainJobStep{pub after:TerrainJob,pub heights:Vec<TerrainHeightMutation>,pub maintenance:Vec<TerrainMaintenanceRequest>,pub retirement_required:bool}
fn word(b:&[u8],p:usize)->i16{i16::from_le_bytes(b[p..p+2].try_into().unwrap())}
fn put(b:&mut[u8],p:usize,v:i16){b[p..p+2].copy_from_slice(&v.to_le_bytes());}
fn cell(x:i16,y:i16)->usize{(((i32::from(y)>>8)&127)*128+((i32::from(x)>>8)&127))as usize}
pub fn plan_allocate_terrain_job(pool:&[TerrainJob;25],count:u8,position:[i16;2],footprint_index:u8,target:i16,step:i16,map:&[u8])->Result<Option<TerrainJobAllocation>,TerrainJobError>{
 if map.len()!=229376{return Err(TerrainJobError::MapLength);}if footprint_index>2{return Err(TerrainJobError::InvalidFootprint);}if step<=0{return Err(TerrainJobError::InvalidStep);}if target<0{return Err(TerrainJobError::InvalidHeight);}
 if count as usize!=pool.iter().filter(|r|word(*r,12)>=0).count(){return Err(TerrainJobError::CountMismatch);}if count>=25{return Ok(None);}
 let slot=pool.iter().position(|r|word(r,12)<0).ok_or(TerrainJobError::CountMismatch)?;let before=pool[slot];let mut a=before;let dim=footprint_index+2;
 let offset=if footprint_index==0{0}else{-256i16};let min=position.map(|v|(v&!255).wrapping_add(offset));let max=min.map(|v|v.wrapping_add(i16::from(dim-1)*256));
 for axis in 0..2{put(&mut a,axis*2,min[axis]);put(&mut a,4+axis*2,max[axis]);}a[8]=dim;a[9]=dim;put(&mut a,10,step);let mut duration=0i32;
 for row in 0..dim{for col in 0..dim{put(&mut a,14+usize::from(row)*18+usize::from(col)*2,target);let h=word(map,cell(min[0].wrapping_add(i16::from(col)*256),min[1].wrapping_add(i16::from(row)*256))*14+2);if h<0{return Err(TerrainJobError::InvalidHeight);}duration=duration.max((i32::from(target)-i32::from(h)).abs()/i32::from(step));}}
 duration+=1;if duration>32767{return Err(TerrainJobError::DurationOverflow);}put(&mut a,12,duration as i16);Ok(Some(TerrainJobAllocation{slot:slot as u8,before,after:a,count_after:count+1}))
}
pub fn plan_terrain_job_numeric_step(before:&TerrainJob,map:&[u8])->Result<TerrainJobStep,TerrainJobError>{
 if map.len()!=229376{return Err(TerrainJobError::MapLength);}if !(2..=4).contains(&before[8])||before[8]!=before[9]{return Err(TerrainJobError::InvalidFootprint);}if word(before,12)<=0{return Err(TerrainJobError::Inactive);}let step=word(before,10);if step<=0{return Err(TerrainJobError::InvalidStep);}
 let min=[word(before,0),word(before,2)];let max=[word(before,4),word(before,6)];let mut heights=Vec::new();
 for row in 0..before[9]{for col in 0..before[8]{let index=cell(min[0].wrapping_add(i16::from(col)*256),min[1].wrapping_add(i16::from(row)*256));let old=word(map,index*14+2);let target=word(before,14+usize::from(row)*18+usize::from(col)*2);if old<0||target<0{return Err(TerrainJobError::InvalidHeight);}if old!=target{let diff=i32::from(target)-i32::from(old);let value=if diff.abs()>i32::from(step){old+if diff>0{step}else{-step}}else{target};heights.push(TerrainHeightMutation{cell:index as u16,before:old,after:value});}}}
 let mut after=*before;let remaining=word(before,12)-1;put(&mut after,12,remaining);let mut maintenance=Vec::new();
 if remaining&1==0{
  for row in 0..before[9]{for col in 0..before[8]{maintenance.push(TerrainMaintenanceRequest::ClassifyCell(cell(min[0].wrapping_add(i16::from(col)*256),min[1].wrapping_add(i16::from(row)*256))as u16));}}
  let low=min.map(|v|v.wrapping_sub(256));let high=max.map(|v|v.wrapping_add(256));
  for row in 0..before[9]+2{for col in 0..before[8]+2{maintenance.push(TerrainMaintenanceRequest::BorderCell(cell(low[0].wrapping_add(i16::from(col)*256),low[1].wrapping_add(i16::from(row)*256))as u16));}}
  maintenance.push(TerrainMaintenanceRequest::BorderRectangle{min:low,max:high});maintenance.push(TerrainMaintenanceRequest::RefreshRectangle{min:low,max:high,completed:remaining==0});
 }
 Ok(TerrainJobStep{after,heights,maintenance,retirement_required:remaining<=0})
}
///8B890 raw-material repair. The source retains its last corner pointer
///between repair branches; it also propagates only the first two corner writes.
pub fn repair_terrain_material_boundary(map:&mut[u8],index:u16)->Result<(),TerrainJobError>{
 if map.len()!=229376{return Err(TerrainJobError::MapLength);}
 let x=i32::from(index&127);let y=i32::from((index>>7)&127);let at=|dx:i32,dy:i32|((((y+dy)&127)*128+((x+dx)&127))*14+5)as usize;
 let corners=[at(0,0),at(1,0),at(1,1),at(0,1)];let mut count=[0u8;7];for i in corners{let v=map[i];if v<7{count[v as usize]+=1;}}
 let mut pointer=corners[0];let mut propagate=false;
 if count[2]!=0&&count[2]+count[5]!=4{for i in corners{if map[i]==2{map[i]=5;count[5]+=1;}}count[2]=0;pointer=corners[3];}
 if count[3]!=0&&(count[5]!=0||(count[6]!=0&&count[4]!=0)){
  for (n,i)in [pointer,corners[1],corners[2],corners[3]].into_iter().enumerate(){if map[i]==3{map[i]=4;if n<2{propagate=true;}}}pointer=corners[3];
 }
 if count[0]!=0&&(count[5]!=0||count[3]!=0||count[2]!=0){
  for(n,i)in [pointer,corners[1],corners[2],corners[3]].into_iter().enumerate(){if map[i]!=0{map[i]=4;if n<2{propagate=true;}}}
 }
 if propagate{for(dx,dy)in[(-1,0),(-1,-1),(0,-1),(1,-1),(2,-1),(2,0)]{let i=at(dx,dy);if map[i]==2{map[i]=5;}}}Ok(())
}
///8CB00 followed by8C1C0, including the preserved-cell class gate.
///`climate_center` is the packed position global11AC02, not a cell index.
pub fn classify_terrain_cell(map:&mut[u8],index:u16,climate_center:[i16;2],water:i16,climate:i16)->Result<(),TerrainJobError>{
 if map.len()!=229376{return Err(TerrainJobError::MapLength);}
 let index=index&16383;let x=index&127;let y=index>>7;let base=usize::from(index)*14;
 let position=[(x<<8)as i16,(y<<8)as i16];let mut distance=0u32;
 for axis in 0..2{let d=climate_center[axis].wrapping_mul(2).wrapping_sub(position[axis].wrapping_mul(2))>>1;distance+=(i32::from(d)*i32::from(d))as u32;}
 let mut root=0u32;let mut bit=1u32<<30;let mut rest=distance;while bit>rest{bit>>=2;}while bit!=0{if rest>=root+bit{rest-=root+bit;root=(root>>1)+bit;}else{root>>=1;}bit>>=2;}
 let altitude=(i32::from(word(map,base+2))-i32::from(water))>>8;
 let term=-(((32768-root)>>8)as i32+altitude);put(map,base+9,term as i16);
 if map[base+11]&0x10!=0{return Ok(());}
 let material=if word(map,base+2)<=water{0}else{
  let mut low=20000i16;let mut high=-20000i16;
  for(dx,dy)in[(0u16,0u16),(1,0),(1,1),(0,1)]{let c=usize::from((((y+dy)&127)<<7)|((x+dx)&127))*14;let h=word(map,c+2);low=low.min(h);high=high.max(h);}
  let slope=high.wrapping_sub(low);let temperature=climate.wrapping_add(term as i16);
  if slope>0xa00{7}else if temperature<=25{6}else if temperature<=35{7}else if temperature<=75{3+u8::from(slope>=0x800)}else if temperature<=85{4}else if slope<0x200{2}else{5}
 };map[base+5]=material;map[base+13]=1;Ok(())
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TerrainSlopeRepair{pub heights:Vec<TerrainHeightMutation>,pub seed_after:u16,pub rng_journal_lines:Vec<u16>,pub refresh_cells:Vec<u16>}
///8C290 shade after its two733A0 normals have been supplied in call order.
pub fn terrain_shade_value(material:u8,height:i16,water:i16,normals:[[i32;3];2],light:[i32;3])->Result<i16,TerrainJobError>{
 if material&7==0{if water==0{return Err(TerrainJobError::InvalidHeight);}let numerator=(i32::from(water)-i32::from(height)).wrapping_shl(12);return Ok(8192i32.wrapping_sub(numerator/i32::from(water))as i16);}
 let mut sum=0i32;for normal in normals{let mut dot=0i32;for axis in 0..3{dot=dot.wrapping_add(normal[axis].wrapping_mul(light[axis])>>8);}sum=sum.wrapping_add(dot);}
 let average=sum/2;if average< -164{return Ok(8192);}
 let scaled=average.wrapping_add(256).wrapping_mul(3).wrapping_shl(12)/512;let value=8192i32.wrapping_sub(scaled)as i16;Ok(value.max(0xf00))
}
///8D1B0 rectangle border bytes. The supplied8192-byte table is global112D24.
///Returned cells require84E10 viewport invalidation in this exact order.
pub fn repair_terrain_rectangle(map:&mut[u8],min:[i16;2],max:[i16;2],table:&[u8;8192])->Result<Vec<u16>,TerrainJobError>{
 if map.len()!=229376{return Err(TerrainJobError::MapLength);}
 let extent=|axis:usize|i32::from(max[axis].wrapping_mul(2).wrapping_sub(min[axis].wrapping_mul(2)))/512;
 let width=extent(0);let height=extent(1);if !(0..=125).contains(&width)||!(0..=125).contains(&height){return Err(TerrainJobError::InvalidFootprint);}
 let start=[i32::from((min[0]&32767)>>8),i32::from((min[1]&32767)>>8)];let at=|x:i32,y:i32|(((y&127)*128+(x&127))*14)as usize;
 for row in 0..=height{for col in 0..=width{let x=start[0]+col;let y=start[1]+row;for(dx,dy)in[(0,0),(-1,0),(-1,-1),(0,-1)]{let p=at(x+dx,y+dy);if map[p+5]&0x80==0{map[p+4]=0x97;}}}}
 let mut invalidations=Vec::new();for row in -1..=height{for col in -1..=width{let x=start[0]+col;let y=start[1]+row;let p=at(x,y);if map[p+4]!=0x97{continue;}
  let values=[map[p+5]&7,map[at(x+1,y)+5]&7,map[at(x+1,y+1)+5]&7,map[at(x,y+1)+5]&7];let selector=usize::from(values[0])*512+usize::from(values[1])*64+usize::from(values[2])*8+usize::from(values[3]);let border=table[selector*2];map[p+4]=border;
  if values.contains(&0)&&values.iter().any(|v|*v!=0){map[p+11]|=2;}else{map[p+11]&=!2;}
  invalidations.push((p/14)as u16);map[p+5]&=0x87;if border>=8{map[p+5]=map[p+5].wrapping_add(table[selector*2+1]);}
 }}Ok(invalidations)
}
///8BEE0 diagonal height repair. Every returned cell requires8C290 followed
///by84B40 in order, before the next mutation. This is a numeric plan only.
pub fn plan_terrain_slope_repair(map:&[u8],index:u16,seed:u16)->Result<TerrainSlopeRepair,TerrainJobError>{
 if map.len()!=229376{return Err(TerrainJobError::MapLength);}
 let mut work=map.to_vec();let mut out=TerrainSlopeRepair{heights:Vec::new(),seed_after:seed,rng_journal_lines:Vec::new(),refresh_cells:Vec::new()};
 let x=i32::from(index&127);let y=i32::from((index>>7)&127);
 for(direction,line)in[(-1,0x3c1),(1,0x3d8)]{
  let mut previous=i32::from(word(&work,usize::from(index&16383)*14+2));
  for n in 1..=128{let c=((((y+direction*n)&127)*128+((x+direction*n)&127)))as u16;let p=usize::from(c)*14+2;let old=word(&work,p);let bound=previous+direction*0x340;
   if(direction<0&&i32::from(old)>=bound)||(direction>0&&i32::from(old)<=bound){break;}
   out.seed_after=out.seed_after.wrapping_mul(0xe62d);let value=(bound-direction*i32::from((out.seed_after>>8)%8))as i16;
   put(&mut work,p,value);out.heights.push(TerrainHeightMutation{cell:c,before:old,after:value});out.rng_journal_lines.push(line);out.refresh_cells.push(c);previous=i32::from(value);
   if n==128{return Err(TerrainJobError::InvalidHeight);}
  }
 }Ok(out)
}
