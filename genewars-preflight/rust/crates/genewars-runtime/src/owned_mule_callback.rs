//! Independently guarded owned Mule use of shared callback mathematics.
//! The world adapter supplies source-backed reveal, queries and owned decisions.
use crate::creature_world::{self,CreatureRecord,CreatureError,CommonPhysicsPlan,CommonWaterEffect,MuleStepContext,MuleAnimations};
use crate::physics_tables::PhysicsTables;
use crate::actor_activation::TrigTables;
pub fn plan_owned_after_prefix<F,O,C,E,P>(before:&CreatureRecord,prefix:&CreatureRecord,working_target:[i16;2],map:&[u8],context:MuleStepContext,
 physics:&PhysicsTables,trig:&TrigTables,contact:F,obstacle:O,animations:&MuleAnimations,choice:C,cell_entry:E,postphysics:P)
 ->Result<(CommonPhysicsPlan<124>,Option<CommonWaterEffect>),CreatureError>
where F:FnMut(&CreatureRecord,[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError>,
 C:FnMut(&CreatureRecord)->Result<CreatureRecord,CreatureError>,E:FnMut(&CreatureRecord,usize,usize)->Result<CreatureRecord,CreatureError>,P:FnMut(&CreatureRecord)->Result<CreatureRecord,CreatureError>{
 let w=|o|i16::from_le_bytes([before[o],before[o+1]]);
 let idle=before[0x5e]==8&&before[0x61]==0&&matches!(before[0x60],0|8|0x88)&&before[0x78]==0;
 let command=matches!(before[0x5e],0|8)&&before[0x61]==0x80&&matches!(before[0x60],0x18|0x98)&&before[0x78]==7;
 if before[6]!=1||before[24]>=4||before[0x7a]!=4||!(idle||command)||before[0x76]>90||before[0x7b]!=255
  ||before[0x15]!=0x80||!matches!(before[0x16],2|0x82)||before[0x4f]!=0||before[0x52]!=0||before[0x57]!=0
  ||w(0x49)as u16&!0x337f!=0||w(0x6c)<0||w(0x66)<=2||w(0x6a)<=0{return Err(CreatureError::UnsupportedMotionState);}
 creature_world::plan_mule_admitted_body_with_postphysics(before,prefix,working_target,map,context,physics,trig,contact,obstacle,animations,choice,false,cell_entry,postphysics)
}
