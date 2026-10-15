//! Independently guarded owned Mule use of shared callback mathematics.
//! The world adapter supplies source-backed reveal, queries and owned decisions.
use crate::creature_world::{self,CreatureRecord,CreatureError,CommonPhysicsPlan,CommonWaterEffect,MuleStepContext,MuleAnimations};
use crate::physics_tables::PhysicsTables;
use crate::actor_activation::TrigTables;
pub fn plan_owned_after_prefix<F,O,C,E,P>(before:&CreatureRecord,prefix:&CreatureRecord,working_target:[i16;2],map:&[u8],context:MuleStepContext,
 physics:&PhysicsTables,trig:&TrigTables,contact:F,obstacle:O,animations:&MuleAnimations,choice:C,cell_entry:E,postphysics:P)
 ->Result<(CommonPhysicsPlan<124>,Option<CommonWaterEffect>),CreatureError>
where F:FnMut(&CreatureRecord,[i32;3],i16)->Result<i16,CreatureError>,O:FnMut(&CreatureRecord)->Result<CreatureRecord,CreatureError>,
 C:FnMut(&CreatureRecord)->Result<CreatureRecord,CreatureError>,E:FnMut(&CreatureRecord,usize,usize)->Result<CreatureRecord,CreatureError>,P:FnMut(&CreatureRecord,Option<CommonWaterEffect>)->Result<CreatureRecord,CreatureError>{
 let w=|o|i16::from_le_bytes([before[o],before[o+1]]);
 let idle=before[0x5e]==8&&before[0x61]==0&&matches!(before[0x60],0|8|0x88)
  && (before[0x78]==0 || (before[0x78]==7&&matches!(before[0x60],8|0x88)));
 let command=matches!(before[0x5e],0|8)&&before[0x61]==0x80&&matches!(before[0x60],0x18|0x98)&&before[0x78]==7;
 let hostile=before[24]==0&&matches!(before[0x5e],1|21)&&before[0x61]==0&&(if before[0x5e]==1{matches!(before[0x60],0x48|0xc8)}else{matches!(before[0x60],8|0x88)})&&before[0x78]==7
  &&before[0x52]==1&&before[0x57]==1&&before[0x5a]==1&&before[0x5d]==0
  &&before[0x50..0x53]==before[0x55..0x58]&&before[0x58..0x5b]==before[0x55..0x58];
 if before[6]!=1||before[24]>=4||before[0x7a]!=4||!(idle||command||hostile)||before[0x76]>90||before[0x7b]!=255
  ||before[0x15]!=0x80||!matches!(before[0x16],2|0x82)||before[0x4f]!=0||(before[0x52]!=0&&!hostile)||(before[0x57]!=0&&!hostile)
  ||w(0x49)as u16&!0x337f!=0||w(0x6c)<0||w(0x66)<=2||w(0x6a)<=0{return Err(CreatureError::UnsupportedMotionState);}
 creature_world::plan_mule_admitted_body_with_collision(before,prefix,working_target,map,context,physics,trig,contact,animations,choice,true,cell_entry,postphysics,obstacle,hostile)
}
