//! Bounded CD code0x31 landing plans with explicit state and pending effects.
//!
//! Native handles are logical requests. These functions do not run a scheduler,
//! resolve a shared RNG history, execute audio, or force an actor ready bit.
//! Source: research/ghidra/landing-evidence.json.

pub const LANDING_RECORD_BYTES: usize = 81;
pub const FAMILY4_CAPACITY: usize = 200;
pub type LandingRecord = [u8; LANDING_RECORD_BYTES];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LandingError {
    InvalidOwner(u8), InvalidSelectedPlayer(u8), InvalidPoolCount(i16),
    NoInactiveSlot, InvalidGroundHeight(i32), InvalidRecord,
    UnsupportedPhase(u8), UnsupportedChildReference,
    InvalidHeadReference, InvalidRing(u8), UnsupportedAnimationState,
    UnsupportedPhysicsState, ContactPhysicsRequired,
    EnvironmentalCallbackRequired, MissingSynchronizedSeed,
}
impl core::fmt::Display for LandingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f,"{self:?}") }
}
impl std::error::Error for LandingError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandingReference { pub index: i16, pub kind: u8 }

/// Identity read from a resolved old head record, rather than blindly copied
/// from an unchecked raw head triple. The host validates pointer consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedCellHead { pub identity: LandingReference }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandingRng { pub seed: u16, pub ring: u8, pub draw_count: u32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandingDraw {
    pub ring_slot: u8, pub seed_before: u16, pub seed_after: u16,
    /// Source journal code0x7C8 for each noncontact phase0 draw.
    pub journal_code: u16,
}

