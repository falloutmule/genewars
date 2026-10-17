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
 //5940 measured extension: the state8 attack-ready arm also admits the
 //expired 0x6c==0 timer; the disposition is the 624B8 reattack resume.
 let hostile=before[24]==0&&(matches!(before[0x5e],1|21)||before[0x5e]==8&&w(0x6c)>=0)&&before[0x61]==0&&(if before[0x5e]==1{matches!(before[0x60],0x48|0xc8)}else{matches!(before[0x60],8|0x88)})&&before[0x78]==7
  &&before[0x52]==1&&before[0x57]==1&&before[0x5a]==1&&before[0x5d]==0
  &&before[0x50..0x53]==before[0x55..0x58]&&before[0x58..0x5b]==before[0x55..0x58];
 let hit=before[24]==0&&before[0x5e]==20&&before[0x61]==0&&before[0x60]==8&&before[0x78]==7
  &&before[0x52]==1&&before[0x57]==1&&before[0x5a]==1&&before[0x5d]==0&&w(0x6c)>0
  &&before[0x50..0x53]==before[0x55..0x58]&&before[0x58..0x5b]==before[0x55..0x58];
 //5940 measured owned state-20 FRESH-timer continuation (live corrected-campaign
//record, fixture attempt17-live-5940-blocked-owned slot 1, 0x6c=32767): the
//literal source path 0005ed37/0005ed3b (0x61 bit0 clear -> light/head branch)
//-> 0005ee25-0005ee32 parity gate FULL path -> 0005eed5-0005ef1f cell pointers
//-> 0005ef22 call 0x5e0b0 -> 0005ef32 call 0x5eb30 -> 0005ef3a-0005ef40 fresh
//timer (the expired-timer arm is NOT taken) -> 0005ef49-0005ef68 +
//0006a20d-0006a21f state dispatch reaches the SAME generic tail
//0006a290-0006a53e (0006a4a0 DEC word(0x6c)) -> epilogue 0005e640/0005e817 ->
//0007ac00. Zero RNG draws: no 5EB30 dispatch site fires for this shape (0x49
//mask 0x0010, no 0x100/0x200 virtual, 0x6c>0), so the port runs only the
//shared even-branch/state dispatch, association handling and tail writes.
//Pins are the interpreter's measured admission shape (attempt18).
 let fresh_hit=before[24]==0&&before[0x5e]==20&&before[0x61]==0x80&&before[0x60]==0x18&&before[0x78]==7
  &&before[0x52]==0&&before[0x57]==1&&before[0x5a]==1&&before[0x5d]==0&&w(0x6c)>0
  &&before[0x58..0x5b]==before[0x55..0x58];
//5946 measured owned state-20 EXPIRED-timer completion (live corrected-campaign
//record, fixture attempt20-live-5946-blocked-owned slot 1, word(0x6c)=0 with
//0x50..0x53=05 00 00 != 0x55..0x58=05 00 01): the literal source path
//0005ef3a/0005ef3e/0005ef40 (JLE 0x65384 TAKEN when word(0x6c)==0) ->
//0x65384-0x65397 state dispatch via the jump table at 0x5ecf0 (entry 0x14 ->
//0x5f4c4) -> 0005f4c4-0005f4d3 (word(0x6a)>0) -> 0005f501-0005f521 ->
//0005f532-0005f569 (0005f564 CALL 0x81c60) -> 0005f5bc JNZ 0x6029a ->
//0006029a-000602b0 (000602a2 MOV byte[0x5e],0x8; 00060291 MOV word[0x6c],0) ->
//00060295 JMP 0x5ef46 -> 0005ef54 JBE 0x6539f -> 000653c3 JMP 0x6a290 -> the
//same generic tail 0006a290-0006a53e (0006a4a0 DEC word(0x6c) SKIPPED:
//0006a49b JLE; 0006a4a7 CALL 0x5e640 selects animation 0x19 0x10->0x05;
//0006a4d5 CALL 0x7ac00; 0006a4e2 CALL 0x83140). Zero RNG draws. The first
//triple 0x50..0x53 is consumed only by the admission predicate, never by the
//executed tail, so only the 0x58..0x5b == 0x55..0x58 match is pinned (the
//interpreter's measured admission arm, authoritative output 2a54ba4c...).
 let expired_hit=before[24]==0&&before[0x5e]==20&&before[0x61]==0x80&&before[0x60]==0x18&&before[0x78]==7
  &&before[0x52]==0&&before[0x57]==1&&before[0x5a]==1&&before[0x5d]==0&&w(0x6c)==0
  &&before[0x58..0x5b]==before[0x55..0x58];
//5922 measured owned hit-reaction completion: same record domain as the hit
 //hold with the expired 0x6c timer.5E0B0/5CC60 return to the explicit move
 //command shape (0x61=0x80,0x60=0x18), clear57 and restore animation. No RNG.
 let hit_complete=before[24]==0&&before[0x5e]==20&&before[0x61]==0&&before[0x60]==8&&before[0x78]==7
  &&before[0x52]==1&&before[0x57]==1&&before[0x5a]==1&&before[0x5d]==0&&w(0x6c)==0
  &&before[0x50..0x53]==before[0x55..0x58]&&before[0x58..0x5b]==before[0x55..0x58];
 //5924 measured retained command dispatch (state8, 0x61=0x80, 0x60=0x18):
 //the command domain carrying the retained pair association 0x52=1 with
 //cleared57. Dispatch is the ordinary AI choice machinery (5EB30/624B8),
 //same plan_choice/plan_explicit_target pieces as the accepted command path.
 let command_retained=before[0x5e]==8&&before[0x61]==0x80&&before[0x60]==0x18&&before[0x78]==7
  &&before[0x52]==1&&before[0x57]==0&&before[0x5a]==1;
 if before[6]!=1||before[24]>=4||before[0x7a]!=4||!(idle||command||hostile||hit||hit_complete||fresh_hit||expired_hit||command_retained)||before[0x76]>90||before[0x7b]!=255
  ||before[0x15]!=0x80||!matches!(before[0x16],2|0x82)||before[0x4f]!=0||(before[0x52]!=0&&!(hostile||hit||hit_complete||command_retained))||(before[0x57]!=0&&!(hostile||hit||hit_complete||fresh_hit||expired_hit))
  ||w(0x49)as u16&!0x337f!=0||w(0x6c)<0||w(0x66)<=2||w(0x6a)<=0{return Err(CreatureError::UnsupportedMotionState);}
 creature_world::plan_mule_admitted_body_with_collision(before,prefix,working_target,map,context,physics,trig,contact,animations,choice,true,cell_entry,postphysics,obstacle,hostile||hit_complete||expired_hit)
}
