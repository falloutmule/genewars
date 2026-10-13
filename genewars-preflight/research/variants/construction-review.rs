#[path = "../../rust/crates/genewars-core/src/construction.rs"] mod construction;
use construction::*;
use std::io::{self, Read};

// Independent register-style transcription: register arithmetic is u32;
// only the original JGE/JG signed conditions reinterpret its bits as i32.
fn machine(base: i32, skill: i8, p: i32, s: i32, refill: bool, ready: bool)
    -> (&'static str, Option<i32>, i32, i32, u16) {
    if !ready { return ("not_ready",None,p,s,3); }
    let eax = (i32::from(skill) as u32).wrapping_add(i32::from(skill) as u32);
    let ecx = (base as u32).wrapping_sub(eax);
    let sum = (s as u32).wrapping_add(p as u32);
    if (sum as i32) < (ecx as i32) { return ("insufficient",Some(ecx as i32),p,s,3); }
    let mut primary = p as u32;
    let mut secondary = s as u32;
    let mut edx = ecx.wrapping_sub(primary);
    if (edx as i32) > 0 {
        edx = edx.wrapping_sub(secondary);
        primary = 0;
        secondary = 0u32.wrapping_sub(edx);
    } else { primary = 0u32.wrapping_sub(edx); }
    if refill && (secondary.wrapping_add(primary) as i32) <= 1000 { secondary = 1000; }
    ("paid",Some(ecx as i32),primary as i32,secondary as i32,4)
}
fn result(selector: i8, skill: i8, p: i32, s: i32, refill: bool, ready: bool)
    -> (&'static str, Option<i32>, i32, i32, u16) {
    let v = construction_stage3_debit(ConstructionSelector::try_from(selector).unwrap(),skill,
        ConstructionPools { primary:p,secondary:s },refill,ready);
    let status = match v.status { ConstructionDebitStatus::NotReady=>"not_ready",
        ConstructionDebitStatus::Insufficient=>"insufficient", ConstructionDebitStatus::Paid=>"paid" };
    (status,v.cost,v.pools.primary,v.pools.secondary,v.stage)
}
fn main() {
    if std::env::args().any(|v|v=="--vectors") {
        let mut input=String::new();io::stdin().read_to_string(&mut input).unwrap();
        for line in input.lines() {
            let a:Vec<i64>=line.split(',').map(|v|v.parse().unwrap()).collect();
            let (status,cost,p,s,stage)=result(a[0] as i8,a[1] as i8,a[2] as i32,a[3] as i32,a[4]!=0,a[5]!=0);
            println!("{status},{},{p},{s},{stage}",cost.map(|v|v.to_string()).unwrap_or("null".into()));
        }
        return;
    }
    let costs=[600,750,400,400,300,600,250,1000,300,500,900,650,600,300];
    for i in i8::MIN..=i8::MAX { assert_eq!(ConstructionSelector::try_from(i).is_ok(),(0..14).contains(&i)); }
    let edge=[i32::MIN,i32::MIN+1,-1001,-1,0,1,599,600,601,999,1000,1001,1600,1601,i32::MAX-1,i32::MAX];
    let mut cost_cases=0u64;let mut transaction_cases=0u64;
    for (selector,base) in costs.into_iter().enumerate() {
        for skill in i8::MIN..=i8::MAX {
            assert_eq!(ConstructionSelector::try_from(selector as i8).unwrap().adjusted_cost(skill),base-2*i32::from(skill));
            cost_cases+=1;
            for p in edge { for s in edge { for refill in [false,true] { for ready in [false,true] {
                assert_eq!(result(selector as i8,skill,p,s,refill,ready),machine(base,skill,p,s,refill,ready));
                transaction_cases+=1;
            }}}}
        }
    }
    let mut seed=0x31c4_8aa5u32;
    let mut next=|| { seed=seed.wrapping_mul(1664525).wrapping_add(1013904223);seed };
    for _ in 0..200000 {
        let selector=(next()%14) as i8;let skill=next() as i8;let p=next() as i32;let s=next() as i32;
        let flags=next();let refill=flags&1!=0;let ready=flags&2!=0;
        assert_eq!(result(selector,skill,p,s,refill,ready),machine(costs[selector as usize],skill,p,s,refill,ready));
        transaction_cases+=1;
    }
    println!("{{\"selector_cases\":256,\"cost_skill_cases\":{cost_cases},\"transaction_cases\":{transaction_cases}}}");
}
