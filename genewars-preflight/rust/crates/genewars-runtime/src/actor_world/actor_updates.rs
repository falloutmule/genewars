//! Owned bounded activated actor movement, successful boundary routing and idle.
//! Exhausted searches, active target AI and charge states reject transactionally.
use super::*;
use crate::actor_motion::{self,ActorMotionError,ActorFootstep,MoveAcknowledgement,ActorIdleContext,ActorCallbackRng,ActorCallbackDraw};
use crate::creature_world::{self,CommonSpeedInput};
use crate::physics_tables::PhysicsTables;
use crate::actor_resources::ActorAnimationDatabase;

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ActorMovingReceipt {
    pub actor_id:u16,
    pub changed_cell:bool,
    pub acknowledgement:Option<MoveAcknowledgement>,
    pub footstep:Option<ActorFootstep>,
    pub created_effect:Option<u16>,
    pub water_effect:Option<u16>,
    pub pending_events:Vec<LandingEvent>,
    pub factory_draw:Option<landing::LandingDraw>,
    pub queue_requested:bool,
    pub rng_before:BlueprintRng,
    pub rng_after:BlueprintRng,
    pub arrived:bool,
    pub idle_draws:Vec<ActorCallbackDraw>,
}

impl ActorWorld {
    ///58780 player-list order, including57A10's no-op unready gate. A later
    /// rejected callback rolls back all earlier callbacks in this family pass.
    /// `player_count` is explicit runtime11878C; supplied resources are typed.
    pub fn step_actor_callbacks_in_player_order<B:AsRef<[u8]>>(&mut self,player_count:u8,frame:u32,context:LandingPhysicsContext,
        physics:&PhysicsTables,trig:&TrigTables,resources:&[ActorAnimationDatabase<B>],runtime_mode:u16,override_180d80:u8)->Result<Vec<ActorMovingReceipt>,ActorWorldError> {
        if player_count>4 {return Err(ActorWorldError::InvalidPlayer(player_count))}
        let mut staged=self.clone();let mut receipts=Vec::new();
        for owner in 0..player_count {
            let mut index=0usize;
            while let Some(id)=actor_motion::next_player_actor_id(staged.player(owner)?.bytes(),index)? {
                index+=1;
                if !(0..20).contains(&id){return Err(ActorMotionError::UnsupportedState.into())}
                let a=staged.actors.record(id as usize)?;
                if a[6]!=3||a[24]!=owner{return Err(ActorWorldError::ActorOwner(id))}
                if a[0x70]&1==0 {continue}
                let key=staged.actors.bindings(id as usize)?.animation_database.ok_or(ActorWorldError::MissingBlueprintBinding(id))?;
                let resource=resources.iter().find(|r|r.key()==key).ok_or(ActorWorldError::MissingBlueprintBinding(id))?;
                let idle=ActorIdleContext{runtime_mode,override_180d80,runtime_count_11c2bc:word(&staged.backing,0x355c)as u16,player_mode:staged.player(owner)?.bytes()[2]};
                receipts.push(staged.step_actor_callback_with_idle(id as u16,frame,context,physics,trig,resource,idle)?);
            }
        }
        *self=staged;Ok(receipts)
    }
    /// Source57A10 through ordinary state8, with actual6FB60 straight-motion
    /// queries, shared6DF30 physics, blueprint/animation updates and unmarked
    /// code6 footsteps. Resource bytes are hash-admitted by their typed loader.
    /// Moving-only entry: arrival requires the explicit idle context API.
    pub fn step_actor_moving_callback<B:AsRef<[u8]>>(&mut self,id:u16,frame:u32,context:LandingPhysicsContext,
        physics:&PhysicsTables,trig:&TrigTables,resource:&ActorAnimationDatabase<B>)->Result<ActorMovingReceipt,ActorWorldError> {
        self.step_actor_callback_inner(id,frame,context,physics,trig,resource,None)
    }
    /// Also admits ordinary arrival, with the source runtime idle globals
    /// explicitly supplied. Player mode and mission count must match storage.
    pub fn step_actor_callback_with_idle<B:AsRef<[u8]>>(&mut self,id:u16,frame:u32,context:LandingPhysicsContext,
        physics:&PhysicsTables,trig:&TrigTables,resource:&ActorAnimationDatabase<B>,idle:ActorIdleContext)->Result<ActorMovingReceipt,ActorWorldError> {
        self.step_actor_callback_inner(id,frame,context,physics,trig,resource,Some(idle))
    }
    fn step_actor_callback_inner<B:AsRef<[u8]>>(&mut self,id:u16,frame:u32,context:LandingPhysicsContext,
        physics:&PhysicsTables,trig:&TrigTables,resource:&ActorAnimationDatabase<B>,idle:Option<ActorIdleContext>)->Result<ActorMovingReceipt,ActorWorldError> {
        self.validate_links()?;
        let before=*self.actors.record(usize::from(id))?;
        if before[6]!=3||before[0x70]&1==0 {return Err(ActorMotionError::Inactive.into())}
        if before[0x16]&0x20!=0 {return Err(ActorMotionError::UnsupportedState.into())}
        let binding=self.actors.bindings(usize::from(id))?;
        let handle=binding.blueprint.ok_or(ActorWorldError::MissingBlueprintBinding(id as i16))?;
        if binding.animation_database!=Some(resource.key())||handle.player_id!=before[24] {return Err(ActorWorldError::MissingBlueprintBinding(id as i16))}
        let blueprint=*self.player(handle.player_id)?.blueprint(usize::from(handle.blueprint_index))?;
        let class3_unpowered=self.player(handle.player_id)?.bytes()[0x610]==0;
        let target=[word(&before,11),word(&before,13)];let raw=[id as u8,(id>>8)as u8,3];
        let counter=frame.wrapping_add(i32::from(word(&before,15)) as u32);
        let idle_suppressed=if before[0x49]&0x10==0&&counter&31==0 {
            (blueprint[0x1a] as i8)<=0||self.actor_route_point_accepted(raw,[word(&before,7),word(&before,9)])?
        }else{false};
        let(prefix,working_target)=self.actor_route_prefix(raw,&before,counter,physics,trig,idle_suppressed)?;
        let(motion,water_request)=creature_world::plan_common_nonflight_physics_with_water_tail(&prefix,self.map(),frame,context.gravity,context.water_height,physics,trig,
            |r,normal,previous|self.actor_contact_speed(r,normal,previous).map_err(|_|CreatureError::MotionEnvironment),
            |point|self.move_obstacle_at(point).map_err(|_|CreatureError::MotionEnvironment),true)?;
        let mut moved=motion.after;
        if moved[0x49]&3!=0&&creature_world::common_distance_squared(self.map(),[word(&moved,7),word(&moved,9)],working_target)? as u32<=4096{moved[0x49]|=8;}

        //6FB60's ordinary arrival tail invokes6E9E0 and then57A10's idle AI.
        // Do not silently skip that callback or its shared RNG draws.
        let arrived=prefix[0x49]&3==0&&(word(&prefix,0x3f)>=1||prefix[0x49]&0x10!=0)&&word(&moved,0x3d)<=256
            &&i32::from_le_bytes(moved[0x27..0x2b].try_into().unwrap())<=i32::from_le_bytes(moved[0x11..0x15].try_into().unwrap())
            &&creature_world::common_distance_squared(self.map(),[word(&moved,7),word(&moved,9)],target)? as u32<=1024;
        let mut staged=self.clone();let old_rng=staged.rng;
        staged.commit_actor_motion_record(id,&before,&moved)?;
        let mut water_effect=None;let mut pending_events=Vec::new();
        if let Some(request)=water_request{
            if request.code!=0x30{return Err(CreatureError::MotionEnvironment.into());}
            if let Some((slot,events))=staged.create_water_splash(request.position,request.physical_height)?{water_effect=Some(slot);pending_events.extend(events.into_iter().filter(|e|matches!(e,LandingEvent::PlayAudio(_))));}
        }

        let moved=*staged.actors.record(usize::from(id))?;
        let (control_after,blueprint_after,acknowledgement,idle_draws)=if arrived {
            let idle=idle.ok_or(ActorMotionError::UnsupportedState)?;
            if idle.player_mode!=staged.player(handle.player_id)?.bytes()[2]||idle.runtime_count_11c2bc!=word(&staged.backing,0x355c)as u16 {return Err(ActorMotionError::UnsupportedState.into())}
            let ground=genewars_core::cd_candidate::interpolate_height(staged.map(),target[0],target[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
            let shade=genewars_core::cd_candidate::interpolate_entity_shade(staged.map(),target[0],target[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
            let snap=actor_motion::plan_actor_arrival_snap(&moved,target,ground,shade)?;
            staged.commit_actor_motion_record(id,&moved,&snap)?;
            let snap=*staged.actors.record(usize::from(id))?;
            let rng=ActorCallbackRng{seed:staged.rng.seed,ring:staged.rng.ring,draw_count:staged.rng.draw_count};
            let plan=match actor_motion::plan_actor_arrival_idle_with_empty_scorer(&snap,&blueprint,frame,rng,idle,false,class3_unpowered) {
                Err(ActorMotionError::IdleNeighborhoodRequired) if staged.actor_idle_scorer_empty(id)?=>actor_motion::plan_actor_arrival_idle_with_empty_scorer(&snap,&blueprint,frame,rng,idle,true,class3_unpowered)?,
                result=>result?,
            };
            staged.rng=BlueprintRng{seed:plan.rng_after.seed,ring:plan.rng_after.ring,draw_count:plan.rng_after.draw_count};
            (plan.after,plan.blueprint_after,None,plan.draws)
        }else if moved[0x55]==9&&moved[0x7e]==255&&moved[0x82]==0 {
            let idle=idle.ok_or(ActorMotionError::UnsupportedState)?;
            if idle.player_mode!=staged.player(handle.player_id)?.bytes()[2]||idle.runtime_count_11c2bc!=word(&staged.backing,0x355c)as u16{return Err(ActorMotionError::UnsupportedState.into());}
            let rng=ActorCallbackRng{seed:staged.rng.seed,ring:staged.rng.ring,draw_count:staged.rng.draw_count};
            let plan=match crate::actor_construction_recovery::plan_cancelled_architect_idle(&moved,&blueprint,frame,rng,idle,false){
                Err(ActorMotionError::IdleNeighborhoodRequired)if staged.actor_idle_scorer_empty(id)?=>crate::actor_construction_recovery::plan_cancelled_architect_idle(&moved,&blueprint,frame,rng,idle,true)?,
                result=>result?,
            };
            staged.rng=BlueprintRng{seed:plan.rng_after.seed,ring:plan.rng_after.ring,draw_count:plan.rng_after.draw_count};
            (plan.after,plan.blueprint_after,None,plan.draws)
        }else if moved[0x55]==1&&moved[0x57]&2==0 {
            let idle=idle.ok_or(ActorMotionError::UnsupportedState)?;
            if idle.player_mode!=staged.player(handle.player_id)?.bytes()[2]||idle.runtime_count_11c2bc!=word(&staged.backing,0x355c)as u16 {return Err(ActorMotionError::UnsupportedState.into())}
            let rng=ActorCallbackRng{seed:staged.rng.seed,ring:staged.rng.ring,draw_count:staged.rng.draw_count};
            let plan=match actor_motion::plan_actor_idle_with_empty_scorer(&moved,&blueprint,frame,rng,idle,false,class3_unpowered) {
                Err(ActorMotionError::IdleNeighborhoodRequired) if staged.actor_idle_scorer_empty(id)?=>actor_motion::plan_actor_idle_with_empty_scorer(&moved,&blueprint,frame,rng,idle,true,class3_unpowered)?,
                result=>result?,
            };
            staged.rng=BlueprintRng{seed:plan.rng_after.seed,ring:plan.rng_after.ring,draw_count:plan.rng_after.draw_count};
            (plan.after,plan.blueprint_after,None,plan.draws)
        }else{
            let control=actor_motion::plan_actor_post_motion_prefix_with_class3(&moved,&blueprint,frame,&[960,576,1088,832],physics.angles(),class3_unpowered)?;
            (control.after,control.blueprint_after,control.acknowledgement,Vec::new())
        };
        let yaw=word(&control_after,0x39);let animation=if control_after[0x55]==1 {i32::from(control_after[0x19])}else{((((i32::from(yaw)>>7)+1)>>1)+1)&7};
        let count=resource.frame_count(animation as usize).map_err(|_|ActorMotionError::InvalidAnimation)?;
        let holds=(0..count).map(|i|resource.frame(animation as usize,i).map(|f|f.byte_10)).collect::<Result<Vec<_>,_>>().map_err(|_|ActorMotionError::InvalidAnimation)?;
        let flags=(resource.animation(animation as usize).map_err(|_|ActorMotionError::InvalidAnimation)?.trailing_word>>8)as u8;
        let after=if control_after[0x55]==1 {actor_motion::advance_actor_animation(&control_after,&holds,flags)?}else{actor_motion::plan_actor_moving_animation(&control_after,&holds,flags)?};
        let actual_before=*staged.actors.record(usize::from(id))?;
        staged.actors.replace_record_checked(usize::from(id),&actual_before,&after)?;
        staged.players[usize::from(handle.player_id)].commit_activation_blueprint(usize::from(handle.blueprint_index),&blueprint,&blueprint_after)?;
        let mut receipt=ActorMovingReceipt{actor_id:id,changed_cell:floor_cell([word(&before,7),word(&before,9)])!=floor_cell([word(&after,7),word(&after,9)]),acknowledgement,footstep:None,created_effect:None,water_effect,pending_events,factory_draw:None,queue_requested:true,rng_before:old_rng,rng_after:old_rng,arrived,idle_draws};
        let current=*staged.actors.record(usize::from(id))?;
        if let Some(point)=actor_motion::actor_footstep_query_position(&current,frame)? {
            let class=staged.terrain_class_at(point)?;
            if let Some(footstep)=actor_motion::plan_actor_footstep(&current,frame,class)? {
                if let Some((slot,draw))=staged.create_unmarked_smoke(footstep.position,footstep.physical_height,footstep.angles,context,trig)? {receipt.created_effect=Some(slot);receipt.factory_draw=Some(draw);}
                receipt.footstep=Some(footstep);
            }
        }
        staged.validate_links()?;if staged.effects.is_some(){staged.validate_effect_count()?;}
        receipt.rng_after=staged.rng;*self=staged;Ok(receipt)
    }

    pub(super) fn commit_actor_motion_record(&mut self,id:u16,before:&[u8;200],after:&[u8;200])->Result<(),ActorWorldError> {
        let changed=floor_cell([word(before,7),word(before,9)])!=floor_cell([word(after,7),word(after,9)]);
        if changed {self.unlink_actor_for_motion(id)?;}
        self.actors.replace_record_checked(usize::from(id),before,after)?;
        if changed {self.insert_actor_motion_head(id)?;}Ok(())
    }

    pub(super) fn actor_contact_speed(&self,r:&[u8;200],normal:[i32;3],previous:i16)->Result<i16,ActorWorldError> {
        let point=[word(r,7),word(r,9)];let target=actor_motion::actor_contact_distance_target(r)?;
        let sample=|p:[i16;2]|genewars_core::cd_candidate::interpolate_height(self.map(),p[0],p[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()));
        let distance=actor_motion::actor_contact_distance_squared(r,sample(point)?,sample(target)?)?;
        Ok(creature_world::plan_common_contact_speed(CommonSpeedInput{speed:word(r,0x41),desired_speed:word(r,0x3f),width:word(r,0x2b)as u16,
            flags_4a:r[0x4a],free_flight:false,ground_height:i32::from_le_bytes(r[0x11..0x15].try_into().unwrap()),previous_height_low:previous,surface_term:normal[2],distance_squared:distance})?)
    }

    /// 8D7B0(radius512, random-axis order) visits the same five-by-five cells
    /// in either direction. 58A70 only scores enemy/unallied kind1 whose
    /// virtual68 is false; all other linked families are skipped. Prove no
    /// qualifying record without assuming an empty neighborhood.
    ///Read-only6FB60 route and steering candidate before the shared physics call.
    pub fn actor_motion_route_candidate(&self,id:u16,frame:u32,physics:&PhysicsTables,trig:&TrigTables,idle_event_search_suppressed:bool)->Result<([u8;200],[i16;2]),ActorWorldError>{
        let before=self.actors.record(id as usize)?;if before[6]!=3{return Err(ActorMotionError::Inactive.into());}
        self.actor_route_prefix([id as u8,(id>>8)as u8,3],before,frame.wrapping_add(i32::from(word(before,15))as u32),physics,trig,idle_event_search_suppressed)
    }
    ///Successful null-obstacle6FB60 boundary routing. Exhausted searches and
    ///the separate retry/backoff branch remain explicit errors.
    ///74D90 uses mask1 and exempts only its already-selected nominated building.
    pub(super) fn actor_route_point_accepted(&self,raw:[u8;3],point:[i16;2])->Result<bool,ActorWorldError>{
        let actor=self.entity(raw)?;if raw[2]!=3{return Err(CreatureError::UnsupportedMotionState.into());}
        let nominated=if actor[0x4f]==4{let reference=reference(actor,0x4d);self.entity(reference)?;Some(word(&reference,0)as u16)}else if actor[0x4f]==0{None}else{return Err(CreatureError::UnsupportedMotionState.into());};
        if self.move_obstacle_at_excluding(point,nominated)?{return Ok(false);}
        let environment=self.placement_environment(word(&raw,0),point)?;
        Ok(actor[0x53]&(1<<environment.terrain_class)==0&&environment.blocker_weight<10)
    }
    fn actor_route_ray(&self,raw:[u8;3],from:[i16;2],to:[i16;2],physics:&PhysicsTables,trig:&TrigTables)->Result<bool,ActorWorldError>{
        let ray=actor_motion::actor_straight_ray::<CreatureError,_>(from,to,physics.angles(),trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap(),|point|self.actor_route_point_accepted(raw,point).map_err(|_|CreatureError::MotionEnvironment))?;
        Ok(ray.clear)
    }
    pub(super) fn actor_route_prefix(&self,raw:[u8;3],before:&[u8;200],counter:u32,physics:&PhysicsTables,trig:&TrigTables,idle_suppressed:bool)->Result<([u8;200],[i16;2]),ActorWorldError>{
        if before[0x16]&0x48!=0||before[0x49]&3==3||before[0x4a]&4!=0||!matches!(before[0x4f],0|4)||before[0x52]!=0{return Err(CreatureError::UnsupportedMotionState.into());}
        let current=[word(before,7),word(before,9)];let mut a=*before;a[0x49]&=0x9f;
        if a[0x49]&0x10==0&&counter&31==0&&!idle_suppressed {
            // 6FB60 -> 305F0/6D230: deterministic eight-ring search. Unlike
            // the current-point gate, candidate74D90 has no nominated building.
            let id=word(&raw,0);
            let found=creature_world::find_common_idle_relocation(current,|point| {
                let accepts=||->Result<bool,ActorWorldError>{
                    if self.move_obstacle_at_excluding(point,None)? {return Ok(false);}
                    let environment=self.placement_environment(id,point)?;
                    Ok(before[0x53]&(1<<environment.terrain_class)==0&&environment.blocker_weight<10)
                };
                accepts().map_err(|_|CreatureError::MotionEnvironment)
            })?;
            if let Some(requested)=found {
                // Virtual52280 resolves through74A50 using mask0 obstacles.
                let resolved=crate::target_resolver::resolve_target(current,requested,physics,trig,|point| {
                    let obstacle=self.move_obstacle_at(point)?;
                    let class=self.terrain_class_at(point)?;
                    if before[0x53]&(1<<class)!=0||obstacle {return Ok(false);}
                    Ok(self.cell_plant_blocker(id,point)?<10)
                }).map_err(|e|match e {
                    crate::target_resolver::TargetResolverError::Query(error)=>error,
                    crate::target_resolver::TargetResolverError::IterationLimit=>ActorWorldError::TargetSearchLimit,
                })?;
                a[0x5b]=0;
                for offset in [0x0b,0x5c] {
                    a[offset..offset+2].copy_from_slice(&resolved.point[0].to_le_bytes());
                    a[offset+2..offset+4].copy_from_slice(&resolved.point[1].to_le_bytes());
                }
                a[0x57]|=2;
                // The caller sets bit0 around virtual52280, then clears it.
                a[0x49]&=!1;
            }
        }
        let mut target=[word(&a,11),word(&a,13)];
        let mut working=if a[0x49]&3!=0{[word(&a,0x45),word(&a,0x47)]}else{target};let mut immediate=false;
        if a[0x49]&0x10!=0&&a[0x49]&12!=0{
            if !self.actor_route_point_accepted(raw,target)?{
                // Existing waypoint/current-point retry variants remain bounded.
                if a[0x49]&3!=0{return Err(CreatureError::TargetSearchRequired.into());}
                //6FEC0..6FED3: repair the working destination with74A50's
                //mask1/nominated-building predicate. This is not52280: only+B
                //changes; pending target5C, flags57 and mode5B remain intact.
                let resolved=crate::target_resolver::resolve_target(current,target,physics,trig,
                    |point|self.actor_route_point_accepted(raw,point)).map_err(|e|match e{
                        crate::target_resolver::TargetResolverError::Query(error)=>error,
                        crate::target_resolver::TargetResolverError::IterationLimit=>ActorWorldError::TargetSearchLimit,
                    })?;
                target=resolved.point;
                a[11..13].copy_from_slice(&target[0].to_le_bytes());a[13..15].copy_from_slice(&target[1].to_le_bytes());
            }
            let from=if a[0x49]&3!=0{working}else{current};
            let ray=crate::actor_motion::actor_straight_ray::<CreatureError,_>(from,target,physics.angles(),trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap(),|p|self.actor_route_point_accepted(raw,p).map_err(|_|CreatureError::MotionEnvironment))?;
            if ray.clear{if a[0x49]&3!=0{let f=word(&a,0x49)as u16&0xeffc;a[0x49..0x4b].copy_from_slice(&f.to_le_bytes());a[0x16]&=0x7f;immediate=true;}working=target;}
            else{
                let walk=|start,side,limit|creature_world::plan_common_waypoint(current,start,target,side,limit,physics,|p|self.actor_route_point_accepted(raw,p).map_err(|_|CreatureError::MotionEnvironment),|x,y|self.actor_route_ray(raw,x,y,physics,trig).map_err(|_|CreatureError::MotionEnvironment));
                if a[0x49]&3!=0{if a[0x49]&8==0{return Err(CreatureError::UnsupportedMotionState.into());}let r=walk(from,a[0x49]&1!=0,120)?;if r.cost>=120{return Err(CreatureError::MotionRayBlocked.into());}working=r.point.ok_or(CreatureError::MotionEnvironment)?;}
                else{let start=ray.failure_point.ok_or(CreatureError::MotionEnvironment)?;let left=walk(start,false,120)?;let right=walk(start,true,if left.cost<120{left.cost+1}else{120})?;let(r,flag)=if left.cost<right.cost{(left,0x1002u16)}else{(right,0x1001u16)};if r.cost>=120{return Err(CreatureError::MotionRayBlocked.into());}working=r.point.ok_or(CreatureError::MotionEnvironment)?;let f=(word(&a,0x49)as u16&0xeffc)|flag;a[0x49..0x4b].copy_from_slice(&f.to_le_bytes());a[0x16]&=0x7f;a[0x54]=20;}
                a[0x45..0x47].copy_from_slice(&working[0].to_le_bytes());a[0x47..0x49].copy_from_slice(&working[1].to_le_bytes());immediate=true;
            }a[0x49]&=0xf3;
        }
        if a[0x49]&0x10!=0&&(immediate||counter&1!=0){let yaw=crate::actor_motion::angle(i32::from(working[0].wrapping_sub(current[0]).wrapping_mul(2)),i32::from(working[1].wrapping_sub(current[1]).wrapping_mul(2)),physics.angles());a[0x4b..0x4d].copy_from_slice(&yaw.to_le_bytes());}
        Ok((creature_world::plan_common_ground_steering(&a,trig)?,working))
    }
    pub(super) fn actor_idle_scorer_empty(&self,id:u16)->Result<bool,ActorWorldError> {
        let actor=self.actors.record(usize::from(id))?;let owner=actor[24];let alliances=self.player(owner)?.bytes()[0x6e5];
        let center=floor_cell([word(actor,7),word(actor,9)]);let cx=(center%128)as i32;let cy=(center/128)as i32;
        for dy in -2..=2 {for dx in -2..=2 {
            let cell=((cy+dy)&127)as usize*128+((cx+dx)&127)as usize;let mut raw=reference(self.map(),cell*14+6);let mut seen=HashSet::new();
            while raw[2]!=0 {
                if !seen.insert(raw){return Err(ActorWorldError::DuplicateLink(raw))}
                let entity=self.entity(raw)?;
                if raw[2]==1 {
                    let creature=self.creatures.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?.record(word(&raw,0)as usize)?;
                    let enemy=owner!=creature[24]&&(owner==4||(u32::from(alliances)&1u32.wrapping_shl(u32::from(creature[24])&31))==0);
                    if enemy&&creature[0x5e]<24 {return Ok(false)}
                }
                raw=reference(entity,3);
            }
        }}Ok(true)
    }

    fn unlink_actor_for_motion(&mut self,id:u16)->Result<(),ActorWorldError> {
        let raw=[id as u8,(id>>8)as u8,3];let r=self.entity(raw)?;let previous=reference(r,0);let next=reference(r,3);let cell=floor_cell([word(r,7),word(r,9)]);
        if previous[2]==0 {
            if word(&previous,0)as usize!=cell||reference(self.map(),cell*14+6)!=raw{return Err(ActorWorldError::StaleLink)}
            if next[2]!=0 {self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&next);}else{self.backing[MAP_START+cell*14+8]=0;}
        }else{if reference(self.entity(previous)?,3)!=raw{return Err(ActorWorldError::StaleLink)}self.replace_next(previous,next)?;}
        if next[2]!=0 {if reference(self.entity(next)?,0)!=raw{return Err(ActorWorldError::StaleLink)}self.replace_previous(next,previous)?;}Ok(())
    }
    fn insert_actor_motion_head(&mut self,id:u16)->Result<(),ActorWorldError> {
        let raw=[id as u8,(id>>8)as u8,3];let before=*self.actors.record(usize::from(id))?;let mut after=before;let cell=floor_cell([word(&before,7),word(&before,9)]);let head=reference(self.map(),cell*14+6);
        after[..2].copy_from_slice(&(cell as u16).to_le_bytes());after[2]=0;
        if head[2]!=0 {self.entity(head)?;after[3..6].copy_from_slice(&head);self.replace_previous(head,raw)?;}else{after[5]=0;}
        self.actors.replace_record_checked(usize::from(id),&before,&after)?;self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&raw);Ok(())
    }
}
