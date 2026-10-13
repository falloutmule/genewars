//! Source6CCF6..6CFC5 reproduction target selection, before plant allocation.
//! The caller must first establish the counter, owner and population gates.
use crate::blueprint::BlueprintRng;

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SpreadDirections([[i16;2];9]);
impl SpreadDirections {
    pub fn from_mapped_bytes(bytes:&[u8])->Result<Self,&'static str> {
        if bytes.len()!=36 || crate::identity::sha256_hex(bytes)!="8aa88c050cdb4229467f43347efcd9a8d88e5651cf2684e92666dcfe886515b6" {
            return Err("plant spread direction table identity mismatch");
        }
        Ok(Self(core::array::from_fn(|i|[i16::from_le_bytes([bytes[i*4],bytes[i*4+1]]),i16::from_le_bytes([bytes[i*4+2],bytes[i*4+3]])])))
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SpreadQuery {
    Terrain([i16;2]),
    PlantCount([u8;2]),
    CellFlags([u8;2]),
    Height([i16;2]),
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SpreadRejection { Terrain, Occupied, Obstacle, Height }

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SpreadTarget {
    pub walked_point:[i16;2],
    /// Exact argument to6C4B0. Its own placement quantization is separate.
    pub factory_point:Option<[i16;2]>,
    pub rejection:Option<SpreadRejection>,
    pub rng_after:BlueprintRng,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum SpreadError<E> { InvalidRing(u8), InvalidTerrain(i32), Environment(E) }

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PlantInitialization {
    /// Resource/vtable pointers at31..39 remain opaque native priors. The
    /// caller binds the selector and must not interpret them as host pointers.
    pub after:[u8;49],
    pub position:[i16;2],
    pub resource_selector:u8,
}

/// Neutral branch of6C4B0, excluding first-free-slot selection, link insertion
/// and global count increment. The caller supplies the exact terrain samples
/// at the returned quantized point; these are not guessed defaults.
pub fn plan_neutral_plant_initialization(prior:&[u8;49],slot:u16,selector:u8,
    requested:[i16;2],height:i32,shade:u8)->Result<PlantInitialization,&'static str> {
    if slot>=512||prior[6]!=0||selector>=12{return Err("unsupported plant initialization input");}
    let position=plant_factory_position(requested);
    let mut after=*prior;
    after[2]=0;after[5]=0;after[6]=2;after[24]=4;
    after[7..9].copy_from_slice(&position[0].to_le_bytes());after[9..11].copy_from_slice(&position[1].to_le_bytes());
    after[11..15].fill(0);after[15..17].copy_from_slice(&slot.to_le_bytes());after[17..21].copy_from_slice(&height.to_le_bytes());
    after[21]=4;after[22]=0;after[23]=shade;
    for offset in [25usize,26,27,29]{after[offset]=0;}after[28]=1;
    after[39..43].fill(0);after[43]=selector;after[44..46].copy_from_slice(&4i16.to_le_bytes());after[46]=0;after[47]=0;
    Ok(PlantInitialization{after,position,resource_selector:selector})
}

pub fn plant_factory_position(requested:[i16;2])->[i16;2] {
    requested.map(|value|((value as u16&0xff00)+((value as u16&255)>>2)+96)as i16)
}

fn draw(rng:&mut BlueprintRng)->u8 {
    rng.seed=rng.seed.wrapping_mul(0xe62d);
    rng.ring=(rng.ring+1)&31;
    rng.draw_count=rng.draw_count.wrapping_add(1);
    (rng.seed>>8)as u8
}

/// Preserve source query order and every RNG draw, including rejected targets.
/// The query callback is read-only; a failed environment lookup yields no plan.
pub fn plan_spread_target<E,F>(position:[i16;2],steps:u8,growth_deltas:[i8;8],
    height_threshold:i16,directions:&SpreadDirections,rng_before:BlueprintRng,mut query:F)
    ->Result<SpreadTarget,SpreadError<E>>
where F:FnMut(SpreadQuery)->Result<i32,E> {
    if rng_before.ring>=32{return Err(SpreadError::InvalidRing(rng_before.ring));}
    let mut rng=rng_before;let mut point=position;
    for _ in 0..steps {
        let delta=directions.0[usize::from(draw(&mut rng)%9)];
        point=[point[0].wrapping_add(delta[0]),point[1].wrapping_add(delta[1])];
    }
    let rejected=|reason,rng_after|SpreadTarget{walked_point:point,factory_point:None,rejection:Some(reason),rng_after};
    let terrain=query(SpreadQuery::Terrain(point)).map_err(SpreadError::Environment)?;
    if !(0..8).contains(&terrain){return Err(SpreadError::InvalidTerrain(terrain));}
    if growth_deltas[terrain as usize]<0{return Ok(rejected(SpreadRejection::Terrain,rng));}
    let cell=[((point[0]as u16>>8)&127)as u8,((point[1]as u16>>8)&127)as u8];
    // Source compares EAX unsigned to1: negative counts also reject.
    if query(SpreadQuery::PlantCount(cell)).map_err(SpreadError::Environment)? as u32>=1 {
        return Ok(rejected(SpreadRejection::Occupied,rng));
    }
    if query(SpreadQuery::CellFlags(cell)).map_err(SpreadError::Environment)? & 1!=0 {
        return Ok(rejected(SpreadRejection::Obstacle,rng));
    }
    // Y jitter is drawn before X. Only the low byte is replaced, retaining
    // the random walk's cell even when signed addition crosses its boundary.
    let jitter_y=i16::from(draw(&mut rng)%255)-128;
    let jitter_x=i16::from(draw(&mut rng)%255)-128;
    let candidate=[((point[0]as u16&0xff00)|(point[0].wrapping_add(jitter_x)as u16&255))as i16,
        ((point[1]as u16&0xff00)|(point[1].wrapping_add(jitter_y)as u16&255))as i16];
    let height=query(SpreadQuery::Height(candidate)).map_err(SpreadError::Environment)?;
    if height<=i32::from(height_threshold)+512{return Ok(rejected(SpreadRejection::Height,rng));}
    Ok(SpreadTarget{walked_point:point,factory_point:Some(candidate),rejection:None,rng_after:rng})
}
