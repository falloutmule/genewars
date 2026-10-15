//! Rust-first partial mission runtime. Requires private, hash-bound CD assets.
use std::{env,fs,io::{Read,Write},path::{Path,PathBuf},process};
#[path="support/menu_art.rs"] mod menu_art;
#[path="support/runtime_resources.rs"] mod runtime_resources;
#[path="support/actor_scene.rs"] mod actor_scene;
#[path="support/building_scene.rs"] mod building_scene;
#[path="support/family8_scene.rs"] mod family8_scene;
#[path="support/effect_scene.rs"] mod effect_scene;
#[path="support/actor_dispatch_audit.rs"] mod actor_dispatch_audit;
#[path="support/shared_scene.rs"] mod shared_scene;
#[path="support/runtime_scene.rs"] mod runtime_scene;
#[path="support/status_text.rs"] mod status_text;
#[path="support/result_scene.rs"] mod result_scene;
#[path="support/game_window.rs"] mod game_window;
#[path="support/native_input.rs"] mod native_input;
use runtime_scene::Game;

fn read(path:&Path)->Result<Vec<u8>,String>{runtime_resources::read_bound(path)}
fn save_new(path:&Path,bytes:&[u8])->Result<(),String>{
    let mut f=fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e|format!("{}: {e}",path.display()))?;
    f.write_all(bytes).map_err(|e|e.to_string())
}
fn replay(game:&mut Game,text:&str)->Result<(),String>{
    if text.len()>65536{return Err("replay exceeds 64KiB".into());}
    for (line,text) in text.lines().enumerate(){
        let words=text.split_whitespace().collect::<Vec<_>>();
        if words.is_empty()||words[0].starts_with('#'){continue;}
        replay_gameplay_ready(game,line+1)?;
        let n=|i:usize|->Result<i32,String>{words.get(i).ok_or("missing number")?.parse().map_err(|_|"invalid number".into())};
        match words.as_slice(){
            ["move",_,_]=>game.pointer(Some((n(1)?,n(2)?))),
            ["down"]=>game.left(true),["up"]=>game.left(false),["cancel"]=>game.cancel_capture(),
            ["click",_,_]=>{game.pointer(Some((n(1)?,n(2)?)));game.left(true);game.left(false);},
            ["load"]=>game.finish_loading(),
            ["enter"]=>game.toggle_run(),
            ["tick",_]=>{let count=n(1)?;if !(1..=4096).contains(&count){return Err("tick count must be 1..4096".into());}for _ in 0..count {game.tick();replay_gameplay_ready(game,line+1)?;}},
            ["pan",_,_]=>game.pan(i16::try_from(n(1)?).map_err(|_|"pan dx outside i16")?,i16::try_from(n(2)?).map_err(|_|"pan dy outside i16")?),
            ["escape"]=>game.escape(),["toggle"]=>game.toggle_run(),
            ["select",_]=>game.select_actor(Some(usize::try_from(n(1)?).map_err(|_|"negative actor slot")?)),
            ["order",_,_]=>game.order([n(1)?,n(2)?]),
            // Native development affordance only: coordinates pass through
            // the existing screen-to-terrain picker and opcode-3 move gates.
            ["build-order",_,_]=>game.build_order([n(1)?,n(2)?]),
            // The admitted selector-1 command is guarded by opcode-5; normal
            // callbacks retain responsibility for subsequent site setup.
            ["build-place",selector,_,_]=>game.place_selected_building_type(selector.parse().map_err(|_|"invalid building selector")?,[n(2)?,n(3)?])?,
            ["build-start"]=>game.start_selected_building()?,
            ["build-start",selector]=>game.start_selected_building_type(selector.parse().map_err(|_|"invalid building selector")?)?,
            _=>return Err(format!("unknown replay command at line {}",line+1)),
        }
        if game.scene==runtime_scene::Scene::Quit{break;}
    }Ok(())
}
fn replay_gameplay_ready(game:&Game,line:usize)->Result<(),String>{
    if let Some(m)=&game.mission{if let Some(reason)=&m.blocked{return Err(format!("replay blocked at line {line}, step {}: {reason}",m.ticks));}}
    Ok(())
}
fn run()->Result<(),String>{
    let args=env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty(){return Err("usage: genewars-game PRIVATE_RUNTIME_DIR [--landing-seed 0..65535] [--headless REPLAY | --window-replay REPLAY | --native-input-replay INPUT --observations NEW_DIR] [--report NEW_JSON] [--bmp NEW_BMP] [--frames N] [--seconds N]".into());}
    let(mut headless,mut report,mut bmp):(Option<PathBuf>,Option<PathBuf>,Option<PathBuf>)=(None,None,None);
    let mut world_output: Option<PathBuf> = None;
    let mut landing_seed=None;
    let mut reference_startup=false;
    let mut window_replay:Option<PathBuf>=None;
    let mut native_replay:Option<PathBuf>=None;
    let mut observations:Option<PathBuf>=None;
    let(mut frames,mut seconds)=(None,None);let mut i=1;
    while i<args.len(){
        if i+1>=args.len(){return Err("missing option value".into());}
        match args[i].as_str(){
            "--headless" if headless.is_none()=>headless=Some(args[i+1].clone().into()),
            "--window-replay" if window_replay.is_none()=>window_replay=Some(args[i+1].clone().into()),
            "--native-input-replay" if native_replay.is_none()=>native_replay=Some(args[i+1].clone().into()),
            "--observations" if observations.is_none()=>observations=Some(args[i+1].clone().into()),
            "--report" if report.is_none()=>report=Some(args[i+1].clone().into()),
            "--bmp" if bmp.is_none()=>bmp=Some(args[i+1].clone().into()),
            "--world-output" if world_output.is_none()=>world_output=Some(args[i+1].clone().into()),
            "--reference-startup" if !reference_startup&&args[i+1]=="fresh"=>reference_startup=true,
            "--landing-seed" if landing_seed.is_none()=>landing_seed=Some(args[i+1].parse::<u16>().map_err(|_|"landing seed must be 0..65535")?),
            "--frames" if frames.is_none()=>{let n=args[i+1].parse::<u64>().map_err(|_|"invalid frame count")?;if !(1..=100000).contains(&n){return Err("frames outside 1..100000".into());}frames=Some(n);},
            "--seconds" if seconds.is_none()=>{let n=args[i+1].parse::<u64>().map_err(|_|"invalid seconds")?;if !(1..=1200).contains(&n){return Err("seconds outside 1..1200".into());}seconds=Some(n);},
            other=>return Err(format!("unknown or duplicate option {other}")),
        }i+=2;
    }
    if headless.is_some()&&(frames.is_some()||seconds.is_some()){return Err("window bounds cannot be used with headless replay".into());}
    if native_replay.is_some()&&(headless.is_some()||window_replay.is_some()||seconds.is_none()||observations.is_none()){return Err("native input replay requires --seconds and --observations, and excludes other replay modes".into());}
    if observations.is_some()&&native_replay.is_none(){return Err("observations require native input replay".into());}
    if headless.is_some()&&window_replay.is_some(){return Err("choose one replay mode".into());}
    if window_replay.is_some()&&frames.is_none()&&seconds.is_none(){return Err("window replay requires a frame or time bound".into());}
    if landing_seed.is_some()&&world_output.is_some(){return Err("landing study requires full-state export; --world-output only supports the plant world".into());}
    for p in [&report,&bmp,&world_output].into_iter().flatten(){if p.exists(){return Err(format!("refusing to overwrite {}",p.display()));}}
    let mut game=Game::open(Path::new(&args[0]))?;
    if reference_startup&&landing_seed.is_some(){return Err("reference startup uses the serialized RNG; omit --landing-seed".into());}
    game.landing_seed=landing_seed;game.reference_startup=reference_startup;
    let show_window=headless.is_none();
    let mut replay_error=None;
    if let Some(path)=headless.or(window_replay){let mut text=String::new();fs::File::open(path).map_err(|e|e.to_string())?.take(65537).read_to_string(&mut text).map_err(|e|e.to_string())?;replay_error=replay(&mut game,&text).err();}
    let native=if let Some(path)=native_replay {
        let mut text=String::new();fs::File::open(path).map_err(|e|e.to_string())?.take(65537).read_to_string(&mut text).map_err(|e|e.to_string())?;
        let output=observations.unwrap();let replay=native_input::Replay::parse(&text,output.clone())?;
        fs::create_dir(&output).map_err(|e|format!("{}: {e}",output.display()))?;Some(replay)
    }else{None};
    if show_window&&replay_error.is_none(){game_window::show(&mut game,frames,seconds,native)?;}
    if let Some(path)=bmp{save_new(&path,&game.image()?.to_bmp_bytes().map_err(|e|e.to_string())?)?;}
    if let Some(path)=world_output {save_new(&path,game.mission.as_ref().ok_or("no mission to export")?.world.source_bytes()?)?;}
    let json=game.report();if let Some(path)=report{save_new(&path,json.as_bytes())?;}print!("{json}");if let Some(error)=replay_error{return Err(error);}Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("{e}");process::exit(1);}}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn blocked_replay_preserves_failure_fixture_before_later_commands(){
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
        let mut game=Game::open(&root).unwrap();game.landing_seed=Some(5428);game.scene=runtime_scene::Scene::Loading;game.finish_loading();
        game.mission.as_mut().unwrap().blocked=Some("explicit harness boundary".into());
        let before=game.report();let error=replay(&mut game,"pan 256 256\nselect 0\nbuild-place 3 400 370\ntick 100\n").unwrap_err();
        assert!(error.contains("line 1, step 0: explicit harness boundary"));assert_eq!(game.report(),before);
    }
    #[test]fn reference_startup_keeps_locked_buildings_unavailable(){
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
        let mut game=Game::open(&root).unwrap();game.reference_startup=true;game.scene=runtime_scene::Scene::Loading;game.finish_loading();
        let before=game.report();assert_eq!(game.start_selected_building().unwrap_err(),"building is not available");assert_eq!(game.report(),before);
    }
    #[test]fn json_quotes_controls_and_unicode(){assert_eq!(runtime_scene::json_string("a\"\\\n\tÃ¢Ëœâ‚¬"),"\"a\\\"\\\\\\n\\tÃ¢Ëœâ‚¬\"");}
    #[test]fn catalog_is_unique_and_bounded(){
        let mut names=std::collections::HashSet::new();let mut count=0;
        for line in runtime_resources::CATALOG.lines().filter(|l|!l.starts_with('#')){
            let f=line.split('\t').collect::<Vec<_>>();assert_eq!(f.len(),5);assert!(names.insert(f[0]));
            assert_eq!(f[1].len(),64);assert!(f[1].bytes().all(|b|b.is_ascii_hexdigit()));
            assert!(!f[0].contains(".."));assert!(f[2].parse::<usize>().unwrap()<1024*1024);count+=1;
        }assert!(count>=57);
        for required in ["menu/text.dat","terrain/CONQ001.DAT","results/WINSCR.DAT","campaign/LEVSCR01.DAT","terrain/CONQ002.DAT"]{assert!(names.contains(required),"required reference resource absent: {required}");}
    }

    fn development_build_fixture() -> Game {
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
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

    #[test]
    fn development_build_flow_uses_picked_move_and_opcode5_idle_guard() {
        let _game=development_build_fixture();
    }

    #[test]
    fn development_build_continuation_completes_and_returns_architect_to_idle() {
        let mut game=development_build_fixture();
        let mut completed_at=None;
        let mut idle_after_completion=None;
        let mut callbacks=0usize;
        for _ in 0..2000 {
            game.tick();callbacks+=1;
            let mission=game.mission.as_ref().unwrap();
            let runtime_scene::MissionState::Landing(world)=&mission.world else {panic!("landing study expected")};
            let building=world.world.buildings().unwrap().records()[0];
            let flags=u16::from_le_bytes([building[0x31],building[0x32]]);
            let health=i16::from_le_bytes([building[0x2b],building[0x2c]]);
            let actor=world.world.actors().record(0).unwrap();
            if flags&1!=0&&completed_at.is_none(){completed_at=Some((callbacks,health));}
            if completed_at.is_some()&&actor[0x55]==1&&actor[0x57]&3==0&&actor[0x82]==0 {
                idle_after_completion=Some(callbacks);break;
            }
            if let Some(reason)=&mission.blocked {
                if callbacks == 161 {
                    let boundary=std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../../../private/review/build-boundary-callback161");
                    std::fs::create_dir_all(&boundary).expect("private boundary directory");
                    std::fs::write(boundary.join("backing.bin"),world.world.backing_bytes()).expect("backing snapshot");
                    std::fs::write(boundary.join("actors.bin"),world.world.actors().records().as_flattened()).expect("actor snapshot");
                    if let Some(pool)=world.world.creatures(){std::fs::write(boundary.join("creatures.bin"),pool.records().as_flattened()).expect("creature snapshot");}
                    if let Some(pool)=world.world.effects(){std::fs::write(boundary.join("effects.bin"),pool.records().as_flattened()).expect("effect snapshot");}
                    if let Some(pool)=world.world.buildings(){std::fs::write(boundary.join("buildings.bin"),pool.records().as_flattened()).expect("building snapshot");}
                    if let Some(pool)=world.world.family8(){std::fs::write(boundary.join("family8.bin"),pool.records().as_flattened()).expect("family8 snapshot");}
                    for player in 0..4u8 {
                        std::fs::write(boundary.join(format!("player-{player}.bin")),world.world.player(player).unwrap().bytes()).expect("player snapshot");
                    }
                    std::fs::write(boundary.join("state.txt"),format!("callback={callbacks}\nseed={}\nrng={:?}\ntarget={:?}\ntick={}\nblocked={reason}\n",game.landing_seed.unwrap_or_default(),world.world.rng(),mission.target,mission.ticks)).expect("state metadata");
                }
                panic!("construction continuation blocked at callback {callbacks}: {reason}; committed health={health}, flags={flags:04x}, actor={actor:02X?}");
            }
        }
        assert!(completed_at.is_some(),"bounded normal callbacks did not set completed flag: callbacks={callbacks}, report={}",game.report());
        assert_eq!(idle_after_completion,Some(callbacks),"architect did not return to ordinary idle within the bounded continuation: completed_at={completed_at:?}, callbacks={callbacks}, report={}",game.report());
        let mission=game.mission.as_ref().unwrap();
        let runtime_scene::MissionState::Landing(world)=&mission.world else {panic!("landing study expected")};
        let building=world.world.buildings().unwrap().records()[0];
        assert_eq!(i16::from_le_bytes([building[0x2b],building[0x2c]]),120);
        assert_eq!(u16::from_le_bytes([building[0x31],building[0x32]])&1,1);
        assert_eq!(game.mission.as_ref().unwrap().blocked,None);
        eprintln!("development build completion: callbacks={callbacks}, completed_at={completed_at:?}, idle_at={idle_after_completion:?}, balance={}, actor={:02X?}, building_progress=120 flags={:04x}",
            i32::from_le_bytes(world.world.player(0).unwrap().bytes()[0x4a3..0x4a7].try_into().unwrap()),world.world.actors().record(0).unwrap(),u16::from_le_bytes([building[0x31],building[0x32]]));
    }
}