/// Literal source audio arguments. Their platform execution remains external.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandingAudioRequest {
    pub channel: u8, pub mode: u8, pub parameter: u8,
    pub stack_byte: u8, pub stack_dword: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LandingEvent {
    SetOldHeadPrevious { old_head: LandingReference, previous: LandingReference },
    SetCellHead { cell: u16, head: LandingReference },
    SavePlayerLandingReference { owner: u8, reference: LandingReference },
    Reveal { position: [i16;2], radius: u16 },
    PlayAudio(LandingAudioRequest),
    RetireLanding { owner:u8, reference:LandingReference, player_counter_after:u32 },
    StopAudio { channel: u8 },
    /// Timer49 source UI text/resource request; journal802 choice already drawn.
    LandingMessage { owner:u8, message_index:u16 },
    AdjustAudioParameter { channel: u8, value: i16 },
    ChildContactFlags { child: LandingReference, byte3c: u8, byte33: u8 },
    /// The host must use the player's saved type5 reference and implement
    /// its associated move-order path; direct random placement differs.
    ActivateOwner { owner: u8 },
    FollowChild { child: LandingReference, position: [i16;2] },
    /// Success additionally marks newborn flags15 mask01 and contributes1
    /// to the original callback's created count; failure contributes0.
    CreateEffect {
        object_code: u8, position: [i16;2], physical_height: i32,
        angles: [u16;3], owner: u8, mark_newborn_on_success: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandingFactoryPlan {
    pub slot: u16, pub before: LandingRecord,
    /// +0x1F animation pointer is a zero placeholder. +0x23 vtable remains
    /// opaque prior bytes; neither is executed by this port.
    pub after_nonpointer: LandingRecord,
    pub animation_index: u16, pub animation_pointer_offset: u8,
    pub pool_count_after: i16, pub events: Vec<LandingEvent>,
}

fn word(b:&LandingRecord,o:usize)->i16 { i16::from_le_bytes([b[o],b[o+1]]) }
fn dword(b:&LandingRecord,o:usize)->i32 { i32::from_le_bytes(b[o..o+4].try_into().unwrap()) }
fn put_word(b:&mut LandingRecord,o:usize,v:i16){b[o..o+2].copy_from_slice(&v.to_le_bytes());}
fn put_dword(b:&mut LandingRecord,o:usize,v:i32){b[o..o+4].copy_from_slice(&v.to_le_bytes());}
fn position(b:&LandingRecord)->[i16;2]{[word(b,7),word(b,9)]}
fn ground_ok(v:i32)->Result<(),LandingError>{
    if !(-65536..=65534).contains(&v){return Err(LandingError::InvalidGroundHeight(v));} Ok(())
}
fn valid_ref(r:LandingReference)->bool{
    let caps=[300,512,20,256,200,32,32,16];
    (1..=8).contains(&r.kind)&&r.index>=0&&(r.index as usize)<caps[r.kind as usize-1]
}

/// Free-pool code0x31 factory path with all optional creation arguments zero.
/// Requires explicit preexisting81-byte records; no full-pool replacement.
/// Ground/shade are already sampled at the supplied position before reveal.
pub fn plan_landing_factory(
    pool:&[LandingRecord;FAMILY4_CAPACITY], pool_count:i16, owner:u8,
    selected_player:u8, at:[i16;2], ground:i32, shade:u8,
    old_head:Option<ResolvedCellHead>,
)->Result<LandingFactoryPlan,LandingError>{
    if !(0..200).contains(&pool_count){return Err(LandingError::InvalidPoolCount(pool_count));}
    if owner>=4{return Err(LandingError::InvalidOwner(owner));}
    if selected_player>=4{return Err(LandingError::InvalidSelectedPlayer(selected_player));}
    ground_ok(ground)?;
    if old_head.is_some_and(|h|!valid_ref(h.identity)){return Err(LandingError::InvalidHeadReference);}
    let slot=pool.iter().position(|r|r[6]==0).ok_or(LandingError::NoInactiveSlot)?;
    let before=pool[slot];let mut a=before;
    a[2]=0;a[5]=0;a[6]=5;put_word(&mut a,7,at[0]);put_word(&mut a,9,at[1]);
    put_word(&mut a,15,slot as i16);put_dword(&mut a,17,ground);
    a[11..15].fill(0);put_word(&mut a,21,0x20);a[23]=shade;a[24]=owner;
    a[25..28].fill(0);a[28]=1;a[29]=0;
    // Byte1E, fractions43/44 and all other unmentioned bytes are preserved.
    a[31..35].fill(0);put_dword(&mut a,39,ground+25600);put_word(&mut a,43,16);
    put_word(&mut a,45,0);put_word(&mut a,47,0);put_word(&mut a,49,-1088);
    a[51..57].fill(0);put_word(&mut a,57,0);put_word(&mut a,59,1024);put_word(&mut a,61,1088);
    put_word(&mut a,63,-1);a[69]=0x31;a[70]=0;put_word(&mut a,74,-1);a[73]=0;a[76]=0;
    let cell=(((i32::from(at[1])>>8)&127)*128+((i32::from(at[0])>>8)&127)) as u16;
    put_word(&mut a,0,cell as i16);a[2]=0;
    let reference=LandingReference{index:slot as i16,kind:5};let mut events=Vec::new();
    if let Some(head)=old_head{
        put_word(&mut a,3,head.identity.index);a[5]=head.identity.kind;
        events.push(LandingEvent::SetOldHeadPrevious{old_head:head.identity,previous:reference});
    }
    events.push(LandingEvent::SetCellHead{cell,head:reference});
    // Audio is requested by the factory before the caller saves the reference.
    events.push(LandingEvent::PlayAudio(LandingAudioRequest{channel:0x50,mode:2,parameter:255,stack_byte:127,stack_dword:180}));
    events.push(LandingEvent::SavePlayerLandingReference{owner,reference});
    if owner==selected_player{events.push(LandingEvent::Reveal{position:at,radius:4096});}
    Ok(LandingFactoryPlan{slot:slot as u16,before,after_nonpointer:a,animation_index:49,animation_pointer_offset:0x1f,pool_count_after:pool_count+1,events})
}

fn validate_phase0(a:&LandingRecord)->Result<Option<LandingReference>,LandingError>{
    if a[6]!=5||a[69]!=0x31{return Err(LandingError::InvalidRecord);}
    if a[70]!=0{return Err(LandingError::UnsupportedPhase(a[70]));}
    if a[24]>=4{return Err(LandingError::InvalidOwner(a[24]));}
    match a[73]{
        0=>Ok(None),
        4=>{let r=LandingReference{index:word(a,71),kind:4};if valid_ref(r){Ok(Some(r))}else{Err(LandingError::UnsupportedChildReference)}},
        _=>Err(LandingError::UnsupportedChildReference),
    }
}

/// Exact identity of7AC00 for EFF animation49/frame0/hold0 and these five
/// fresh animation bytes. Other animation states require the general updater.
pub fn validate_landing_animation_identity(a:&LandingRecord)->Result<(),LandingError>{
    if a[26..31]!=[0,0,1,0,0]{return Err(LandingError::UnsupportedAnimationState);} Ok(())
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct LandingPhase0Plan {
    pub before:LandingRecord, pub after:LandingRecord, pub contact:bool,
    pub rng_before:LandingRng, pub rng_after:LandingRng,
    pub draws:Vec<LandingDraw>, pub events:Vec<LandingEvent>,
    /// Contact contributes0. Noncontact waits for CreateEffect's success.
    pub created_count:Option<u8>,
}
fn trunc_div(a:i32,b:i32)->i32 { a/b }

/// Phase0 AFTER an explicit completed physics/animation step. No complete
/// scheduler or contact-normal/bounce behavior is inferred by this function.
pub fn plan_landing_phase0_after_physics(
    before:&LandingRecord, ground:i32, rng:LandingRng,
)->Result<LandingPhase0Plan,LandingError>{
    let child=validate_phase0(before)?;ground_ok(ground)?;
    if rng.ring>=32{return Err(LandingError::InvalidRing(rng.ring));}
    let mut after=*before;let mut events=Vec::new();let mut next=rng;let mut draws=Vec::new();
    let hit=ground>=dword(before,39);
    let created_count=if hit{
        put_dword(&mut after,39,ground);after[45..51].fill(0);after[70]=1;put_word(&mut after,74,150);
        if let Some(child)=child{events.push(LandingEvent::ChildContactFlags{child,byte3c:1,byte33:0});}
        events.push(LandingEvent::ActivateOwner{owner:after[24]});
        events.push(LandingEvent::StopAudio{channel:0x50});
        events.push(LandingEvent::PlayAudio(LandingAudioRequest{channel:0x51,mode:2,parameter:0,stack_byte:127,stack_dword:100}));
        Some(0)
    }else{
        // Source increments the count by3 before the three successive draws.
        next.draw_count=next.draw_count.wrapping_add(3);let mut high=[0u16;3];
        for value in &mut high{
            let old=next.seed;let slot=next.ring;next.seed=next.seed.wrapping_mul(0xe62d);next.ring=(next.ring+1)&31;*value=next.seed>>8;
            draws.push(LandingDraw{ring_slot:slot,seed_before:old,seed_after:next.seed,journal_code:0x7c8});
        }
        events.push(LandingEvent::CreateEffect{object_code:6,position:position(before),physical_height:dword(before,39),angles:[high[2]*8,1024-high[1]%128,2560+64*(high[0]%10)],owner:0,mark_newborn_on_success:true});
        let difference=dword(before,39).wrapping_sub(dword(before,17));
        let volume=trunc_div((difference>>6).wrapping_mul(110),400).wrapping_add(70);
        events.push(LandingEvent::PlayAudio(LandingAudioRequest{channel:0x50,mode:2,parameter:255,stack_byte:127,stack_dword:volume}));
        events.push(LandingEvent::AdjustAudioParameter{channel:0x50,value:volume as i16});None
    };
    if let Some(child)=child{events.push(LandingEvent::FollowChild{child,position:position(before)});}
    Ok(LandingPhase0Plan{before:*before,after,contact:hit,rng_before:rng,rng_after:next,draws,events,created_count})
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct LandingAirbornePlan {pub before:LandingRecord,pub after:LandingRecord}

/// Complete restricted airborne vertical slice of6DF30, excluding the step
/// that contacts terrain. The environment must prove no collision object and
/// supply the actual water height and ground/shade samples at the same point.
pub fn plan_landing_vertical_airborne(
    before:&LandingRecord, ground:i32, shade:u8, water_height:i32,
    obstacle_lookup_is_null:bool,
)->Result<LandingAirbornePlan,LandingError>{
    validate_phase0(before)?;validate_landing_animation_identity(before)?;ground_ok(ground)?;
    let at=position(before);
    if before[21]&0x20==0 || before[22]&0x0a!=0 || before[67]!=0 || before[68]!=0
        || at.iter().any(|&p|p<0) || word(before,45)!=0 || word(before,47)!=0 || word(before,49)!=-1088
        || before[51..57]!=[0;6] || [word(before,57),word(before,59),word(before,61)]!=[0,1024,1088]
        || word(before,63)!=-1 {return Err(LandingError::UnsupportedPhysicsState);}
    if !obstacle_lookup_is_null || !(0..=65534).contains(&water_height) || ground<=water_height+320{
        return Err(LandingError::EnvironmentalCallbackRequired);
    }
    let next_height=dword(before,39).wrapping_sub(136);
    if dword(before,39)<=ground || next_height<=ground{return Err(LandingError::ContactPhysicsRequired);}
    let mut after=*before;put_dword(&mut after,17,ground);after[23]=shade;put_dword(&mut after,39,next_height);
    Ok(LandingAirbornePlan{before:*before,after})
}

/// Source733A0 triangle normal from four signed map height words, in
/// [northwest,northeast,southwest,southeast] order. Inputs are native map
/// words before the source's arithmetic shift by3.
pub fn landing_terrain_normal(at:[i16;2], heights:[i16;4])->[i32;3]{
    let h=heights.map(|x|i32::from(x)>>3);
    let (a,b)=if (at[0] as u16&255)<(at[1] as u16&255){
        ([256,256,h[3]-h[0]],[-256,0,h[2]-h[3]])
    }else{([256,0,h[1]-h[0]],[0,256,h[3]-h[1]])};
    fn norm(v:[i32;3])->[i32;3]{let n=landing_sqrt(v.iter().map(|x|x*x).sum::<i32>() as u32).max(1) as i32;v.map(|x|x*256/n)}
    let a=norm(a);let b=norm(b);
    norm([(a[1]*b[2]-b[1]*a[2])>>8,(a[2]*b[0]-b[2]*a[0])>>8,(a[0]*b[1]-b[0]*a[1])>>8])
}
fn landing_horizontal_squared(x:i16,y:i16)->u32{(i64::from(x)*i64::from(x)+i64::from(y)*i64::from(y)) as u32}
fn landing_sqrt(n:u32)->u32{
    if n==0{return 0;}let mut x=1u64<<((32-n.leading_zeros()+1)/2);
    loop{let y=(x+u64::from(n)/x)/2;if y>=x{return x as u32;}x=y;}
}
/// AE028's octant lookup. Caller must bind the257-word table atAE134.
fn landing_angle(x:i32,y:i32,t:&[u16;257])->u16{
    if x==0&&y==0{return 0;}
    let ax=x.unsigned_abs();let ay=y.unsigned_abs();
    let low=if ax<ay{t[((u64::from(ax)*256)/u64::from(ay)) as usize]}else{t[((u64::from(ay)*256)/u64::from(ax)) as usize]};
    match(x>=0,y>=0,ax<ay){
        (true,false,true)=>low,(true,false,false)=>512-low,
        (true,true,true)=>1024-low,(true,true,false)=>512+low,
        (false,false,true)=>2048-low,(false,false,false)=>1536+low,
        (false,true,true)=>1024+low,(false,true,false)=>1536-low,
    }
}
/// Exact first terrain-contact slice for fresh vertical code31. The caller
/// supplies the four actual map corners, source bounce coefficient indexed
/// by actual8B520 result, and the pinnedAE134 table. This computes the
/// postphysics angles, rather than supplying an invented afterimage.
/// Environmental gates are the same as the airborne slice; phase0 follows.
pub fn plan_landing_vertical_contact(
    before:&LandingRecord,ground:i32,shade:u8,water_height:i32,
    obstacle_lookup_is_null:bool, terrain_height_words:[i16;4],
    surface_bounce_coefficient:i16, angle_table:&[u16;257],
)->Result<LandingAirbornePlan,LandingError>{
    if angle_table.iter().any(|&x|x>256){return Err(LandingError::UnsupportedPhysicsState);}
    // Reuse every fresh-route gate without bypassing the contact boundary.
    let mut probe=*before;put_dword(&mut probe,39,ground+137);
    plan_landing_vertical_airborne(&probe,ground,shade,water_height,obstacle_lookup_is_null)?;
    if dword(before,39)<=dword(before,17)||dword(before,39).wrapping_sub(136)>ground{
        return Err(LandingError::UnsupportedPhysicsState);
    }
    let mut after=*before;put_dword(&mut after,17,ground);after[23]=shade;put_dword(&mut after,39,ground);
    let normal=landing_terrain_normal(position(before),terrain_height_words);
    let vz=((1088i32*i32::from(surface_bounce_coefficient))>>8) as i16;
    let vx=(i32::from(vz)*normal[0]/256) as i16;
    let vy=(i32::from(vz)*normal[1]/256) as i16;
    put_word(&mut after,45,vx);put_word(&mut after,47,vy);put_word(&mut after,49,vz);
    let xy=landing_sqrt(landing_horizontal_squared(vx,vy));
    if xy!=0{put_word(&mut after,57,landing_angle(i32::from(vx),i32::from(vy),angle_table) as i16);}
    let speed=landing_sqrt(xy*xy+(i32::from(vz)*i32::from(vz)) as u32) as i16;
    if speed<0{return Err(LandingError::UnsupportedPhysicsState);}
    put_word(&mut after,61,speed);
    put_word(&mut after,59,if speed==0{512}else{landing_angle(xy as i32,-i32::from(vz),angle_table) as i16});
    Ok(LandingAirbornePlan{before:*before,after})
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct LandingEffect6FactoryPlan{
    pub factory:LandingFactoryPlan,pub rng_before:LandingRng,
    pub rng_after:LandingRng,pub draw:LandingDraw,
}
/// Code6 factory requested by airborne code31, with null parent and all
/// optional arguments zero. Source sine/cosine tables remain explicit input.
/// Full pool replacement is excluded and does not silently consume a draw.
pub fn plan_landing_effect6_factory(
    pool:&[LandingRecord;FAMILY4_CAPACITY],pool_count:i16,
    at:[i16;2],ground:i32,shade:u8,physical_height:i32,angles:[u16;3],
    old_head:Option<ResolvedCellHead>,gravity:i16,rng:LandingRng,
    sine:&[i32;2048],cosine:&[i32;2048],
)->Result<LandingEffect6FactoryPlan,LandingError>{
    if rng.ring>=32{return Err(LandingError::InvalidRing(rng.ring));}
    // Source1F070/48790 sign-extend both angles before indexing the two
    // 2048-entry trig tables. Extractor smoke can launch with negative pitch.
    if angles[..2].iter().any(|&v|!(-1536..=2559).contains(&(v as i16)))||angles[2]>32767{return Err(LandingError::UnsupportedPhysicsState);}
    let mut factory=plan_landing_factory(pool,pool_count,0,0,at,ground,shade,old_head)?;
    let prior=factory.before;let a=&mut factory.after_nonpointer;
    put_dword(a,39,physical_height);a[69]=6;a[73]=prior[73];a[76]=prior[76];
    for(i,v)in angles.into_iter().enumerate(){put_word(a,57+i*2,v as i16);}
    fn shift(v:u16)->usize{let signed=i32::from(v as i16);if signed<512{(signed+1536)as usize}else{(signed-512)as usize}}
    let horizontal=((i64::from(cosine[shift(angles[1])])*i64::from(angles[2]))>>16) as i32;
    put_word(a,45,((i64::from(cosine[shift(angles[0])])*i64::from(horizontal))>>16) as i16);
    put_word(a,47,((i64::from(sine[shift(angles[0])])*i64::from(horizontal))>>16) as i16);
    put_word(a,49,((-i64::from(sine[shift(angles[1])])*i64::from(angles[2]))>>16) as i16);
    put_word(a,55,-(gravity>>6));
    let next=LandingRng{seed:rng.seed.wrapping_mul(0xe62d),ring:(rng.ring+1)&31,draw_count:rng.draw_count.wrapping_add(1)};
    a[29]=1+((next.seed>>8)%3) as u8;
    factory.animation_index=6;
    factory.events.retain(|e|matches!(e,LandingEvent::SetCellHead{..}|LandingEvent::SetOldHeadPrevious{..}));
    Ok(LandingEffect6FactoryPlan{factory,rng_before:rng,rng_after:next,draw:LandingDraw{ring_slot:rng.ring,seed_before:rng.seed,seed_after:next.seed,journal_code:0x21d}})
}
/// Code6's own selector tail after the shared physics/animation/retirement
/// prefix. No RNG: each signed velocity becomes arithmetic(15*v)>>4.
pub fn plan_effect6_damping_after_prefix(before:&LandingRecord)->Result<LandingRecord,LandingError>{
    if before[6]!=5||before[69]!=6{return Err(LandingError::InvalidRecord);}
    let mut after=*before;for o in [45,47,49]{put_word(&mut after,o,((i32::from(word(before,o))*15)>>4) as i16);}Ok(after)
}

/// The effect physics ground/shade/normal query must use this fixed-point
/// candidate, after XY accelerations. Fraction bytes43/44 are native input.
pub fn effect6_physics_candidate(before:&LandingRecord)->Result<[i16;2],LandingError>{
    if before[6]!=5||before[69]!=6{return Err(LandingError::InvalidRecord);}
    let mut at=[0;2];for(i,o)in[7,9].into_iter().enumerate(){
        let velocity=word(before,45+2*i).wrapping_add(word(before,51+2*i));
        let fixed=(i32::from(word(before,o))<<16).wrapping_add(i32::from(before[67+i])<<8).wrapping_add(i32::from(velocity)<<10);
        at[i]=(fixed>>16) as i16;
    }Ok(at)
}
/// Airborne or first-impact6DF30 for code6, including fractional XY and
/// source orientation. Repeated grounded motion, water callbacks and occupied
/// collision queries reject. A changed map cell requires host unlink/relink
/// and virtual88 handling before committing this record.
pub fn plan_effect6_flight(
    before:&LandingRecord,ground:i32,shade:u8,water_height:i32,
    obstacle_lookup_is_null:bool,terrain_height_words:[i16;4],
    bounce_coefficient:i16,angle_table:&[u16;257],
)->Result<LandingAirbornePlan,LandingError>{
    ground_ok(ground)?;if angle_table.iter().any(|&x|x>256){return Err(LandingError::UnsupportedPhysicsState);}let at=effect6_physics_candidate(before)?;
    if before[21]&0x20==0||before[21]&!0x21!=0||before[22]!=0||word(before,63)!=-1
        ||before[51..55]!=[0;4]{return Err(LandingError::UnsupportedPhysicsState);}
    let mut after=*before;
    let mut velocity=[word(before,45),word(before,47),word(before,49).wrapping_add(word(before,55))];
    let candidate_height=dword(before,39).wrapping_add(i32::from(velocity[2])>>3);
    let height=ground.max(candidate_height.max(0));
    if !obstacle_lookup_is_null||!(0..=65534).contains(&water_height){return Err(LandingError::EnvironmentalCallbackRequired);}
    for(i,o)in[7,9].into_iter().enumerate(){
        let fixed=(i32::from(word(before,o))<<16).wrapping_add(i32::from(before[67+i])<<8).wrapping_add(i32::from(velocity[i])<<10);
        put_word(&mut after,o,at[i]);after[67+i]=(fixed>>8) as u8;
    }
    put_dword(&mut after,17,ground);after[23]=shade;put_dword(&mut after,39,height);
    if candidate_height<=ground{
        if dword(before,39)<=dword(before,17){return Err(LandingError::UnsupportedPhysicsState);}
        // Source first-impact branch's damage request only occurs for a
        // positive downward-expression. Smallpositiveuphillsurfaceimpacts
        // stillhaveexpression0andmustbounce, ratherthanrejectallpositiveVZ.
        if (i32::from(velocity[2])>>6)/16>0{return Err(LandingError::UnsupportedPhysicsState);}
        let normal=landing_terrain_normal(at,terrain_height_words);
        velocity[2]=((i32::from(velocity[2]).abs()*i32::from(bounce_coefficient))>>8) as i16;
        velocity[0]=velocity[0].wrapping_add((i32::from(velocity[2])*normal[0]/256) as i16);
        velocity[1]=velocity[1].wrapping_add((i32::from(velocity[2])*normal[1]/256) as i16);
    }
    for(i,v)in velocity.into_iter().enumerate(){put_word(&mut after,45+2*i,v);}
    let xy=landing_sqrt(landing_horizontal_squared(velocity[0],velocity[1]));
    if xy!=0{put_word(&mut after,57,landing_angle(i32::from(velocity[0]),i32::from(velocity[1]),angle_table) as i16);}
    let speed=landing_sqrt(xy*xy+(i32::from(velocity[2])*i32::from(velocity[2])) as u32) as i16;
    if speed<0||speed>0x4b00{return Err(LandingError::UnsupportedPhysicsState);}
    put_word(&mut after,61,speed);put_word(&mut after,59,if speed==0{512}else{landing_angle(xy as i32,-i32::from(velocity[2]),angle_table) as i16});
    for o in [7,9]{let v=word(&after,o)&0x7fff;put_word(&mut after,o,v);}
    Ok(LandingAirbornePlan{before:*before,after})
}
/// Grounded code6 branch of6DF30 with flags16=0, flags15 limited to21,
/// 3F=-1. First sample is at effect6_physics_candidate. The source then
/// quantizes velocity through AE028/trig and samples at this returned point
/// AGAIN. Host must query that actual point before finalizing the plan.
pub fn effect6_grounded_candidate(
    before:&LandingRecord,first_ground:i32,angle_table:&[u16;257],
    sine:&[i32;2048],cosine:&[i32;2048],
)->Result<LandingRecord,LandingError>{
    ground_ok(first_ground)?;if angle_table.iter().any(|&x|x>256){return Err(LandingError::UnsupportedPhysicsState);}effect6_physics_candidate(before)?;
    if before[21]&!0x21!=0||before[21]&0x20==0||before[22]!=0||word(before,63)!=-1
        ||before[51..55]!=[0;4]||dword(before,39)>dword(before,17){return Err(LandingError::UnsupportedPhysicsState);}
    let vz=word(before,49).wrapping_add(word(before,55));
    if dword(before,39).wrapping_add(i32::from(vz)>>3)>first_ground{return Err(LandingError::UnsupportedPhysicsState);}
    let mut after=*before;let vx=word(before,45);let vy=word(before,47);
    let vz=(first_ground as i16).wrapping_sub(dword(before,17) as i16).wrapping_mul(8);
    let xy=landing_sqrt(landing_horizontal_squared(vx,vy));
    if xy!=0{put_word(&mut after,57,landing_angle(i32::from(vx),i32::from(vy),angle_table) as i16);}
    let speed=landing_sqrt(xy*xy+(i32::from(vz)*i32::from(vz)) as u32) as i16;
    if speed<0{return Err(LandingError::UnsupportedPhysicsState);}
    put_word(&mut after,61,speed);put_word(&mut after,59,if speed==0{512}else{landing_angle(xy as i32,-i32::from(vz),angle_table) as i16});
    fn shift(v:i16)->usize{if v<512{v as usize+1536}else{v as usize-512}}
    let yaw=word(&after,57);let pitch=word(&after,59);
    if !(0..=2048).contains(&yaw)||!(0..=2048).contains(&pitch)||speed<0{return Err(LandingError::UnsupportedPhysicsState);}
    let horizontal=((i64::from(cosine[shift(pitch)])*i64::from(speed))>>16) as i32;
    let velocity=[((i64::from(cosine[shift(yaw)])*i64::from(horizontal))>>16) as i16,((i64::from(sine[shift(yaw)])*i64::from(horizontal))>>16) as i16,((-i64::from(sine[shift(pitch)])*i64::from(speed))>>16) as i16];
    for(i,v)in velocity.into_iter().enumerate(){put_word(&mut after,45+i*2,v);}
    for(i,o)in[7,9].into_iter().enumerate(){let fixed=(i32::from(word(before,o))<<16).wrapping_add(i32::from(before[67+i])<<8).wrapping_add(i32::from(velocity[i])<<10);put_word(&mut after,o,(fixed>>16) as i16);after[67+i]=(fixed>>8) as u8;}
    Ok(after)
}
/// Complete the second7B380 sample and final no-obstacle/no-water gates for
/// effect6_grounded_candidate. All cell link and retirement effects remain
/// host transactions, exactly as for plan_effect6_flight.
pub fn plan_effect6_grounded_after_second_sample(
    before:&LandingRecord,candidate:&LandingRecord,ground:i32,shade:u8,
    water_height:i32,obstacle_lookup_is_null:bool,
)->Result<LandingAirbornePlan,LandingError>{
    ground_ok(ground)?;
    if candidate[6]!=5||candidate[69]!=6||word(candidate,15)!=word(before,15){return Err(LandingError::InvalidRecord);}
    if !obstacle_lookup_is_null||!(0..=65534).contains(&water_height){return Err(LandingError::EnvironmentalCallbackRequired);}
    let mut after=*candidate;put_dword(&mut after,17,ground);after[23]=shade;put_dword(&mut after,39,ground);
    for o in [7,9]{let v=word(&after,o)&0x7fff;put_word(&mut after,o,v);}
    Ok(LandingAirbornePlan{before:*before,after})
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Effect6UpdatePlan{pub after:LandingRecord,pub retirement_required:bool}
/// EFF6 has13frames, flags0, everyframehold0. This source7AC00 prefix
/// advances delay1..3, tests the last frame, and damps only surviving records.
/// Retirement needs49D30 mapunlink/724C0 reference cleanup/count decrement;
/// no code6 child effect or RNG is requested (row byte3=FF).
pub fn plan_effect6_animation_after_physics(before:&LandingRecord)->Result<Effect6UpdatePlan,LandingError>{
    if before[6]!=5||before[69]!=6{return Err(LandingError::InvalidRecord);}
    if before[26]!=0||before[27]>12||before[28]!=1||!(1..=3).contains(&before[29])||before[30]>3{
        return Err(LandingError::UnsupportedAnimationState);
    }
    let mut after=*before;let delay=after[29];after[30]=after[30].min(delay);
    let old=after[30];after[30]=old.wrapping_sub(1);
    if old==0{after[30]=delay;after[27]=(after[27]+1)%13;}
    let timer=word(&after,74);if timer>=0{put_word(&mut after,74,timer.wrapping_sub(1));}
    let retired=word(&after,74)<0&&after[27]==12&&after[30]==0;
    if !retired{after=plan_effect6_damping_after_prefix(&after)?;}
    Ok(Effect6UpdatePlan{after,retirement_required:retired})
}

/// Quiet grounded phase1 interval after contact, before timer49's UI/RNG
/// and code2F factory branch. Fresh mode4C0, no kind4 child, zero velocities/
/// accelerations and fractions are required. Source physical virtualheight
/// becomes0, so collision callback is skipped; ground must exclude water.
/// This includes the grounded physics angle update and animation49 identity.
pub fn plan_landing_phase1_quiet_grounded(
 before:&LandingRecord,ground:i32,shade:u8,water_height:i32,
)->Result<LandingAirbornePlan,LandingError>{
 ground_ok(ground)?;
 if before[6]!=5||before[69]!=0x31||before[70]!=1||before[24]>=4
  ||before[73]!=0||before[76]!=0||before[21]&!0x21!=0||before[21]&0x20==0||before[22]!=0
  ||before[45..57]!=[0;12]||before[67..69]!=[0;2]||word(before,63)!=-1
  ||dword(before,17)!=ground||dword(before,39)!=ground||word(before,74)<51{return Err(LandingError::UnsupportedPhysicsState);}
 validate_landing_animation_identity(before)?;
 if !(0..=65534).contains(&water_height)||ground<=water_height+320{return Err(LandingError::EnvironmentalCallbackRequired);}
 let mut after=*before;after[23]=shade;put_word(&mut after,59,512);put_word(&mut after,61,0);put_word(&mut after,74,word(before,74)-1);
 Ok(LandingAirbornePlan{before:*before,after})
}

/// Stationary grounded or bounded free-flight phase1 physics before its timer.
/// This uses the same gates as the quiet interval but leaves timer untouched.
pub fn plan_landing_phase1_rest_physics(before:&LandingRecord,ground:i32,shade:u8,water_height:i32)->Result<LandingAirbornePlan,LandingError>{
 if dword(before,39)==ground {
  let mut probe=*before;put_word(&mut probe,74,51);
  let mut plan=plan_landing_phase1_quiet_grounded(&probe,ground,shade,water_height)?;
  plan.before=*before;put_word(&mut plan.after,74,word(before,74));return Ok(plan);
 }
 //6DFCF integrates zero acceleration under free-flight20.6E538 retains
 //positive candidate height above ground;3F=-1 skips obstacle-height work.
 ground_ok(ground)?;
 if before[6]!=5||before[69]!=0x31||before[70]!=1||before[24]>=4
  ||before[73]!=0||before[76]!=0||before[21]&!0x21!=0||before[21]&0x20==0||before[22]!=0
  ||before[45..57]!=[0;12]||before[67..69]!=[0;2]||word(before,63)!=-1
  ||dword(before,17)!=ground||dword(before,39)<=ground||!(1..=65534).contains(&dword(before,39))
  ||word(before,7)<0||word(before,9)<0{return Err(LandingError::UnsupportedPhysicsState);}
 validate_landing_animation_identity(before)?;
 if !(0..=65534).contains(&water_height)||ground<=water_height+320{return Err(LandingError::EnvironmentalCallbackRequired);}
 let mut after=*before;after[23]=shade;put_word(&mut after,59,512);put_word(&mut after,61,0);
 Ok(LandingAirbornePlan{before:*before,after})
}
/// Phase1 selector after actual physics+EFF49 animation identity. Timer49
/// chooses one UI message and all remaining positive timers create code6.
/// Success/failure factory accounting remains an ordered host transaction.
pub fn plan_landing_phase1_after_physics(before:&LandingRecord,rng:LandingRng)->Result<LandingPhase0Plan,LandingError>{
 if before[6]!=5||before[69]!=0x31||before[70]!=1||before[24]>=4||before[73]!=0||before[76]!=0||word(before,74)<1{return Err(LandingError::UnsupportedPhysicsState);}
 if rng.ring>=32{return Err(LandingError::InvalidRing(rng.ring));}
 validate_landing_animation_identity(before)?;
 let mut after=*before;let timer=word(before,74)-1;put_word(&mut after,74,timer);
 let mut next=rng;let mut draws=Vec::new();let mut events=Vec::new();
 fn draw(r:&mut LandingRng,draws:&mut Vec<LandingDraw>,journal_code:u16)->u16{let old=r.seed;let slot=r.ring;r.seed=r.seed.wrapping_mul(0xe62d);r.ring=(r.ring+1)&31;r.draw_count=r.draw_count.wrapping_add(1);draws.push(LandingDraw{ring_slot:slot,seed_before:old,seed_after:r.seed,journal_code});r.seed>>8}
 let created_count=if timer==0{
  put_word(&mut after,74,30);put_word(&mut after,57,0);put_word(&mut after,59,0);put_word(&mut after,61,512);
  put_word(&mut after,45,0);put_word(&mut after,47,0);put_word(&mut after,49,512);after[70]=2;Some(0)
 }else if timer<50{
  if timer==49{let choice=draw(&mut next,&mut draws,0x802)%8;events.push(LandingEvent::LandingMessage{owner:before[24],message_index:79+choice});}
  let speed=(40+draw(&mut next,&mut draws,0x806)%10)*64;
  let pitch=1024-draw(&mut next,&mut draws,0x806)%128;
  let yaw=draw(&mut next,&mut draws,0x806)*8;
  events.push(LandingEvent::CreateEffect{object_code:6,position:position(before),physical_height:dword(before,39),angles:[yaw,pitch,speed],owner:0,mark_newborn_on_success:true});
  let value=trunc_div((dword(before,39).wrapping_sub(dword(before,17))>>6).wrapping_mul(110),200).wrapping_add(70);
  events.push(LandingEvent::PlayAudio(LandingAudioRequest{channel:0x50,mode:2,parameter:255,stack_byte:127,stack_dword:value}));events.push(LandingEvent::AdjustAudioParameter{channel:0x50,value:value as i16});None
 }else{Some(0)};
 Ok(LandingPhase0Plan{before:*before,after,contact:false,rng_before:rng,rng_after:next,draws,events,created_count})
}

/// Phase2/3 rising vertical code31 physics, actualno-obstacle/no-watergates.
pub fn plan_landing_rising_physics(before:&LandingRecord,ground:i32,shade:u8,water:i32,collision_null:bool,angles:&[u16;257])->Result<LandingAirbornePlan,LandingError>{
 if before[6]!=5||before[69]!=0x31||!matches!(before[70],2|3)||before[73]!=0||before[76]!=0||before[45..49]!=[0;4]||before[51..55]!=[0;4]||before[67..69]!=[0;2]||word(before,49)<=0||word(before,55)<0{return Err(LandingError::UnsupportedPhysicsState);}
 validate_landing_animation_identity(before)?;let mut temporary=*before;temporary[69]=6;
 let mut plan=plan_effect6_flight(&temporary,ground,shade,water,collision_null,[0;4],0,angles)?;
 if dword(&plan.after,39)<=water+320{return Err(LandingError::EnvironmentalCallbackRequired);}plan.before=*before;plan.after[69]=0x31;Ok(plan)
}
pub fn plan_landing_phase2_after_physics(before:&LandingRecord,rng:LandingRng)->Result<LandingPhase0Plan,LandingError>{
 if rng.ring>=32{return Err(LandingError::InvalidRing(rng.ring));}
 if before[6]!=5||before[69]!=0x31||before[70]!=2||before[24]>=4||before[73]!=0||before[76]!=0||!(1..=30).contains(&word(before,74)){return Err(LandingError::UnsupportedPhysicsState);}
 validate_landing_animation_identity(before)?;
 if word(before,74)==1{let mut after=*before;put_word(&mut after,74,20);after[51..55].fill(0);put_word(&mut after,55,640);after[70]=3;return Ok(LandingPhase0Plan{before:*before,after,contact:false,rng_before:rng,rng_after:rng,draws:Vec::new(),events:vec![LandingEvent::StopAudio{channel:0x50},LandingEvent::PlayAudio(LandingAudioRequest{channel:0x52,mode:2,parameter:0,stack_byte:127,stack_dword:100})],created_count:Some(0)});}
 let mut temporary=*before;temporary[70]=1;let mut plan=plan_landing_phase1_after_physics(&temporary,rng)?;
 plan.before=*before;plan.after[70]=2;for event in &mut plan.events{if let LandingEvent::CreateEffect{angles,..}=event{angles[2]+=1280;}}Ok(plan)
}
pub fn plan_landing_phase3_after_physics(before:&LandingRecord,rng:LandingRng)->Result<LandingPhase0Plan,LandingError>{
 if before[6]!=5||before[69]!=0x31||before[70]!=3||before[24]>=4||before[73]!=0||before[76]!=0||!(1..=20).contains(&word(before,74)){return Err(LandingError::UnsupportedPhysicsState);}
 if rng.ring>=32{return Err(LandingError::InvalidRing(rng.ring));}validate_landing_animation_identity(before)?;
 let mut after=*before;let timer=word(before,74)-1;put_word(&mut after,74,timer);let mut next=rng;let mut draws=Vec::new();let mut events=Vec::new();
 if timer==0{events.push(LandingEvent::RetireLanding{owner:before[24],reference:LandingReference{index:word(before,15),kind:5},player_counter_after:700});return Ok(LandingPhase0Plan{before:*before,after,contact:false,rng_before:rng,rng_after:next,draws,events,created_count:Some(0)});}
 let mut angles=[word(before,57)as u16,word(before,59)as u16,(word(before,61)>>2)as u16];
 next.draw_count=next.draw_count.wrapping_add(2);
 for (i,journal_code)in[0x822,0x823].into_iter().enumerate(){let old=next.seed;let slot=next.ring;next.seed=next.seed.wrapping_mul(0xe62d);next.ring=(next.ring+1)&31;angles[i]=angles[i].wrapping_add(1920+(next.seed>>8))&2047;draws.push(LandingDraw{ring_slot:slot,seed_before:old,seed_after:next.seed,journal_code});}
 events.push(LandingEvent::CreateEffect{object_code:0x2f,position:position(before),physical_height:dword(before,39),angles,owner:before[24],mark_newborn_on_success:true});
 Ok(LandingPhase0Plan{before:*before,after,contact:false,rng_before:rng,rng_after:next,draws,events,created_count:None})
}
/// Code2Fnull-parent factory sharescode6commoninitialization, acceleration,
/// delay1..3 butjournal229 andownerparent. ItbindsEFFanimation47.
pub fn plan_landing_effect2f_factory(pool:&[LandingRecord;200],count:i16,owner:u8,at:[i16;2],ground:i32,shade:u8,height:i32,angles:[u16;3],head:Option<ResolvedCellHead>,gravity:i16,rng:LandingRng,sine:&[i32;2048],cosine:&[i32;2048])->Result<LandingEffect6FactoryPlan,LandingError>{
 if owner>=4{return Err(LandingError::InvalidOwner(owner));}let mut plan=plan_landing_effect6_factory(pool,count,at,ground,shade,height,angles,head,gravity,rng,sine,cosine)?;plan.factory.after_nonpointer[24]=owner;plan.factory.after_nonpointer[69]=0x2f;plan.factory.animation_index=47;plan.draw.journal_code=0x229;Ok(plan)
}
/// EFF47sixhold0framesflags0; sourcecode2F hasnoownselectortaildamping.
pub fn plan_effect2f_animation_after_physics(before:&LandingRecord)->Result<Effect6UpdatePlan,LandingError>{
 if before[6]!=5||before[69]!=0x2f{return Err(LandingError::InvalidRecord);}
 if before[26]!=0||before[27]>5||before[28]!=1||!(1..=3).contains(&before[29])||before[30]>3{return Err(LandingError::UnsupportedAnimationState);}
 let mut after=*before;let timer=word(&after,74);if timer>=0{put_word(&mut after,74,timer.wrapping_sub(1));}let delay=after[29];let c=after[30].min(delay);after[30]=c.wrapping_sub(1);if c==0{after[30]=delay;after[27]=(after[27]+1)%6;}
 let retired=word(&after,74)<0&&after[27]==5&&after[30]==0;Ok(Effect6UpdatePlan{after,retirement_required:retired})
}
/// Code2Fcommonphysics matchescode6exceptvirtual70alwaysfalse: the
/// obstacle query is skipped by the source, not assumedNULL. No damping.
pub fn plan_effect2f_flight(before:&LandingRecord,ground:i32,shade:u8,water:i32,corners:[i16;4],bounce:i16,angles:&[u16;257])->Result<LandingAirbornePlan,LandingError>{
 if before[6]!=5||before[69]!=0x2f{return Err(LandingError::InvalidRecord);}let mut temporary=*before;temporary[69]=6;let mut plan=plan_effect6_flight(&temporary,ground,shade,water,true,corners,bounce,angles)?;plan.before=*before;plan.after[69]=0x2f;Ok(plan)
}

pub fn effect2f_physics_candidate(before:&LandingRecord)->Result<[i16;2],LandingError>{
 if before[6]!=5||before[69]!=0x2f{return Err(LandingError::InvalidRecord);}let mut temporary=*before;temporary[69]=6;effect6_physics_candidate(&temporary)
}
pub fn effect2f_grounded_candidate(before:&LandingRecord,ground:i32,angles:&[u16;257],sine:&[i32;2048],cosine:&[i32;2048])->Result<LandingRecord,LandingError>{
 if before[6]!=5||before[69]!=0x2f{return Err(LandingError::InvalidRecord);}let mut temporary=*before;temporary[69]=6;let mut after=effect6_grounded_candidate(&temporary,ground,angles,sine,cosine)?;after[69]=0x2f;Ok(after)
}
pub fn plan_effect2f_grounded_after_second_sample(before:&LandingRecord,candidate:&LandingRecord,ground:i32,shade:u8,water:i32)->Result<LandingAirbornePlan,LandingError>{
 if before[6]!=5||before[69]!=0x2f||candidate[6]!=5||candidate[69]!=0x2f{return Err(LandingError::InvalidRecord);}let mut b=*before;let mut c=*candidate;b[69]=6;c[69]=6;let mut plan=plan_effect6_grounded_after_second_sample(&b,&c,ground,shade,water,true)?;plan.before=*before;plan.after[69]=0x2f;Ok(plan)
}

/// Explicit branch inputs to7D090's post-load seed selection. This is not a
/// claim about which mode/profile/clock values a real session used.
pub fn select_landing_startup_seed(
    mode:u8, post_preprocess_seed:u16, saved_seed:u16, local_is_host:bool,
    clock_minute:u8, clock_second:u8, clock_centisecond:u8,
    synchronized_seed:Option<u16>,
)->Result<u16,LandingError>{
    if mode&0x80!=0{return Ok(saved_seed.wrapping_sub(1));}
    if mode&1==0{return Ok(post_preprocess_seed);}
    if local_is_host{return Ok((u16::from(clock_second)+1+u16::from(clock_minute)).wrapping_mul(u16::from(clock_centisecond)+1));}
    synchronized_seed.ok_or(LandingError::MissingSynchronizedSeed)
}

#[cfg(test)]mod tests{
 use super::*;
 fn factory()->LandingFactoryPlan{plan_landing_factory(&[[0;81];200],0,0,0,[20736,13568],6272,30,None).unwrap()}
 #[test]fn factory_preserves_gaps_and_requests_real_landing_reference(){
  let mut pool=[[0xa5;81];200];pool[3][6]=0;let p=plan_landing_factory(&pool,3,1,1,[0,0],10,20,None).unwrap();
  assert_eq!(p.slot,3);assert_eq!(p.after_nonpointer[30],0xa5);assert_eq!(&p.after_nonpointer[35..39],&[0xa5;4]);assert_eq!(word(&p.after_nonpointer,3),word(&p.before,3));assert_eq!(p.animation_index,49);assert_eq!(word(&p.after_nonpointer,49),-1088);assert_eq!(p.pool_count_after,4);
  assert!(p.events.contains(&LandingEvent::SavePlayerLandingReference{owner:1,reference:LandingReference{index:3,kind:5}}));
 }
 #[test]fn contact_is_signed_and_preserves_rng_and_postphysics_angles(){
  let mut a=factory().after_nonpointer;put_dword(&mut a,39,6272);put_word(&mut a,57,123);let r=LandingRng{seed:7,ring:31,draw_count:u32::MAX};let p=plan_landing_phase0_after_physics(&a,6272,r).unwrap();
  assert!(p.contact);assert_eq!(p.rng_after,r);assert_eq!(p.after[70],1);assert_eq!(word(&p.after,74),150);assert_eq!(word(&p.after,57),123);assert_eq!(&p.after[45..51],&[0;6]);assert!(matches!(p.events[0],LandingEvent::ActivateOwner{owner:0}));
 }
 #[test]fn noncontact_draws_three_even_if_effect_result_is_pending(){
  let a=factory().after_nonpointer;let p=plan_landing_phase0_after_physics(&a,6272,LandingRng{seed:0,ring:31,draw_count:u32::MAX}).unwrap();assert!(!p.contact);assert_eq!(p.after,a);assert_eq!(p.draws.len(),3);assert_eq!(p.rng_after,LandingRng{seed:0,ring:2,draw_count:2});assert_eq!(p.created_count,None);
  assert!(matches!(p.events[0],LandingEvent::CreateEffect{angles:[0,1024,2560],mark_newborn_on_success:true,..}));
 }
 #[test]fn airborne_stops_before_unmodeled_contact_physics(){
  let a=factory().after_nonpointer;let p=plan_landing_vertical_airborne(&a,6272,44,1632,true).unwrap();assert_eq!(dword(&p.after,39),dword(&a,39)-136);assert_eq!(p.after[23],44);
  let mut a=a;put_dword(&mut a,39,6273);assert_eq!(plan_landing_vertical_airborne(&a,6272,44,1632,true),Err(LandingError::ContactPhysicsRequired));
 }
 #[test]fn small_positive_vertical_velocity_can_contact_rising_terrain_without_damage(){
  let mut a=factory().after_nonpointer;a[69]=6;put_dword(&mut a,39,6286);put_dword(&mut a,17,6285);put_word(&mut a,49,25);put_word(&mut a,55,-16);
  let p=plan_effect6_flight(&a,6290,40,1632,true,[3145;4],128,&[0;257]).unwrap();assert_eq!(dword(&p.after,39),6290);assert_eq!(word(&p.after,49),4);
  // Numeric projection of the seed1/pass332 slot32 regression; no source bytes.
  let mut a=factory().after_nonpointer;a[69]=6;put_word(&mut a,7,20726);put_word(&mut a,9,13546);a[67]=200;a[68]=172;put_dword(&mut a,17,6285);put_dword(&mut a,39,6286);put_word(&mut a,45,-132);put_word(&mut a,47,-297);put_word(&mut a,49,25);put_word(&mut a,55,-16);
  let p=plan_effect6_flight(&a,6288,0,1632,true,[3456,3008,3136,3136],50,&[0;257]).unwrap();assert_eq!(position(&p.after),[20724,13542]);assert_eq!(&p.after[67..69],&[184,8]);assert_eq!(dword(&p.after,39),6288);assert_eq!(word(&p.after,49),1);
  put_word(&mut a,49,1040);assert_eq!(plan_effect6_flight(&a,6500,40,1632,true,[3250;4],128,&[0;257]),Err(LandingError::UnsupportedPhysicsState));
 }
 #[test]fn full_i16_horizontal_velocity_rejects_without_debug_overflow(){
  let rng=LandingRng{seed:1,ring:0,draw_count:0};let pool=[[0;81];200];
  let edge=plan_landing_effect6_factory(&pool,0,[0,0],0,0,1,[2048,2048,1],None,1024,rng,&[0;2048],&[0;2048]).unwrap();
  assert_eq!(word(&edge.factory.after_nonpointer,57),2048);
  // Source1F070 sign-extends angles, then indexes angle+1536 below512,
  // angle-512 otherwise. The prior2048 ceiling predates the smoke repair.
  for angle in [2049u16,2559,(-1536i16)as u16]{assert!(plan_landing_effect6_factory(&pool,0,[0,0],0,0,1,[angle,0,1],None,1024,rng,&[0;2048],&[0;2048]).is_ok());}
  for angle in [2560u16,(-1537i16)as u16]{assert!(plan_landing_effect6_factory(&pool,0,[0,0],0,0,1,[angle,0,1],None,1024,rng,&[0;2048],&[0;2048]).is_err());}
  let mut a=factory().after_nonpointer;a[69]=6;put_word(&mut a,45,i16::MIN);put_word(&mut a,47,i16::MIN);put_word(&mut a,49,-1);
  assert_eq!(plan_effect6_flight(&a,6272,40,1632,true,[3136;4],128,&[0;257]),Err(LandingError::UnsupportedPhysicsState));
  put_dword(&mut a,39,6272);assert_eq!(effect6_grounded_candidate(&a,6272,&[0;257],&[0;2048],&[0;2048]),Err(LandingError::UnsupportedPhysicsState));
  put_word(&mut a,45,0);put_word(&mut a,47,0);put_word(&mut a,57,-1);assert_eq!(effect6_grounded_candidate(&a,6272,&[0;257],&[0;2048],&[0;2048]),Err(LandingError::UnsupportedPhysicsState));
 }
 #[test]fn phase1_quiet_interval_preserves_state_until_source_rng_boundary(){
  let mut a=factory().after_nonpointer;put_dword(&mut a,39,6272);a=plan_landing_phase0_after_physics(&a,6272,LandingRng{seed:7,ring:0,draw_count:0}).unwrap().after;
  put_word(&mut a,57,123);for _ in 0..100{a=plan_landing_phase1_quiet_grounded(&a,6272,40,1632).unwrap().after;assert_eq!(word(&a,57),123);assert_eq!(word(&a,59),512);assert_eq!(word(&a,61),0);}
  assert_eq!(word(&a,74),50);assert_eq!(plan_landing_phase1_quiet_grounded(&a,6272,40,1632),Err(LandingError::UnsupportedPhysicsState));
 }
 #[test]fn seed_modes_do_not_guess_clock_or_network(){
  assert_eq!(select_landing_startup_seed(0,5428,10,false,0,0,0,None),Ok(5428));assert_eq!(select_landing_startup_seed(0x81,5428,0,false,0,0,0,None),Ok(65535));
  assert_eq!(select_landing_startup_seed(1,0,0,true,4,5,6,None),Ok(70));assert_eq!(select_landing_startup_seed(1,0,0,false,0,0,0,None),Err(LandingError::MissingSynchronizedSeed));
 }
}

