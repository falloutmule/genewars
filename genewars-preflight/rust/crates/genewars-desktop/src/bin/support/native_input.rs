//! Frame-paced inputs to the native host, not pre-executed gameplay commands.
use std::{collections::VecDeque, path::PathBuf};
use super::runtime_scene::{Game,Scene};

#[derive(Clone,Debug,PartialEq)]
pub enum Input { Pointer(Option<(i32,i32)>), Left(bool), Right(bool), Key(String) }
pub fn dispatch(game:&mut Game,input:Input) {
    match input {
        Input::Pointer(p)=>game.pointer(p),Input::Left(v)=>game.left(v),Input::Right(v)=>game.right(v),
        Input::Key(key)=>{if game.building_menu_open&&!matches!(key.as_str(),"c"|"escape"){return;}match key.as_str() {
            "m"=>game.focus_next_mule(),"c"=>game.toggle_building_menu(),"escape"=>game.escape(),"enter"=>game.toggle_run(),
            "space"=>if game.scene==Scene::Options{game.toggle_run()}else{game.tick()},
            "tab"=>game.select_actor(None),"left"=>game.pan(-256,0),"right"=>game.pan(256,0),
            "up"=>game.pan(0,-256),"down"=>game.pan(0,256),
            "b"=>if let Err(e)=game.start_selected_building(){game.error=Some(e)},_=>{}
        }}
    }
}
#[derive(Debug)]
pub enum Action { Input(Input), Observe(String), Wait }
pub struct Replay { pub actions:VecDeque<Action>,pub output:PathBuf }
impl Replay {
    pub fn next(&mut self,game:&Game)->Result<Option<Action>,String> {
        if let Some(m)=&game.mission {if let Some(reason)=&m.blocked {
            return Err(format!("native input replay blocked at step {}: {reason}",m.ticks));
        }}
        Ok(self.actions.pop_front())
    }
    pub fn preserve_blocked(&self,game:&Game,reason:&str)->Result<(),String> {
        // Underscores are forbidden in user observation names, preventing collisions.
        super::save_new(&self.output.join("__blocked.json"),game.report().as_bytes())?;
        super::save_new(&self.output.join("__outcome.json"),format!("{{\"status\":\"blocked\",\"reason\":{},\"remaining_actions\":{}}}\n",super::runtime_scene::json_string(reason),self.actions.len()).as_bytes())?;
        super::save_new(&self.output.join("__blocked.bmp"),&game.image()?.to_bmp_bytes().map_err(|e|e.to_string())?)?;
        Ok(())
    }

    pub fn parse(text:&str,output:PathBuf)->Result<Self,String> {
        if text.len()>65536{return Err("native input replay exceeds 64KiB".into());}
        let mut actions=VecDeque::new();let mut names=std::collections::HashSet::new();
        for (line,text) in text.lines().enumerate() {
            let w=text.split_whitespace().collect::<Vec<_>>();
            if w.is_empty()||w[0].starts_with('#'){continue;}
            let invalid=||format!("invalid native input at line {}",line+1);
            match w.as_slice() {
                ["pointer",x,y]=>{let x=x.parse::<i32>().map_err(|_|invalid())?;let y=y.parse::<i32>().map_err(|_|invalid())?;
                    if !(0..640).contains(&x)||!(0..480).contains(&y){return Err(invalid());}
                    actions.push_back(Action::Input(Input::Pointer(Some((x,y)))));},
                [button,state] if ["left","right"].contains(button)&&["down","up"].contains(state)=>{
                    actions.push_back(Action::Input(if *button=="left"{Input::Left(*state=="down")}else{Input::Right(*state=="down")}));},
                ["key",key]|["key",key,"1"] if valid_key(key)=>actions.push_back(Action::Input(Input::Key(key.to_string()))),
                ["key",key,n] if valid_key(key)=>{let n=n.parse::<usize>().map_err(|_|invalid())?;if !(1..=4096).contains(&n){return Err(invalid());}
                    for _ in 0..n{actions.push_back(Action::Input(Input::Key(key.to_string())));}},
                ["wait"]=>actions.push_back(Action::Wait),
                ["observe",name] if !name.is_empty()&&name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-')=>{
                    if !names.insert(name.to_string()){return Err("duplicate observation name".into());}
                    actions.push_back(Action::Observe(name.to_string()));},
                _=>return Err(invalid())
            }
            if actions.len()>20000{return Err("native input replay exceeds 20000 actions".into());}
        }
        if actions.is_empty(){return Err("empty native input replay".into());}
        Ok(Self{actions,output})
    }
}
fn valid_key(k:&str)->bool{["escape","enter","space","tab","left","right","up","down","b","c","m"].contains(&k)}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn rejects_gameplay_shortcuts_and_overwrites(){
        for input in ["load","tick 3","select 0","order 320 240","observe ../escape","observe a\nobserve a","key space 0","key space 4097","pointer 640 0"] {
            assert!(Replay::parse(input,PathBuf::new()).is_err(),"{input}");
        }
    }
    #[test]fn blocked_native_replay_preserves_world_and_diagnostics(){
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
        let mut game=Game::open(&root).unwrap();game.landing_seed=Some(5428);game.scene=Scene::Loading;game.finish_loading();
        game.mission.as_mut().unwrap().blocked=Some("explicit native harness boundary".into());
        let before=game.report();
        let output=root.parent().unwrap().join(format!("native-stop-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&output).unwrap();
        let mut replay=Replay::parse("key right\nkey tab\nkey c\nkey space 100\nobserve later",output.clone()).unwrap();
        let count=replay.actions.len();let error=replay.next(&game).unwrap_err();
        assert!(error.contains("blocked at step 0: explicit native harness boundary"));
        assert_eq!(replay.actions.len(),count);assert_eq!(game.report(),before);
        replay.preserve_blocked(&game,&error).unwrap();
        assert_eq!(std::fs::read_to_string(output.join("__blocked.json")).unwrap(),before);
        assert!(std::fs::read_to_string(output.join("__outcome.json")).unwrap().contains("\"status\":\"blocked\""));
        assert!(std::fs::metadata(output.join("__blocked.bmp")).unwrap().len()>54);
        assert!(!output.join("later.json").exists());
    }
    #[test]fn expands_only_bounded_inputs(){let r=Replay::parse("pointer 312 140\nleft down\nleft up\nwait\nkey space 189\nobserve landed",PathBuf::new()).unwrap();assert_eq!(r.actions.len(),194);}
}
