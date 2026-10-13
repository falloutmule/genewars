//! Source74270/74160 movement query with mask0 and negative height.
//! The nine ordered offsets include the source table's distant final entry.
//! Positive-height sprite bounds and other query masks remain separate APIs.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BuildingObstacleError<E>{MapLength,InvalidBuilding,Query(E)}
pub fn move_obstacle<E>(map:&[u8],point:[i16;2],mut first_building:impl FnMut(usize)->Result<Option<[u8;67]>,E>)
    ->Result<bool,BuildingObstacleError<E>> {
    if map.len()!=229376{return Err(BuildingObstacleError::MapLength);}
    let cell=|p:[i16;2]|((((i32::from(p[1])>>8)&127)*128)+((i32::from(p[0])>>8)&127))as usize;
    if map[cell(point)*14+11]&1==0{return Ok(false);}
    const OFFSETS:[[i16;2];9]=[[0,0],[256,0],[256,256],[0,256],[-256,256],[-256,0],[-256,-256],[256,-256],[7680,0]];
    const CLASSES:[u8;14]=[3,1,2,3,2,2,1,2,1,2,3,3,2,1];
    for (index,offset)in OFFSETS.into_iter().enumerate(){
        let at=[point[0].wrapping_add(offset[0]),point[1].wrapping_add(offset[1])];
        let Some(record)=first_building(cell(at)).map_err(BuildingObstacleError::Query)? else{continue};
        if record[6]!=4{return Err(BuildingObstacleError::InvalidBuilding);}
        let class=*CLASSES.get(usize::from(record[45])).ok_or(BuildingObstacleError::InvalidBuilding)?;
        if class==2&&index>=4{continue;}
        let extent=match class{1=>85,2=>128,3=>256,_=>unreachable!()};
        let delta=|off,axis:usize|i32::from(i16::from_le_bytes(record[off..off+2].try_into().unwrap()).wrapping_sub(point[axis])).abs();
        // Source returns after the first admitted candidate, even when the
        // extent rejects it; searching subsequent cells would change behavior.
        return Ok(delta(7,0)<=extent&&delta(9,1)<=extent);
    }
    Ok(false)
}
