//! Frame-paced inputs to the native host, not pre-executed gameplay commands.
use std::{collections::VecDeque, path::PathBuf};
use super::runtime_scene::{Game,Scene};

#[derive(Clone,Debug,PartialEq)]
pub enum Input { Pointer(Option<(i32,i32)>), Left(bool), Right(bool), Key(String) }
pub fn dispatch(game:&mut Game,input:Input) {
    match input {
        Input::Pointer(p)=>game.pointer(p),Input::Left(v)=>game.left(v),Input::Right(v)=>game.right(v),
        Input::Key(key)=>match key.as_str() {
            "escape"=>game.escape(),"enter"=>game.toggle_run(),
            "space"=>if game.scene==Scene::Options{game.toggle_run()}else{game.tick()},
            "tab"=>game.select_actor(None),"left"=>game.pan(-256,0),"right"=>game.pan(256,0),
            "up"=>game.pan(0,-256),"down"=>game.pan(0,256),
            "b"=>if let Err(e)=game.start_selected_building(){game.error=Some(e)},_=>{}
        }
    }
}
#[derive(Debug)]
pub enum Action { Input(Input), Observe(String), Wait }
pub struct Replay { pub actions:VecDeque<Action>,pub output:PathBuf }
impl Replay {
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
fn valid_key(k:&str)->bool{["escape","enter","space","tab","left","right","up","down","b"].contains(&k)}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn rejects_gameplay_shortcuts_and_overwrites(){
        for input in ["load","tick 3","select 0","order 320 240","observe ../escape","observe a\nobserve a","key space 0","key space 4097","pointer 640 0"] {
            assert!(Replay::parse(input,PathBuf::new()).is_err(),"{input}");
        }
    }
    #[test]fn expands_only_bounded_inputs(){let r=Replay::parse("pointer 312 140\nleft down\nleft up\nwait\nkey space 189\nobserve landed",PathBuf::new()).unwrap();assert_eq!(r.actions.len(),194);}
}
