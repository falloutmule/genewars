//! Completed terraform transition including the actual526C0 ready/unready tail.
use crate::construction_terraform_completion::{self,TerraformCompletionPlan,TerraformCompletionError};
use crate::construction_motion::{self,ConstructionMotionError};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum TerraformResumeError{Completion(TerraformCompletionError),Movement(ConstructionMotionError)}
pub fn plan_architect_terraform_resume(before:&[u8;200],blueprint:&[u8;36],job:&[u8;176],seed:u16,desired_speed:i16,angles:&[u16;257])
    ->Result<TerraformCompletionPlan,TerraformResumeError>{
    let mut plan=construction_terraform_completion::plan_architect_completed_terraform_prefix(before,blueprint,job,seed,desired_speed).map_err(TerraformResumeError::Completion)?;
    plan.after=construction_motion::plan_architect_task_movement(&plan.after,blueprint,angles).map_err(TerraformResumeError::Movement)?.after;
    Ok(plan)
}
