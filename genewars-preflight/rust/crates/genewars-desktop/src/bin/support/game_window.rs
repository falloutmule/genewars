use std::{num::NonZeroU32,sync::Arc,time::{Duration,Instant}};
use softbuffer::{Context,Surface};
use winit::{application::ApplicationHandler,dpi::LogicalSize,event::{ElementState,MouseButton,WindowEvent},
    event_loop::{ActiveEventLoop,ControlFlow,EventLoop},keyboard::{Key,NamedKey},window::{Window,WindowId}};
use super::runtime_scene::{Game,Scene};
use super::native_input::{self,Input,Action,Replay};

struct Host<'a> {
    game:&'a mut Game,window:Option<Arc<Window>>,context:Option<Context<Arc<Window>>>,surface:Option<Surface<Arc<Window>,Arc<Window>>>,
    replay:Option<Replay>,replay_complete:bool,next_step:Instant,deadline:Option<Instant>,frame_limit:Option<u64>,error:Option<String>,
}
impl Host<'_> {
    fn fail(&mut self,events:&ActiveEventLoop,error:impl ToString) {self.error=Some(error.to_string());events.exit();}
    fn redraw(&self) {if let Some(w)=&self.window {w.request_redraw();}}
    fn present(&mut self)->Result<bool,String> {
        let window=self.window.as_ref().ok_or("window unavailable")?;
        let size=window.inner_size();
        let (Some(width),Some(height))=(NonZeroU32::new(size.width),NonZeroU32::new(size.height)) else {return Ok(false);};
        let image=self.game.image()?;
        if let Some(mission)=&self.game.mission {
            window.set_title(&format!("GeneWars Rust - {} - {}",String::from_utf8_lossy(&mission.resources.title),if self.game.landing_seed.is_some(){"landing study"}else{"partial plant world"}));
        } else {window.set_title("GeneWars Rust - development build");}
        let surface=self.surface.as_mut().ok_or("surface unavailable")?;
        surface.resize(width,height).map_err(|e|e.to_string())?;
        let mut buffer=surface.buffer_mut().map_err(|e|e.to_string())?;
        let (w,h)=(width.get()as usize,height.get()as usize);
        for y in 0..h {for x in 0..w {
            let at=((y*480/h)*640+x*640/w)*3;
            let rgb=&image.rgb_bytes()[at..at+3];buffer[y*w+x]=(u32::from(rgb[0])<<16)|(u32::from(rgb[1])<<8)|u32::from(rgb[2]);
        }}
        buffer.present().map_err(|e|e.to_string())?;Ok(true)
    }
}
impl ApplicationHandler for Host<'_> {
    fn resumed(&mut self,events:&ActiveEventLoop) {
        if self.window.is_some(){return;}
        let attrs=Window::default_attributes().with_title("GeneWars Rust - partial world development build").with_inner_size(LogicalSize::new(960.0,720.0)).with_min_inner_size(LogicalSize::new(640.0,480.0)).with_resizable(true);
        let window=match events.create_window(attrs){Ok(w)=>Arc::new(w),Err(e)=>return self.fail(events,e)};
        let context=match Context::new(window.clone()){Ok(c)=>c,Err(e)=>return self.fail(events,e)};
        let surface=match Surface::new(&context,window.clone()){Ok(s)=>s,Err(e)=>return self.fail(events,e)};
        window.request_redraw();self.window=Some(window);self.context=Some(context);self.surface=Some(surface);
    }
    fn about_to_wait(&mut self,events:&ActiveEventLoop) {
        let now=Instant::now();
        if self.deadline.is_some_and(|end|now>=end)||self.game.scene==Scene::Quit {events.exit();return;}
        if now>=self.next_step {
            if self.replay.is_none() && self.game.automatic && self.game.scene==Scene::Mission {self.game.tick();self.redraw();}
            self.next_step=now+Duration::from_millis(100);
        }
        events.set_control_flow(ControlFlow::WaitUntil(self.deadline.map_or(self.next_step,|end|end.min(self.next_step))));
    }
    fn window_event(&mut self,events:&ActiveEventLoop,id:WindowId,event:WindowEvent) {
        if self.window.as_ref().is_none_or(|w|w.id()!=id){return;}
        match event {
            WindowEvent::CloseRequested=>events.exit(),
            WindowEvent::Focused(false)=>{self.game.cancel_capture();self.game.automatic=false;self.redraw();},
            WindowEvent::CursorLeft{..}=>{self.game.pointer(None);self.redraw();},
            WindowEvent::CursorMoved{position,..}=>{
                let size=self.window.as_ref().unwrap().inner_size();
                let point=if size.width>0&&size.height>0&&position.x>=0.0&&position.y>=0.0&&position.x<f64::from(size.width)&&position.y<f64::from(size.height){Some(((position.x*640.0/f64::from(size.width)).floor()as i32,(position.y*480.0/f64::from(size.height)).floor()as i32))}else{None};
                native_input::dispatch(self.game,Input::Pointer(point));self.redraw();
            },
            WindowEvent::MouseInput{state,button,..}=>{
                match button {MouseButton::Left=>native_input::dispatch(self.game,Input::Left(state==ElementState::Pressed)),MouseButton::Right=>native_input::dispatch(self.game,Input::Right(state==ElementState::Pressed)),_=>{}}
                self.redraw();
            },
            WindowEvent::KeyboardInput{event,..} if event.state==ElementState::Pressed&&!event.repeat=>{
                let key=match event.logical_key {
                    Key::Named(NamedKey::Escape)=>Some("escape"),Key::Named(NamedKey::Enter)=>Some("enter"),
                    Key::Named(NamedKey::Space)=>Some("space"),Key::Named(NamedKey::Tab)=>Some("tab"),
                    Key::Named(NamedKey::ArrowLeft)=>Some("left"),Key::Named(NamedKey::ArrowRight)=>Some("right"),
                    Key::Named(NamedKey::ArrowUp)=>Some("up"),Key::Named(NamedKey::ArrowDown)=>Some("down"),
                    Key::Character(ref c) if c.eq_ignore_ascii_case("b")=>Some("b"),
                    Key::Character(ref c) if c.eq_ignore_ascii_case("c")=>Some("c"),_=>None
                };
                if let Some(key)=key {native_input::dispatch(self.game,Input::Key(key.into()));}
                self.redraw();
            },
            WindowEvent::Resized(_)=>self.redraw(),
            WindowEvent::RedrawRequested=>{
                match self.present(){Ok(true)=>self.game.presented+=1,Ok(false)=>return,Err(e)=>return self.fail(events,e)}
                if self.game.scene==Scene::Loading {self.game.finish_loading();self.next_step=Instant::now()+Duration::from_millis(100);self.redraw();if self.replay.is_some(){return;}}
                if let Some(replay)=&mut self.replay {
                    match replay.actions.pop_front() {
                        Some(Action::Input(input))=>native_input::dispatch(self.game,input),
                        Some(Action::Wait)=>{},
                        Some(Action::Observe(name))=>{
                            let result=(||->Result<(),String>{
                                super::save_new(&replay.output.join(format!("{name}.json")),self.game.report().as_bytes())?;
                                super::save_new(&replay.output.join(format!("{name}.bmp")),&self.game.image()?.to_bmp_bytes().map_err(|e|e.to_string())?)?;Ok(())
                            })();
                            if let Err(e)=result{return self.fail(events,e);}
                        },
                        None=>{self.replay_complete=true;events.exit();return;}
                    }
                    self.redraw();
                }
                if self.frame_limit.is_some_and(|limit|self.game.presented>=limit){events.exit();}
                else if self.frame_limit.is_some(){self.redraw();}
            },_=>{}
        }
    }
}
pub fn show(game:&mut Game,frame_limit:Option<u64>,seconds:Option<u64>,replay:Option<Replay>)->Result<(),String> {
    let events=EventLoop::new().map_err(|e|e.to_string())?;
    let now=Instant::now();
    let expects_replay=replay.is_some();
    let mut host=Host{replay,replay_complete:false,game,window:None,context:None,surface:None,next_step:now+Duration::from_millis(100),
        deadline:seconds.map(|s|now+Duration::from_secs(s)),frame_limit,error:None};
    events.run_app(&mut host).map_err(|e|e.to_string())?;
    if let Some(e)=host.error {return Err(e);}
    if expects_replay&&!host.replay_complete{return Err("native input replay interrupted before completion".into());}Ok(())
}