///48790/491CA stationary water splash: EFF48, delay2, no RNG.
pub fn plan_water_splash_factory(pool:&[LandingRecord;200],count:i16,at:[i16;2],ground:i32,shade:u8,height:i32,head:Option<ResolvedCellHead>)->Result<LandingFactoryPlan,LandingError>{
 let mut p=plan_landing_factory(pool,count,0,0,at,ground,shade,head)?;
 let a=&mut p.after_nonpointer;put_dword(a,39,height);a[45..63].fill(0);put_word(a,21,0x0820);a[29]=2;a[69]=0x30;a[73]=p.before[73];a[76]=0;p.animation_index=48;
 p.events.retain(|e|matches!(e,LandingEvent::SetCellHead{..}|LandingEvent::SetOldHeadPrevious{..}));
 p.events.push(LandingEvent::PlayAudio(LandingAudioRequest{channel:0x3b,mode:2,parameter:0,stack_byte:127,stack_dword:100}));Ok(p)
}
///6DF30 immobile flag skips integration; virtual70 and90 are false for30.
pub fn plan_water_splash_update(before:&LandingRecord)->Result<Effect6UpdatePlan,LandingError>{
 if before[6]!=5||before[69]!=0x30||before[22]!=8||before[21]&!1!=0x20||before[76]!=0{return Err(LandingError::InvalidRecord);}
 if before[26]!=0||before[27]>=12||before[28]!=1||before[29]!=2{return Err(LandingError::UnsupportedAnimationState);}
 let mut a=*before;put_word(&mut a,7,word(before,7)&0x7fff);put_word(&mut a,9,word(before,9)&0x7fff);
 let timer=word(&a,74);if timer>=0{put_word(&mut a,74,timer.wrapping_sub(1));}
 let c=if (a[30]as i8)>2{2}else{a[30]};a[30]=c.wrapping_sub(1);if (c as i8)<=0{a[30]=2;a[27]=(a[27]+1)%12;}
 Ok(Effect6UpdatePlan{retirement_required:word(&a,74)<0&&a[27]==11&&a[30]==0,after:a})
}

