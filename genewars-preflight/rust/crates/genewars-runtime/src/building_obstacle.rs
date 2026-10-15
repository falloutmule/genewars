//! Source74270/74160 building query with explicit physical-height admission.
//! The nine ordered offsets include the source table's distant final entry.
//! Current-frame raw height is supplied by a separately bound resource lookup.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BuildingObstacleError<E>{MapLength,InvalidBuilding,Query(E)}
pub fn move_obstacle<E>(map:&[u8],point:[i16;2],first_building:impl FnMut(usize)->Result<Option<[u8;67]>,E>)
    ->Result<bool,BuildingObstacleError<E>> {
    move_obstacle_filtered(map,point,0,None,first_building)
}
///74D90 accepts the first74270 result when it is the nominated building.
/// Exclusion happens after query selection; it must not scan onward for a
/// different building in the same cell or a later neighboring cell.
pub fn move_obstacle_excluding<E>(map:&[u8],point:[i16;2],nominated:Option<u16>,
    first_building:impl FnMut(usize)->Result<Option<[u8;67]>,E>)
    ->Result<bool,BuildingObstacleError<E>> {
    move_obstacle_filtered(map,point,1,nominated,first_building)
}
pub fn move_obstacle_filtered<E>(map:&[u8],point:[i16;2],required_flags:u8,nominated:Option<u16>,
    first_building:impl FnMut(usize)->Result<Option<[u8;67]>,E>)
    ->Result<bool,BuildingObstacleError<E>> {
    if map.len()!=229376{return Err(BuildingObstacleError::MapLength);}
    if nominated.is_some_and(|slot|slot>=256){return Err(BuildingObstacleError::InvalidBuilding);}
    let candidate=building_at_height(map,point,-1,required_flags,first_building,
        |_| -> Result<i16,E> {unreachable!("negative-height query does not read animation metadata")})?;
    Ok(candidate.is_some_and(|record|nominated!=Some(u16::from_le_bytes(record[15..17].try_into().unwrap()))))
}

/// Source74270 with its actual physical-height argument. The caller resolves
/// each cell's first linked building and hash-bound current-frame height_raw.
/// Rejection of the selected candidate never resumes the neighboring scan.
pub fn building_at_height<E>(
    map:&[u8], point:[i16;2], physical_height:i32, required_flags:u8,
    mut first_building:impl FnMut(usize)->Result<Option<[u8;67]>,E>,
    mut height_raw:impl FnMut(&[u8;67])->Result<i16,E>,
) -> Result<Option<[u8;67]>,BuildingObstacleError<E>> {
    if map.len()!=229376{return Err(BuildingObstacleError::MapLength);}
    let cell=|p:[i16;2]|((((i32::from(p[1])>>8)&127)*128)+((i32::from(p[0])>>8)&127))as usize;
    if map[cell(point)*14+11]&1==0{return Ok(None);}
    const OFFSETS:[[i16;2];9]=[[0,0],[256,0],[256,256],[0,256],[-256,256],[-256,0],[-256,-256],[256,-256],[7680,0]];
    const CLASSES:[u8;14]=[3,1,2,3,2,2,1,2,1,2,3,3,2,1];
    for (index,offset)in OFFSETS.into_iter().enumerate(){
        let at=[point[0].wrapping_add(offset[0]),point[1].wrapping_add(offset[1])];
        let Some(record)=first_building(cell(at)).map_err(BuildingObstacleError::Query)? else{continue};
        if record[6]!=4{return Err(BuildingObstacleError::InvalidBuilding);}
        let class=*CLASSES.get(usize::from(record[45])).ok_or(BuildingObstacleError::InvalidBuilding)?;
        if class==2&&index>=4{continue;}
        let flags=u16::from_le_bytes(record[49..51].try_into().unwrap());
        if flags&u16::from(required_flags)!=u16::from(required_flags){return Ok(None);}
        let extent=match class{1=>85,2=>128,3=>256,_=>unreachable!()};
        let delta=|off,axis:usize|i32::from(i16::from_le_bytes(record[off..off+2].try_into().unwrap()).wrapping_sub(point[axis])).abs();
        if delta(7,0)>extent||delta(9,1)>extent{return Ok(None);}
        if physical_height>=0 {
            let raw=height_raw(&record).map_err(BuildingObstacleError::Query)?;
            let limit=i32::from(raw).wrapping_shl(6).wrapping_sub(320);
            let ground=i32::from_le_bytes(record[17..21].try_into().unwrap());
            if physical_height.wrapping_sub(ground)>limit{return Ok(None);}
        }
        return Ok(Some(record));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(Vec<u8>,[u8;67]) {
        let mut map=vec![0;229376];map[11]=1;
        let mut building=[0;67];building[6]=4;building[45]=0;building[49]=1;
        building[17..21].copy_from_slice(&3354i32.to_le_bytes());
        (map,building)
    }
    #[test]
    fn airborne_height_includes_boundary_and_rejection_never_searches_later() {
        let (map,building)=fixture();
        for (height,accepted) in [(13786,true),(13787,false)] {
            let mut visits=0;let mut reads=0;
            let result=building_at_height(&map,[0,0],height,0,|_|{visits+=1;Ok::<_,()>(Some(building))},|record|{
                reads+=1;assert_eq!(record,&building);Ok(168)
            }).unwrap();
            assert_eq!(result.is_some(),accepted);assert_eq!((visits,reads),(1,1));
        }
    }
    #[test]
    fn map_flags_footprint_and_negative_height_short_circuit_metadata() {
        let (mut map,mut building)=fixture();
        assert!(building_at_height(&map,[0,0],-1,0,|_|Ok::<_,()>(Some(building)),|_|panic!("negative height must skip metadata")).unwrap().is_some());
        building[49]=0;
        assert!(building_at_height(&map,[0,0],100,1,|_|Ok::<_,()>(Some(building)),|_|panic!("flags reject before metadata")).unwrap().is_none());
        building[7..9].copy_from_slice(&257i16.to_le_bytes());
        assert!(building_at_height(&map,[0,0],100,0,|_|Ok::<_,()>(Some(building)),|_|panic!("extent rejects before metadata")).unwrap().is_none());
        map[11]=0;
        assert!(building_at_height::<()>(&map,[0,0],100,0,|_|panic!("map gate rejects before scan"),|_|panic!("map gate rejects before metadata")).unwrap().is_none());
    }
    #[test]
    fn unknown_frame_error_is_preserved_without_trying_another_candidate() {
        let (map,building)=fixture();let mut visits=0;
        let result=building_at_height(&map,[0,0],100,0,|_|{visits+=1;Ok(Some(building))},|_|Err("unbound current animation frame"));
        assert_eq!(result,Err(BuildingObstacleError::Query("unbound current animation frame")));assert_eq!(visits,1);
    }
}
