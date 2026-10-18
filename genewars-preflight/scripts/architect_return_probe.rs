use std::{path::Path,io::Write};
#[path="../rust/crates/genewars-desktop/src/bin/support/menu_art.rs"] mod menu_art;
#[path="../rust/crates/genewars-desktop/src/bin/support/runtime_resources.rs"] mod runtime_resources;
#[path="../rust/crates/genewars-desktop/src/bin/support/actor_scene.rs"] mod actor_scene;
#[path="../rust/crates/genewars-desktop/src/bin/support/building_scene.rs"] mod building_scene;
#[path="../rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"] mod family8_scene;
#[path="../rust/crates/genewars-desktop/src/bin/support/effect_scene.rs"] mod effect_scene;
#[path="../rust/crates/genewars-desktop/src/bin/support/shared_scene.rs"] mod shared_scene;
#[path="../rust/crates/genewars-desktop/src/bin/support/runtime_scene.rs"] mod runtime_scene;
#[path="../rust/crates/genewars-desktop/src/bin/support/status_text.rs"] mod status_text;
use runtime_scene::Game;
fn read(path:&Path)->Result<Vec<u8>,String>{runtime_resources::read_bound(path)}
fn replay(game:&mut Game,text:&str)->Result<(),String>{
    if text.len()>65536{return Err("replay exceeds 64KiB".into());}
    for (line,text) in text.lines().enumerate(){
        let words=text.split_whitespace().collect::<Vec<_>>();
        if words.is_empty()||words[0].starts_with('#'){continue;}
        let n=|i:usize|->Result<i32,String>{words.get(i).ok_or("missing number")?.parse().map_err(|_|"invalid number".into())};
        match words.as_slice(){
            ["move",_,_]=>game.pointer(Some((n(1)?,n(2)?))),
            ["down"]=>game.left(true),["up"]=>game.left(false),["cancel"]=>game.cancel_capture(),
            ["click",_,_]=>{game.pointer(Some((n(1)?,n(2)?)));game.left(true);game.left(false);},
            ["load"]=>game.finish_loading(),
            ["tick",_]=>{let count=n(1)?;if !(1..=4096).contains(&count){return Err("tick count must be 1..4096".into());}for _ in 0..count {game.tick();}},
            ["pan",_,_]=>game.pan(i16::try_from(n(1)?).map_err(|_|"pan dx outside i16")?,i16::try_from(n(2)?).map_err(|_|"pan dy outside i16")?),
            ["escape"]=>game.escape(),["toggle"]=>game.toggle_run(),
            ["select",_]=>game.select_actor(Some(usize::try_from(n(1)?).map_err(|_|"negative actor slot")?)),
            ["order",_,_]=>game.order([n(1)?,n(2)?]),
            // Native development affordance only: coordinates pass through
            // the existing screen-to-terrain picker and opcode-3 move gates.
            ["build-order",_,_]=>game.build_order([n(1)?,n(2)?]),
            // The admitted selector-1 command is guarded by opcode-5; normal
            // callbacks retain responsibility for subsequent site setup.
            ["build-start"]=>game.start_selected_building()?,
            _=>return Err(format!("unknown replay command at line {}",line+1)),
        }
        if game.scene==runtime_scene::Scene::Quit{break;}
    }Ok(())
}
    fn development_build_fixture() -> Game {
        let root=Path::new(&std::env::var("GENEWARS_REVIEW_ROOT").unwrap()).join("private/integration/game-runtime");
        let mut game=Game::open(&root).expect("staged runtime resources");
        game.landing_seed=Some(5428);
        game.scene=runtime_scene::Scene::Loading;
        game.finish_loading();
        assert_eq!(game.scene,runtime_scene::Scene::Mission,"mission resources must load before commands");
        let runtime_scene::MissionState::Landing(_)=&game.mission.as_ref().unwrap().world else {panic!("landing study expected")};
        assert!(game.mission.as_ref().unwrap().resources.mule_animations.frame_metadata(26,4,0).is_ok(),
            "Resources::load must bind the staged BONES database into the live Mule animation resolver");
        // Pan the visible camera to the source-validated flat construction
        // cell. The subsequent command still uses a real screen-space pick.
        game.pan(-897,-641);
        assert_eq!(game.mission.as_ref().unwrap().target,[19839,12927]);

        replay(&mut game,"tick 189\nselect 0\n").expect("reach active actor through ordinary startup callbacks");
        let mission=game.mission.as_ref().unwrap();
        let runtime_scene::MissionState::Landing(world)=&mission.world else {panic!("landing study expected")};
        let camera=genewars_core::cd_candidate::camera_cells_for_target(
            world.world.map(),mission.target[0],mission.target[1]).unwrap();
        let mut picked:Option<([i32;2],[i16;2],i32)>=None;
        for y in 0..480 {
            for x in 128..640 {
                if let Ok(hit)=genewars_runtime::terrain_target::project_terrain_target(world.world.map(),camera,[x,y]) {
                    if hit.cell_index==50*128+77 {
                        let distance=i32::from(hit.point[0]).abs_diff(19839) as i32
                            + i32::from(hit.point[1]).abs_diff(12927) as i32;
                        if picked.map_or(true,|(_,_,best)|distance<best) {
                            picked=Some(([x,y],hit.point,distance));
                        }
                    }
                }
            }
        }
        let (screen,picked_target,_)=picked.expect("nearby screen pixels must expose the admitted construction cell");
        eprintln!("development build pick: screen={screen:?}, exact_picker_target={picked_target:?}");
        replay(&mut game,&format!("build-order {} {}\ntick 100\nbuild-start\n",screen[0],screen[1]))
            .expect("real terrain pick, normal callbacks, and guarded opcode-5 assignment");
        let runtime_scene::MissionState::Landing(world)=&game.mission.as_ref().unwrap().world else {panic!("landing study expected")};
        let actor=world.world.actors().record(0).unwrap();
        let position=[i16::from_le_bytes([actor[7],actor[8]]),i16::from_le_bytes([actor[9],actor[10]])];
        let destination=[i16::from_le_bytes([actor[11],actor[12]]),i16::from_le_bytes([actor[13],actor[14]])];
        assert_eq!(actor[0x55],1,"actor must reach source idle state before assignment");
        assert_eq!(position,destination,"picked movement must complete through ordinary callbacks");
        assert_eq!(destination,picked_target,"the actor reaches the exact point returned by the screen picker");
        assert_eq!(actor[0x82],1,"opcode5 assignment latch");
        assert_eq!(actor[0x83],1,"packet 0x0100 selects admitted BTURR selector 1");
        assert_ne!(actor[0x57]&1,0,"source task-pending flag");
        assert!(game.trace.iter().any(|line|line.starts_with("DevelopmentBuildMove:")));
        assert!(game.trace.iter().any(|line|line.starts_with("DevelopmentBuildStart:")));

        let before=*actor;
        assert!(game.start_selected_building().is_err(),"opcode5 must reject reinitializing its pending assignment");
        let runtime_scene::MissionState::Landing(world)=&game.mission.as_ref().unwrap().world else {panic!("landing study expected")};
        assert_eq!(world.world.actors().record(0).unwrap(),&before,"rejected reinitialization must be atomic");

        let funds_before=u32::from_le_bytes(world.world.player(0).unwrap().bytes()[0x4a3..0x4a7].try_into().unwrap());
        let mut post_assignment_callbacks=0usize;
        for _ in 0..400 {
            game.tick();
            post_assignment_callbacks+=1;
            let runtime_scene::MissionState::Landing(current)=&game.mission.as_ref().unwrap().world else {panic!("landing study expected")};
            let active_buildings=current.world.buildings().unwrap().records().iter().filter(|r|r[6]!=0).count();
            if current.active_terrain_job_count()!=0 || active_buildings!=0
                || current.world.actors().record(0).unwrap()[0x59]==13 {break;}
            assert!(game.mission.as_ref().unwrap().blocked.is_none(),"normal callbacks should either set up the picked site or report a precise block");
        }
        let mission=game.mission.as_ref().unwrap();
        let runtime_scene::MissionState::Landing(world)=&mission.world else {panic!("landing study expected")};
        let active_buildings=world.world.buildings().unwrap().records().iter().filter(|r|r[6]!=0).count();
        let built_actor=world.world.actors().record(0).unwrap();
        assert!(world.active_terrain_job_count()!=0 || active_buildings!=0
            || built_actor[0x59]==13,
            "normal callbacks did not set up the picked site: point={picked_target:?}, actor={:02X?}, jobs={}, buildings={active_buildings}, blocked={:?}",
            built_actor, world.active_terrain_job_count(), mission.blocked);
        assert_eq!(built_actor[0x59],13,
            "bounded replay should reach source action 13, callbacks={post_assignment_callbacks}, job_count={}, buildings={active_buildings}",
            world.active_terrain_job_count());
        assert_eq!(world.terrain_job_records().iter().filter(|r|i16::from_le_bytes([r[12],r[13]])>=0).count(),
            usize::from(world.active_terrain_job_count()));
        let funds_after=u32::from_le_bytes(world.world.player(0).unwrap().bytes()[0x4a3..0x4a7].try_into().unwrap());
        assert_eq!(funds_after,funds_before,"site setup is not a fabricated payment; source debit belongs to a later callback");
        assert_eq!(funds_before,2000,"campaign funds must come from the source-backed player initialization");
        let before_construction_rgb=game.mission.as_ref().unwrap().frame.rgb_sha256.clone();

        let mut callbacks_to_creation=0usize;
        let mut paid_at=None;
        let mut created_at=None;
        for _ in 0..5000 {
            game.tick();
            callbacks_to_creation+=1;
            let mission=game.mission.as_ref().unwrap();
            let runtime_scene::MissionState::Landing(current)=&mission.world else {panic!("landing study expected")};
            let balance=u32::from_le_bytes(current.world.player(0).unwrap().bytes()[0x4a3..0x4a7].try_into().unwrap());
            let active=current.world.buildings().unwrap().records().iter()
                .filter(|r|r[6]==4&&r[24]==0&&r[0x2d]==1).count();
            if balance==1312&&paid_at.is_none(){paid_at=Some(callbacks_to_creation);}
            if active!=0 {created_at=Some(callbacks_to_creation);break;}
            if let Some(reason)=&mission.blocked {
                panic!("construction callback blocked before selector-1 creation after {callbacks_to_creation} ordinary callbacks: {reason}; actor={:02X?}; jobs={}; report={}",
                    current.world.actors().record(0).unwrap(),current.active_terrain_job_count(),game.report());
            }
        }
        let mission=game.mission.as_ref().unwrap();
        let runtime_scene::MissionState::Landing(world)=&mission.world else {panic!("landing study expected")};
        let funds_final=u32::from_le_bytes(world.world.player(0).unwrap().bytes()[0x4a3..0x4a7].try_into().unwrap());
        let buildings=world.world.buildings().unwrap().records().iter().enumerate()
            .filter(|(_,r)|r[6]==4&&r[24]==0&&r[0x2d]==1).collect::<Vec<_>>();
        assert_eq!(buildings.len(),1,"expected one visible owner-0 selector-1 building; report={}",game.report());
        let (building_slot,building)=buildings[0];
        eprintln!("development build flow: post-assignment callbacks={post_assignment_callbacks}, callbacks-to-payment={paid_at:?}, callbacks-to-building={created_at:?}, funds={funds_before}->{funds_final}, building_slot={building_slot}, progress_word_2b={}, phase_byte_2f={}",
            i16::from_le_bytes([building[0x2b],building[0x2c]]),building[0x2f]);
        assert_eq!(funds_final,1312,"source callback should debit exactly 688 before creating selector 1; callbacks={callbacks_to_creation}, paid_at={paid_at:?}, created_at={created_at:?}, report={}",game.report());
        assert_eq!(funds_before-funds_final,688);
        assert!(created_at.is_some(),"bounded normal callbacks did not create the building; callbacks={callbacks_to_creation}, paid_at={paid_at:?}, report={}",game.report());
        let rendered=world.render(&mission.resources,mission.target).expect("created building must remain renderable");
        assert_eq!(rendered.rgb_sha256,mission.frame.rgb_sha256,"committed frame must be the source-backed scene render containing the active building");
        assert_ne!(mission.frame.rgb_sha256,before_construction_rgb,"construction replay must produce a visibly changed rendered frame");
        let report=game.report();
        assert!(report.contains("\"player_funds\":[{\"slot\":0,\"balance\":1312}"));
        assert!(report.contains("\"selector\":1"));
        assert!(report.contains("\"terrain_jobs\":{\"count\":"));
        game
    }

    fn emit_pre161_production_afterimages_private() {
        let mut game=development_build_fixture();
        eprintln!("fixture tick={} frame={}",game.mission.as_ref().unwrap().ticks, match &game.mission.as_ref().unwrap().world {runtime_scene::MissionState::Landing(w)=>w.frame_counter,_=>0});
        for _ in 0..160 { game.tick(); assert!(game.mission.as_ref().unwrap().blocked.is_none(), "blocked: {:?}",game.mission.as_ref().unwrap().blocked); }
        let mission=game.mission.as_ref().unwrap();
        let runtime_scene::MissionState::Landing(shared)=&mission.world else {panic!("landing")};
        let before=shared.world.actors().record(0).unwrap();
        let ctx=shared.world.mission_loading_physics();
        let trig=genewars_runtime::actor_activation::TrigTables::generate_pinned().unwrap();
        let mut moved=shared.world.clone();
        let motion=moved.step_architect_motion(0,514,ctx,&mission.resources.physics,&trig).unwrap();
        let motion_after=*moved.actors().record(0).unwrap();
        let out=std::path::PathBuf::from(std::env::var("GENEWARS_REVIEW_OUTPUT").unwrap());
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(out.join("pre-actor.bin"),before).unwrap();
        std::fs::write(out.join("pre-actors.bin"),shared.world.actors().records().as_flattened()).unwrap();
        std::fs::write(out.join("motion-after-actors.bin"),moved.actors().records().as_flattened()).unwrap();
        std::fs::write(out.join("motion-after-actor.bin"),motion_after).unwrap();
        std::fs::write(out.join("motion-after-backing.bin"),moved.backing_bytes()).unwrap();
        if let Some(pool)=moved.creatures(){std::fs::write(out.join("motion-after-creatures.bin"),pool.records().as_flattened()).unwrap();}
        if let Some(pool)=moved.effects(){std::fs::write(out.join("motion-after-effects.bin"),pool.records().as_flattened()).unwrap();}
        if let Some(pool)=moved.buildings(){std::fs::write(out.join("motion-after-buildings.bin"),pool.records().as_flattened()).unwrap();}
        if let Some(pool)=moved.family8(){std::fs::write(out.join("motion-after-family8.bin"),pool.records().as_flattened()).unwrap();}
        for player in 0..4u8{std::fs::write(out.join(format!("motion-after-player-{player}.bin")),moved.player(player).unwrap().bytes()).unwrap();}
        std::fs::write(out.join("motion-report.txt"),format!("frame=514\nframe_counter={}\nreceipt={motion:?}\nrng={:?}\n",shared.frame_counter, moved.rng())).unwrap();
        eprintln!("pre actor state={} action={} stage={} target={:?}; motion={motion:?}",before[0x55],before[0x59],i16::from_le_bytes([before[0x88],before[0x89]]),[i16::from_le_bytes([before[11],before[12]]),i16::from_le_bytes([before[13],before[14]])]);
        let mut callback_world=shared.world.clone();
        let mut jobs=genewars_runtime::terrain_job_pool::TerrainJobPool::admit(*shared.terrain_job_records(),shared.active_terrain_job_count()).unwrap();
        std::fs::write(out.join("pre-jobs.bin"),jobs.records().as_flattened()).unwrap();
        let key=callback_world.actors().bindings(0).unwrap().animation_database.unwrap();
        let actor_resource=mission.resources.actors.database(key).unwrap();
        let callback=callback_world.step_architect_construction_callback(0,514,ctx,&mission.resources.physics,&trig,
            &mut jobs,&mission.resources.construction_border,actor_resource,true,true);
        match callback {
            Ok(receipt)=>{
                std::fs::write(out.join("callback-after-actor.bin"),callback_world.actors().record(0).unwrap()).unwrap();
                std::fs::write(out.join("callback-after-actors.bin"),callback_world.actors().records().as_flattened()).unwrap();
                std::fs::write(out.join("callback-after-backing.bin"),callback_world.backing_bytes()).unwrap();
                if let Some(pool)=callback_world.creatures(){std::fs::write(out.join("callback-after-creatures.bin"),pool.records().as_flattened()).unwrap();}
                if let Some(pool)=callback_world.effects(){std::fs::write(out.join("callback-after-effects.bin"),pool.records().as_flattened()).unwrap();}
                if let Some(pool)=callback_world.buildings(){std::fs::write(out.join("callback-after-buildings.bin"),pool.records().as_flattened()).unwrap();}
                if let Some(pool)=callback_world.family8(){std::fs::write(out.join("callback-after-family8.bin"),pool.records().as_flattened()).unwrap();}
                for player in 0..4u8{std::fs::write(out.join(format!("callback-after-player-{player}.bin")),callback_world.player(player).unwrap().bytes()).unwrap();}
                std::fs::write(out.join("callback-after-jobs.bin"),jobs.records().as_flattened()).unwrap();
                std::fs::write(out.join("callback-report.txt"),format!("receipt={receipt:?}\nrng={:?}\njobs={}\n",callback_world.rng(),jobs.count())).unwrap();
            },
            Err(error)=>{std::fs::write(out.join("callback-error.txt"),format!("{error:?}\n")).unwrap();}
        }
    }
fn main(){emit_pre161_production_afterimages_private();}