///48846 scans ascending, retaining the first strictly greatest priority above20.
/// Replacement cleanup needs an explicit UI state and remains a world boundary.
pub fn water_splash_replacement(pool:&[LandingRecord;200])->Result<Option<u16>,LandingError>{
 let mut priority=20;let mut selected=None;
 for(i,r)in pool.iter().enumerate(){let p=match r[69]{6=>27,0x2f=>35,0x30=>20,0x21|0x31=>1,_=>return Err(LandingError::UnsupportedPhysicsState)};
  if p>priority&&r[21]&1==0{priority=p;selected=Some(i as u16);}
 }Ok(selected)
}
#[cfg(test)]
mod water_particle_tests {
 use super::*;
 #[test]
 fn type5_false_virtual90_skips_water_callbacks_without_changing_physics(){
  let mut a=[0;81];a[6]=5;a[69]=6;a[21]=0x20;put_word(&mut a,63,-1);put_dword(&mut a,39,1900);
  let dry=plan_effect6_flight(&a,0,40,0,true,[0;4],128,&[0;257]).unwrap();
  let wet=plan_effect6_flight(&a,0,40,1632,true,[0;4],128,&[0;257]).unwrap();assert_eq!(dry,wet);
  a[69]=0x2f;let dry=plan_effect2f_flight(&a,0,40,0,[0;4],128,&[0;257]).unwrap();let wet=plan_effect2f_flight(&a,0,40,1632,[0;4],128,&[0;257]).unwrap();assert_eq!(dry,wet);
  a[69]=6;let dry=plan_effect6_grounded_after_second_sample(&a,&a,1600,40,0,true).unwrap();let wet=plan_effect6_grounded_after_second_sample(&a,&a,1600,40,1632,true).unwrap();assert_eq!(dry,wet);
 }
}
