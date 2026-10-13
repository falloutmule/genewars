//! Rust-first partial mission runtime. Requires private, hash-bound CD assets.
use std::{env,fs,io::{Read,Write},path::{Path,PathBuf},process};
#[path="support/menu_art.rs"] mod menu_art;
#[path="support/runtime_resources.rs"] mod runtime_resources;
#[path="support/actor_scene.rs"] mod actor_scene;
#[path="support/family8_scene.rs"] mod family8_scene;
#[path="support/effect_scene.rs"] mod effect_scene;
#[path="support/shared_scene.rs"] mod shared_scene;
#[path="support/runtime_scene.rs"] mod runtime_scene;
#[path="support/status_text.rs"] mod status_text;
#[path="support/game_window.rs"] mod game_window;
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
            _=>return Err(format!("unknown replay command at line {}",line+1)),
        }
        if game.scene==runtime_scene::Scene::Quit{break;}
    }Ok(())
}
fn run()->Result<(),String>{
    let args=env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty(){return Err("usage: genewars-game PRIVATE_RUNTIME_DIR [--landing-seed 0..65535] [--headless REPLAY | --window-replay REPLAY] [--report NEW_JSON] [--bmp NEW_BMP] [--frames N] [--seconds N]".into());}
    let(mut headless,mut report,mut bmp):(Option<PathBuf>,Option<PathBuf>,Option<PathBuf>)=(None,None,None);
    let mut world_output: Option<PathBuf> = None;
    let mut landing_seed=None;
    let mut window_replay:Option<PathBuf>=None;
    let(mut frames,mut seconds)=(None,None);let mut i=1;
    while i<args.len(){
        if i+1>=args.len(){return Err("missing option value".into());}
        match args[i].as_str(){
            "--headless" if headless.is_none()=>headless=Some(args[i+1].clone().into()),
            "--window-replay" if window_replay.is_none()=>window_replay=Some(args[i+1].clone().into()),
            "--report" if report.is_none()=>report=Some(args[i+1].clone().into()),
            "--bmp" if bmp.is_none()=>bmp=Some(args[i+1].clone().into()),
            "--world-output" if world_output.is_none()=>world_output=Some(args[i+1].clone().into()),
            "--landing-seed" if landing_seed.is_none()=>landing_seed=Some(args[i+1].parse::<u16>().map_err(|_|"landing seed must be 0..65535")?),
            "--frames" if frames.is_none()=>{let n=args[i+1].parse::<u64>().map_err(|_|"invalid frame count")?;if !(1..=100000).contains(&n){return Err("frames outside 1..100000".into());}frames=Some(n);},
            "--seconds" if seconds.is_none()=>{let n=args[i+1].parse::<u64>().map_err(|_|"invalid seconds")?;if !(1..=1200).contains(&n){return Err("seconds outside 1..1200".into());}seconds=Some(n);},
            other=>return Err(format!("unknown or duplicate option {other}")),
        }i+=2;
    }
    if headless.is_some()&&(frames.is_some()||seconds.is_some()){return Err("window bounds cannot be used with headless replay".into());}
    if headless.is_some()&&window_replay.is_some(){return Err("choose one replay mode".into());}
    if window_replay.is_some()&&frames.is_none()&&seconds.is_none(){return Err("window replay requires a frame or time bound".into());}
    if landing_seed.is_some()&&world_output.is_some(){return Err("landing study requires full-state export; --world-output only supports the plant world".into());}
    for p in [&report,&bmp,&world_output].into_iter().flatten(){if p.exists(){return Err(format!("refusing to overwrite {}",p.display()));}}
    let mut game=Game::open(Path::new(&args[0]))?;
    game.landing_seed=landing_seed;
    let show_window=headless.is_none();
    if let Some(path)=headless.or(window_replay){let mut text=String::new();fs::File::open(path).map_err(|e|e.to_string())?.take(65537).read_to_string(&mut text).map_err(|e|e.to_string())?;replay(&mut game,&text)?;}
    if show_window{game_window::show(&mut game,frames,seconds)?;}
    if let Some(path)=bmp{save_new(&path,&game.image()?.to_bmp_bytes().map_err(|e|e.to_string())?)?;}
    if let Some(path)=world_output {save_new(&path,game.mission.as_ref().ok_or("no mission to export")?.world.source_bytes()?)?;}
    let json=game.report();if let Some(path)=report{save_new(&path,json.as_bytes())?;}print!("{json}");Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("{e}");process::exit(1);}}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn json_quotes_controls_and_unicode(){assert_eq!(runtime_scene::json_string("a\"\\\n\tâ˜€"),"\"a\\\"\\\\\\n\\tâ˜€\"");}
    #[test]fn catalog_is_unique_and_bounded(){
        let mut names=std::collections::HashSet::new();let mut count=0;
        for line in runtime_resources::CATALOG.lines().filter(|l|!l.starts_with('#')){
            let f=line.split('\t').collect::<Vec<_>>();assert_eq!(f.len(),5);assert!(names.insert(f[0]));
            assert_eq!(f[1].len(),64);assert!(f[1].bytes().all(|b|b.is_ascii_hexdigit()));
            assert!(!f[0].contains(".."));assert!(f[2].parse::<usize>().unwrap()<1024*1024);count+=1;
        }assert_eq!(count,51);
    }
}
