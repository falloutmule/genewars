//! Validation-only probe: explicit fixture players/RNG over the real mission.
//! It does not claim that these fixtures are the game's live startup state.
use genewars_assets::ConqMissionSource;
use genewars_runtime::{actor_world::{ActorWorld,ActorWorldError,PlacementEnvironment,GroundMoveInput,GroundMoveOutcome},
    actor_pool::ActorHandleBindings,actor_activation::TrigTables,
    blueprint::BlueprintRng,player_state::{PlayerState,ActorOriginSample}};
use std::{fs,io::{BufWriter,Write},path::Path};

fn snapshot(out:&mut impl Write,world:&ActorWorld) {
    out.write_all(world.backing_bytes()).unwrap();
    for actor in world.actors().records() { out.write_all(actor).unwrap(); }
    for player in 0..4 { out.write_all(world.player(player).unwrap().bytes()).unwrap(); }
    let rng=world.rng();
    out.write_all(&rng.seed.to_le_bytes()).unwrap();
    out.write_all(&[rng.ring]).unwrap();
    out.write_all(&rng.draw_count.to_le_bytes()).unwrap();
}
fn main() {
    let dir=std::env::args().nth(1).unwrap(); let dir=Path::new(&dir);
    let input=fs::read(dir.join("mission.bin")).unwrap();
    let source=ConqMissionSource::parse(&input).unwrap();
    let tables=TrigTables::generate_pinned().unwrap();
    let parameters=fs::read(dir.join("parameters.bin")).unwrap();
    let mut out=BufWriter::new(fs::File::create(dir.join("actual.bin")).unwrap());
    let seeds=[0,1,5428,65535,256,32768,4191,55,3914,29111,8192,13,54676,16384,1929,8888];
    for case in 0..64_usize {
        let owner=(case%4)as u8;
        let center=[[0,0],[32767,32767],[20736,13568],[16383,16384]][case%4];
        let mut players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,center,b"fixture\0").unwrap());
        let mut record=*players[usize::from(owner)].bytes();
        for (index,category) in [(0,0),(10,2)] {
            let start=0x16+index*36;
            record[start]=b'X';
            record[start+0x15]=((case+index)%4)as u8;
            record[start+0x16]=category;
            record[start+0x18]=(case*3)as u8;
            record[start+0x1B]=0xA5;
            record[start+0x1C..start+0x20].copy_from_slice(&(if case%2==0 {u32::MAX}else{0}).to_le_bytes());
        }
        record[0x49D..0x4A1].copy_from_slice(&(if case%3==0 {u32::MAX}else{case as u32}).to_le_bytes());
        players[usize::from(owner)]=PlayerState::from_record(&record).unwrap();
        let prior=core::array::from_fn(|id|core::array::from_fn(|offset|((case+id*37+offset*19)&255)as u8));
        let mut world=ActorWorld::from_mission(&source,&prior,[ActorHandleBindings::empty();20],players,
            BlueprintRng{seed:seeds[case/4],ring:(case%32)as u8,draw_count:u32::MAX-case as u32}).unwrap();
        world.prepare_fresh_player(owner,5,ActorOriginSample{position:[0,0],height:1234+case as i32,shade:67},center).unwrap();
        snapshot(&mut out,&world);
        // A failure in slot1 follows a fully staged slot0, yet rolls back all.
        let mut rejected=world.clone();
        assert_eq!(rejected.activate_without_associated_unit(owner,&tables,|_,slot,_| {
            if slot==1 {Err(ActorWorldError::EnvironmentUnavailable)}
            else {Ok(PlacementEnvironment{terrain_class:1,blocker_weight:0})}
        }).unwrap_err(),ActorWorldError::EnvironmentUnavailable);
        assert_eq!(world,rejected);
        let mut calls=[0_usize;5];
        let receipt=world.activate_without_associated_unit(owner,&tables,|staged,slot,candidate| {
            assert_eq!(staged.rng().seed,candidate.seed_after);
            assert_eq!(staged.actors().record(usize::from(owner)*5+slot).unwrap()[0x70]&1,1);
            calls[slot]+=1;
            Ok(PlacementEnvironment{terrain_class:if calls[slot] <= (case+slot)%4 {0}else{1},blocker_weight:9})
        }).unwrap();
        assert_eq!(receipt.activations.len(),2);
        snapshot(&mut out,&world);
        let actor=world.actors().record(usize::from(owner)*5).unwrap();
        let point=[i16::from_le_bytes([actor[7],actor[8]]),i16::from_le_bytes([actor[9],actor[10]])];
        let requested=point.map(|n|((i32::from(n)+1000)&32767)as i16);
        let outcome=world.dispatch_ground_move(owner,GroundMoveInput {
            buttons:8,modifiers:0,pause_flags:0,group_word_526:0,group_byte_52f:0,
            current_portrait:0,selected:[owner*5,0,3],projected_point:requested,
        },PlacementEnvironment{terrain_class:1,blocker_weight:0},false).unwrap();
        assert!(matches!(outcome,GroundMoveOutcome::Dispatched { opcode:3,.. }));
        snapshot(&mut out,&world);
        let before=world.clone();
        assert!(world.apply_bounded_move_order(owner,i16::from(owner)*5,requested,1,true,0).is_err());
        assert_eq!(world,before);
        world.admit_plants(&parameters).unwrap();
        for _ in 0..96 { world.tick_plants().unwrap(); }
        assert_eq!(world.plant_tick_number(),Some(96));
        snapshot(&mut out,&world);
    }
    out.flush().unwrap();
}
