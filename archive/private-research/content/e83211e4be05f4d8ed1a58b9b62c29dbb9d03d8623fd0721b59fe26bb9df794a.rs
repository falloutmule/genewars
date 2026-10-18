//! Transactional diagnostic history; it never supplies gameplay inputs.
use std::sync::Arc;
use genewars_runtime::actor_world::{ActorDispatchObservation,ActorDispatchSnapshot,ActorObservedBranch};

#[derive(Clone,Debug)]
pub struct ActivationObservation {pub frame:u32,pub actor_id:u16,pub snapshot:ActorDispatchSnapshot}
#[derive(Default,Clone)]
pub struct ActorDispatchAudit {
 head:Option<Arc<AuditFrame>>,dispatches:u64,mismatches:u64,unknown:u64,dependent:u64,
 actor_counts:[u64;20],activation_events:Vec<ActivationObservation>,recorded_dispatches:u64,
}
struct AuditFrame {
    previous:Option<Arc<AuditFrame>>,
    dispatches:Vec<ActorDispatchObservation>,
}
// Long missions must not recursively drop thousands of historical frames on
// the Windows main-thread stack. Shared snapshots stop the iterative release.
impl Drop for AuditFrame {
    fn drop(&mut self){
        let mut previous=self.previous.take();
        while let Some(frame)=previous {
            match Arc::try_unwrap(frame){Ok(mut frame)=>previous=frame.previous.take(),Err(_)=>break}
        }
    }
}
fn snapshot_json(s:&ActorDispatchSnapshot)->String {
    format!("{{\"state\":{},\"restore\":{},\"category\":{},\"style\":{},\"task\":{},\"task_kind\":{},\"selector\":{},\"blueprint18\":{},\"blueprint1b\":{}}}",s.state,s.restore_state,s.category,s.style,s.task,s.task_kind,s.selector,s.blueprint_prior,s.blueprint_current)
}
fn dispatch_address(branch:&ActorObservedBranch)->u32 {
    match branch{ActorObservedBranch::Ordinary{source_dispatch,..}|ActorObservedBranch::Architect{source_dispatch,..}=>*source_dispatch}
}
impl ActorDispatchAudit {
    pub fn append(&mut self,dispatches:Vec<ActorDispatchObservation>,activations:Vec<ActivationObservation>){
        self.append_recording(dispatches,activations,std::env::var("GENEWARS_ACTOR_DISPATCH_TRACE").as_deref()==Ok("1"));
    }
    fn append_recording(&mut self,dispatches:Vec<ActorDispatchObservation>,activations:Vec<ActivationObservation>,record:bool){
        for a in &activations{self.mismatches+=u64::from(!a.snapshot.blueprint_equal());}
        self.activation_events.extend(activations);
        for d in &dispatches{
            self.dispatches+=1;self.actor_counts[usize::from(d.actor_id)]+=1;
            self.mismatches+=u64::from(!d.before.blueprint_equal())+u64::from(!d.after.blueprint_equal());
            let address=dispatch_address(&d.branch);self.unknown+=u64::from(address==0);self.dependent+=u64::from(address==0x56530);
        }
        if record&&!dispatches.is_empty(){
            self.recorded_dispatches+=dispatches.len()as u64;
            self.head=Some(Arc::new(AuditFrame{previous:self.head.clone(),dispatches}));
        }
    }
    pub fn report(&self,include_trace:bool)->String {
        let trace_complete=self.recorded_dispatches==self.dispatches;
        let include_trace=include_trace&&trace_complete&&self.recorded_dispatches>0;
        let mut frames=Vec::new();
        if include_trace{let mut current=self.head.as_deref();while let Some(frame)=current{frames.push(frame);current=frame.previous.as_deref();}}
        let activations=self.activation_events.len();let dispatches=self.dispatches;let mismatches=self.mismatches;let unknown=self.unknown;let dependent=self.dependent;let actor_counts=self.actor_counts;
        let activation_events=self.activation_events.iter().map(|a|format!("{{\"frame\":{},\"actor\":{},\"phase\":\"end_of_frame_after_effects\",\"snapshot\":{}}}",a.frame,a.actor_id,snapshot_json(&a.snapshot))).collect::<Vec<_>>();
        let mut events=Vec::new();
        for frame in frames.into_iter().rev(){
            for d in &frame.dispatches{
                if include_trace{
                    let branch=match &d.branch{
                        ActorObservedBranch::Ordinary{source_dispatch,idle_scorer_empty}=>format!("{{\"kind\":\"ordinary\",\"source_dispatch\":{source_dispatch},\"idle_scorer_empty\":{}}}",idle_scorer_empty.map_or("null".into(),|v|v.to_string())),
                        ActorObservedBranch::Architect{source_dispatch,setup_after_state,rejected_site}=>format!("{{\"kind\":\"architect\",\"source_dispatch\":{source_dispatch},\"setup_after_state\":{},\"rejected_site\":{rejected_site}}}",setup_after_state.map_or("null".into(),|v|v.to_string())),
                    };
                    events.push(format!("{{\"frame\":{},\"actor\":{},\"before\":{},\"after\":{},\"branch\":{branch}}}",d.frame,d.actor_id,snapshot_json(&d.before),snapshot_json(&d.after)));
                }
            }
        }
        format!("{{\"schema\":\"actor-dispatch-priors-v1\",\"coverage\":\"committed activated-actor callbacks\",\"activations\":{activations},\"dispatches\":{dispatches},\"actor_dispatch_counts\":{actor_counts:?},\"blueprint_equality_mismatches\":{mismatches},\"unknown_dispatches\":{unknown},\"dependent_56530_dispatches\":{dependent},\"trace_included\":{include_trace},\"trace_complete\":{trace_complete},\"recorded_dispatches\":{},\"activation_events\":[{}],\"events\":[{}]}}",self.recorded_dispatches,activation_events.join(","),events.join(","))
    }
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn cloned_audit_is_transactional_and_long_history_drops_iteratively(){
        let s=ActorDispatchSnapshot{state:1,restore_state:0,category:2,style:0,task:255,task_kind:0,selector:0,blueprint_prior:165,blueprint_current:165};
        let event=ActorDispatchObservation{frame:0,actor_id:1,before:s.clone(),after:s,branch:ActorObservedBranch::Ordinary{source_dispatch:0x52780,idle_scorer_empty:Some(true)}};
        let mut audit=ActorDispatchAudit::default();audit.append_recording(vec![event.clone()],Vec::new(),true);let saved=audit.report(true);
        let mut failed=audit.clone();failed.append_recording(vec![event.clone()],Vec::new(),true);drop(failed);assert_eq!(audit.report(true),saved);
        let mut unknown=event.clone();unknown.branch=ActorObservedBranch::Architect{source_dispatch:0,setup_after_state:None,rejected_site:false};
        let mut probe=audit.clone();probe.append_recording(vec![unknown],Vec::new(),false);assert!(probe.report(false).contains("\"unknown_dispatches\":1"));
        assert!(probe.report(true).contains("\"trace_included\":false"));
        assert!(probe.report(true).contains("\"trace_complete\":false"));
        let mut summary=ActorDispatchAudit::default();
        summary.append_recording(vec![event.clone()],vec![ActivationObservation{frame:0,actor_id:1,snapshot:event.before.clone()}],false);
        assert!(summary.head.is_none());assert_eq!(summary.dispatches,1);
        let saved_summary=summary.report(true);let mut discarded=summary.clone();
        discarded.append_recording(vec![event.clone()],Vec::new(),true);drop(discarded);
        assert_eq!(summary.report(true),saved_summary);
        summary.append_recording(vec![event.clone()],Vec::new(),true);
        assert!(summary.report(true).contains("\"trace_included\":false"));
        assert!(summary.report(false).contains("end_of_frame_after_effects"));
        assert_eq!(summary.recorded_dispatches,1);assert_eq!(summary.dispatches,2);
        for _ in 0..12000{audit.append_recording(vec![event.clone()],Vec::new(),true);}drop(audit);drop(probe);
    }
}
