#!/usr/bin/env python3
"""Independent oracle for the bounded associated-type5 activation plan.

The production Rust API is exercised through a generated, private harness.
Python independently samples the pinned trig tables, evaluates acceptance and
the bounded move resolver, and compares complete actor/blueprint afterimages.
No original executable is run and no proprietary bytes are written to the
tracked tree.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]
PRIVATE = ROOT / "private" / "associated-activation-validation"
HARNESS = PRIVATE / "harness"
INPUT = PRIVATE / "cases.bin"
OUTPUT = PRIVATE / "actual.txt"
SINE = ROOT / "private" / "review" / "actor-activation" / "sin.bin"
COSINE = ROOT / "private" / "review" / "actor-activation" / "cos.bin"
ACTIVATION_SOURCE = ROOT / "rust" / "crates" / "genewars-runtime" / "src" / "actor_activation.rs"
COMMAND_SOURCE = ROOT / "rust" / "crates" / "genewars-runtime" / "src" / "actor_command.rs"
INSTRUCTION_A = ROOT / "private" / "ghidra" / "command-resolver" / "instructions-00080ae0.txt"
INSTRUCTION_B = ROOT / "private" / "ghidra" / "plant-command" / "instructions-00074a50.txt"
ACTIVATION_EVIDENCE = ROOT / "research" / "ghidra" / "actor-activation-evidence.json"
MOVE_ORDER_EVIDENCE = ROOT / "research" / "ghidra" / "move-order-evidence.json"
EXPECTED_SINE = "84dfe6902e4c8947b75f4ff0a54d82fced478dd9682255eaa973a88cbf76cf3c"
EXPECTED_COSINE = "688e629369dc1c5379bae04b5e11100a4fd0aabd89745f2c0b5b6309ce0261c9"
EXPECTED_80AE0 = "f6abb34796b3a71a8443f62f35547b049f371968f15a7365dea2c367858739a6"
EXPECTED_74A50 = "c44b051dc39fd5998975fecccd10798615d09cba01a10c6e909646373358861a"
MAP_BYTES = 128 * 128 * 14


HARNESS_RS = r'''use genewars_runtime::actor_activation::{
    plan_associated_actor_activation, sample_placement_candidate, ActivationPlanningOutcome,
    CandidateEnvironment, ResolvedCellHead, TrigTables,
};
use std::{env, fs};

fn take<'a>(b: &'a [u8], p: &mut usize, n: usize) -> &'a [u8] {
    let s = *p; let e = s.checked_add(n).expect("input cursor overflow");
    let v = b.get(s..e).expect("truncated case input"); *p = e; v
}
fn u8v(b: &[u8], p: &mut usize) -> u8 { take(b,p,1)[0] }
fn u16v(b: &[u8], p: &mut usize) -> u16 { u16::from_le_bytes(take(b,p,2).try_into().unwrap()) }
fn i16v(b: &[u8], p: &mut usize) -> i16 { i16::from_le_bytes(take(b,p,2).try_into().unwrap()) }
fn i32v(b: &[u8], p: &mut usize) -> i32 { i32::from_le_bytes(take(b,p,4).try_into().unwrap()) }
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
fn head_cell(point: [u16;2]) -> usize { usize::from((point[1] >> 8) & 127) * 128 + usize::from((point[0] >> 8) & 127) }

fn main() {
    let a: Vec<String> = env::args().collect();
    let input = fs::read(&a[1]).unwrap();
    let sine = fs::read(&a[2]).unwrap(); let cosine = fs::read(&a[3]).unwrap();
    let tables = TrigTables::from_mapped_bytes(&sine,&cosine).unwrap();
    let mut p=0usize; let count=u16v(&input,&mut p) as usize;
    let mut lines=Vec::new();
    for ci in 0..count {
        let actor_id=i16v(&input,&mut p); let record_id=i16v(&input,&mut p); let slot=u8v(&input,&mut p) as usize;
        let center=[u16v(&input,&mut p),u16v(&input,&mut p)]; let seed=u16v(&input,&mut p);
        let obstacle=u8v(&input,&mut p)!=0; let flags71=u8v(&input,&mut p);
        let flags57=u8v(&input,&mut p); let state82=u8v(&input,&mut p);
        let avoid=u8v(&input,&mut p); let category=u8v(&input,&mut p);
        let bp_count=u32::from_le_bytes(take(&input,&mut p,4).try_into().unwrap());
        let head_id=i16v(&input,&mut p); let head_kind=u8v(&input,&mut p);
        let nenv=u16v(&input,&mut p) as usize;
        let mut envs=Vec::with_capacity(nenv);
        for _ in 0..nenv {
            let terrain=u8v(&input,&mut p); let blocker=i32v(&input,&mut p);
            let has=u8v(&input,&mut p)!=0; let resolved_id=i16v(&input,&mut p); let resolved_kind=u8v(&input,&mut p);
            envs.push(CandidateEnvironment { terrain_class:terrain, blocker_weight:blocker,
                resolved_cell_head: if has {Some(ResolvedCellHead {record_id:resolved_id,record_type:resolved_kind})} else {None} });
        }
        let mut actor=core::array::from_fn::<_,200,_>(|i| (i.wrapping_mul(17).wrapping_add(ci.wrapping_mul(7))) as u8);
        actor[6]=3; actor[0x0f..0x11].copy_from_slice(&record_id.to_le_bytes());
        actor[0x53]=avoid; actor[0x70..0x72].copy_from_slice(&0x0400u16.to_le_bytes());
        actor[0x71]=0x04 | flags71; actor[0x57]=flags57; actor[0x82]=state82; actor[0x74]=0;
        let mut blueprints=vec![0u8;32*36]; let bstart=0usize;
        blueprints[bstart+0x16]=category; blueprints[bstart+0x18]=9;
        blueprints[bstart+0x1c..bstart+0x20].copy_from_slice(&bp_count.to_le_bytes());
        let mut map=vec![0u8;128*128*14];
        let center_cell=head_cell(center); let headoff=center_cell*14+6;
        map[headoff..headoff+2].copy_from_slice(&head_id.to_le_bytes()); map[headoff+2]=head_kind;
        // Destination cells carry a decoy head to prove the plan uses the
        // associated object's center cell for insertion.
        for i in 0..nenv {
            if let Ok(c)=sample_placement_candidate(seed,center,&tables) {
                let cell=head_cell(c.position_u15); let off=cell*14+6;
                if cell!=center_cell { map[off..off+3].copy_from_slice(&[0x63,0,4]); }
            }
        }
        let result=plan_associated_actor_activation(actor_id,slot,&actor,&blueprints,&map,center,seed,&tables,&envs,obstacle);
        match result {
            Err(e) => lines.push(format!("{ci}|ERR|{e:?}")),
            Ok(ActivationPlanningOutcome::Ineligible {..}) => lines.push(format!("{ci}|INELIGIBLE")),
            Ok(ActivationPlanningOutcome::Planned(plan)) => {
                let attempts=plan.candidate_attempts.iter().map(|r| format!("{},{},{},{},{},{}",
                    r.candidate.seed_before,r.candidate.seed_after,r.candidate.angle_index,
                    r.candidate.position_u15[0],r.candidate.position_u15[1],u8::from(r.accepted)))
                    .collect::<Vec<_>>().join(";");
                let linkprev=plan.link.old_head_previous_after.map(|v|hex(&v)).unwrap_or_else(||"-".into());
                lines.push(format!("{ci}|OK|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                    plan.seed_after,plan.link.cell_index,attempts,hex(&plan.actor_after),hex(&plan.blueprint_after),
                    hex(&plan.link.expected_old_head),hex(&plan.link.new_cell_head),linkprev,
                    hex(&plan.link.actor_next_after)));
            }
        }
    }
    assert_eq!(p,input.len(),"unconsumed case input");
    fs::write(&a[4],lines.join("\n")+"\n").unwrap();
}
'''


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def put_case(case: dict) -> bytes:
    b = bytearray()
    b += struct.pack("<hhBHHHBBBBBBIhBH", case["actor_id"], case.get("record_id",case["actor_id"]), case["slot"], *case["center"], case["seed"],
                     int(case["obstacle"]), case["flags71"], case["flags57"], case["state82"],
                     case["avoid"], case["category"], case["bp_count"], case["head_id"],
                     case["head_kind"], len(case["envs"]))
    for item in case["envs"]:
        resolved = item.get("resolved")
        b += struct.pack("<BiBhB", item["terrain"], item["blocker"], int(resolved is not None),
                         resolved[0] if resolved else 0, resolved[1] if resolved else 0)
    return bytes(b)


def cases() -> list[dict]:
    targeted = [
        dict(name="far-destination-null-center", actor_id=0, slot=0, center=[20736,13568], seed=0, obstacle=False,
             flags71=0, flags57=0x40, state82=0, avoid=0, category=1, bp_count=0, head_id=123, head_kind=0,
             envs=[dict(terrain=1,blocker=0)]),
        dict(name="far-destination-obstacle-rejected", actor_id=5, slot=0, center=[20736,13568], seed=1, obstacle=True,
             flags71=0, flags57=0x40, state82=0, avoid=0, category=1, bp_count=0, head_id=777, head_kind=0,
             envs=[dict(terrain=1,blocker=0)]),
        dict(name="flag71-noop-precedes-obstacle-and-reinit", actor_id=10, slot=0, center=[32767,32767], seed=65535,
             obstacle=True, flags71=0x10, flags57=1, state82=3, avoid=0, category=0, bp_count=0xFFFFFFFF,
             head_id=99, head_kind=0, envs=[dict(terrain=2,blocker=0)]),
        dict(name="reject-then-accept", actor_id=1, slot=1, center=[0,0], seed=5428, obstacle=False,
             flags71=0, flags57=0x80, state82=0, avoid=1, category=1, bp_count=0xFFFFFFFF,
             head_id=-2, head_kind=0, envs=[dict(terrain=0,blocker=0),dict(terrain=1,blocker=0)]),
        dict(name="resolved-null-family-head", actor_id=2, slot=2, center=[32767,0], seed=17, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=321, head_kind=0, envs=[dict(terrain=1,blocker=0,resolved=[0,3])]),
        dict(name="nonnull-center-head-and-destination-decoy", actor_id=3, slot=3, center=[0,32767], seed=65534, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=7, head_kind=4, envs=[dict(terrain=1,blocker=0,resolved=[7,4])]),
        dict(name="nonnull-head-mismatch", actor_id=4, slot=4, center=[20736,13568], seed=2, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=7, head_kind=4, envs=[dict(terrain=1,blocker=0,resolved=[8,4])]),
        dict(name="nonnull-head-missing-resolution", actor_id=6, slot=1, center=[123,456], seed=3, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=12, head_kind=2, envs=[dict(terrain=1,blocker=0)]),
        dict(name="retry-exhaustion", actor_id=7, slot=2, center=[2048,8192], seed=9, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=255, category=1, bp_count=0,
             head_id=0, head_kind=0, envs=[dict(terrain=7,blocker=0)]*3),
        dict(name="reinitialization-flag-rejected", actor_id=8, slot=3, center=[32767,32767], seed=0x1234, obstacle=False,
             flags71=0, flags57=0x41, state82=0, avoid=0, category=1, bp_count=0,
             head_id=0, head_kind=0, envs=[dict(terrain=1,blocker=0)]),
        dict(name="reinitialization-state-rejected", actor_id=9, slot=4, center=[1,32766], seed=0xBEEF, obstacle=False,
             flags71=0, flags57=0x40, state82=1, avoid=0, category=1, bp_count=0,
             head_id=0, head_kind=0, envs=[dict(terrain=1,blocker=0)]),
        dict(name="invalid-terrain-class", actor_id=11, slot=1, center=[0,0], seed=4, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=0, head_kind=0, envs=[dict(terrain=8,blocker=0)]),
        dict(name="negative-blocker", actor_id=12, slot=2, center=[0,0], seed=4, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=0, head_kind=0, envs=[dict(terrain=1,blocker=-1)]),
        dict(name="obstacle-with-retry-before-acceptance", actor_id=13, slot=3, center=[32760,7], seed=65530,
             obstacle=True, flags71=0, flags57=0, state82=0, avoid=2, category=1, bp_count=0,
             head_id=0, head_kind=0, envs=[dict(terrain=1,blocker=12),dict(terrain=0,blocker=0)]),
        dict(name="invalid-center", actor_id=14, slot=4, center=[32768,0], seed=2, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=0, head_kind=0, envs=[dict(terrain=1,blocker=0)]),
        dict(name="bad-slot", actor_id=15, slot=5, center=[100,100], seed=2, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=0, head_kind=0, envs=[dict(terrain=1,blocker=0)]),
        dict(name="record-id-mismatch", actor_id=16, slot=1, center=[400,600], seed=8, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=0, head_kind=0, envs=[dict(terrain=1,blocker=0)], record_id=99),
        dict(name="associated-center-edge-zero", actor_id=17, slot=2, center=[0,0], seed=65535, obstacle=False,
             flags71=0, flags57=0x20, state82=0, avoid=0, category=0, bp_count=7,
             head_id=0x7123, head_kind=0, envs=[dict(terrain=0,blocker=0)]),
        dict(name="associated-center-edge-max", actor_id=18, slot=3, center=[32767,32767], seed=0xA55A, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0xFFFFFFFE,
             head_id=-1, head_kind=0, envs=[dict(terrain=2,blocker=0)]),
        dict(name="accepted-head-invalid-index", actor_id=19, slot=4, center=[16000,16000], seed=991, obstacle=False,
             flags71=0, flags57=0, state82=0, avoid=0, category=1, bp_count=0,
             head_id=-1, head_kind=8, envs=[dict(terrain=1,blocker=0,resolved=[-1,8])]),
    ]
    owners = range(4)
    seeds = [0, 1, 5428, 65535]
    centers = [[0,0], [32767,32767], [20736,13568], [16383,16384]]
    matrix=[]
    case_no=0
    for owner in owners:
        for slot in range(5):
            actor_id=owner*5+slot
            for si,seed in enumerate(seeds):
                for ci,center in enumerate(centers):
                    for nonnull in (False,True):
                        head_id=(actor_id+1)%20 if nonnull else (0x4000+case_no if case_no%2 else -case_no-1)
                        head_kind=((owner*5+slot)%8)+1 if nonnull else 0
                        flag_extra=0x10 if (owner+slot+si+ci)%7==0 else (0x02 if case_no%3==0 else 0x80 if case_no%3==1 else 0)
                        category=(owner+slot+si+ci)&1
                        count=[0,0xFFFFFFFF,0xFFFFFFFE,0x7FFFFFFF][(owner+slot+si+ci)%4]
                        matrix.append(dict(name=f"matrix-o{owner}-s{slot}-seed{seed}-c{ci}-head{int(nonnull)}",
                            actor_id=actor_id,slot=slot,center=center,seed=seed,obstacle=False,
                            flags71=flag_extra,flags57=0x40|((case_no*13)&0x7E),state82=0,avoid=0,
                            category=category,bp_count=count,head_id=head_id,head_kind=head_kind,
                            envs=[dict(terrain=(case_no+si)%8,blocker=case_no%10,
                                       **({"resolved":[head_id,head_kind]} if nonnull else {}))]))
                        case_no+=1
    return targeted + matrix


def i16(x: int) -> int:
    x &= 0xFFFF
    return x - 0x10000 if x & 0x8000 else x


def candidate(seed: int, center: list[int], sin: list[int], cos: list[int]) -> tuple[int,int,int,list[int]]:
    after = (seed * 0xE62D) & 0xFFFF
    angle = ((after >> 8) * 8) & 0xFFFF
    ci = (angle - 512) & 2047
    dx = i16((1024 * cos[ci]) >> 16)
    dy = i16((1024 * sin[ci]) >> 16)
    x = (i16((i16(center[0]) * 2 + dx)) >> 1) & 0x7FFF
    y = (i16((i16(center[1]) * 2 + dy)) >> 1) & 0x7FFF
    return after, angle, ci, [x,y]


def cell(point: list[int]) -> int:
    return (((point[1] >> 8) & 127) * 128) + ((point[0] >> 8) & 127)


def initial_actor(c: dict, ci: int) -> bytearray:
    b = bytearray((i * 17 + ci * 7) & 255 for i in range(200))
    b[6] = 3
    b[0x0F:0x11] = struct.pack("<h", c.get("record_id", c["actor_id"]))
    b[0x53] = c["avoid"]
    b[0x70:0x72] = struct.pack("<H", 0x0400)
    b[0x71] = 0x04 | c["flags71"]
    b[0x57] = c["flags57"]
    b[0x82] = c["state82"]
    b[0x74] = 0
    return b


def oracle(c: dict, ci: int, sin: list[int], cos: list[int]) -> dict:
    if not (0 <= c["actor_id"] < 20): return {"error":f"InvalidActorId({c['actor_id']})"}
    if not (0 <= c["slot"] < 5): return {"error":f"InvalidSlotIndex({c['slot']})"}
    if c["slot"] != c["actor_id"] % 5: return {"error":f"ActorSlotMismatch {{ actor_id: {c['actor_id']}, slot_index: {c['slot']} }}"}
    actor = initial_actor(c,ci)
    if c.get("record_id",c["actor_id"]) != c["actor_id"]: return {"error":f"ActorIdMismatch {{ parameter: {c['actor_id']}, record: {c.get('record_id')} }}"}
    if not all(0 <= n <= 32767 for n in c["center"]): return {"error":f"InvalidCenter([{c['center'][0]}, {c['center'][1]}])"}
    if len(c["envs"]) > 4096: return {"error":"CandidateListExhausted { attempts: 4096 }"}
    attempt_rows=[]; draw=c["seed"]; accepted=None
    for env in c["envs"]:
        if env["terrain"] > 7: return {"error":f"InvalidTerrainClass({env['terrain']})"}
        if env["blocker"] < 0: return {"error":f"NegativeBlockerWeight({env['blocker']})"}
        seed_after,angle,_,point=candidate(draw,c["center"],sin,cos)
        ok=(c["avoid"] & (1 << env["terrain"]) == 0 and env["blocker"] < 10)
        attempt_rows.append((draw,seed_after,angle,point[0],point[1],int(ok)))
        draw=seed_after
        if ok:
            accepted=(env,point)
            break
    if accepted is None: return {"error":f"CandidateListExhausted {{ attempts: {len(c['envs'])} }}"}
    env,point=accepted
    head_id=c["head_id"]; head_kind=c["head_kind"]
    if head_kind == 0:
        if env.get("resolved") is not None: return {"error":"UnexpectedResolvedCellHead"}
    else:
        if head_id < 0 or not 1 <= head_kind <= 8: return {"error":f"InvalidReference {{ index: {head_id}, kind: {head_kind} }}"}
        resolved=env.get("resolved")
        if resolved is None: return {"error":f"MissingResolvedCellHead {{ index: {head_id}, kind: {head_kind} }}"}
        if resolved != [head_id,head_kind]:
            return {"error":f"ResolvedCellHeadMismatch {{ raw_index: {head_id}, raw_kind: {head_kind}, record_id: {resolved[0]}, record_kind: {resolved[1]} }}"}

    placement=c["center"]
    link_cell=cell(placement)
    after=bytearray(actor)
    actor_ref=struct.pack("<hB",c["actor_id"],3)
    after[0:3]=struct.pack("<hB",link_cell,0)
    if head_kind:
        after[3:6]=struct.pack("<hB",head_id,head_kind)
    else:
        # The source clears kind only and keeps the prior next index word.
        after[3:6]=after[3:5]+b"\0"
    flags=(struct.unpack("<H",after[0x70:0x72])[0] & 0xFBFE) | 1
    after[0x70:0x72]=struct.pack("<H",flags)
    after[0x16] &= 0xF7
    after[7:11]=struct.pack("<hh",*placement)
    requested=[i16(point[0]),i16(point[1])]
    if c["flags71"] & 0x10:
        pass
    else:
        if c["flags57"] & 1 or c["state82"] != 0:
            return {"error":f"AssociatedMoveCommand(ReinitializationRequired {{ flags_at_57: {c['flags57']}, state_at_82: {c['state82']} }})"}
        dx=((i16(placement[0])-requested[0]+16384)&32767)-16384
        dy=((i16(placement[1])-requested[1]+16384)&32767)-16384
        distance=dx*dx+dy*dy
        if distance <=4096:
            resolved=requested if False else [i16(placement[0]),i16(placement[1])]
        elif not c["obstacle"] and c["avoid"] & (1 << env["terrain"]) == 0 and env["blocker"] < 10:
            resolved=requested
        else:
            return {"error":"AssociatedMoveResolver(BackwardSearchRequired)"}
        after[0x57] = c["flags57"] | 2
        after[0x5B] = 0
        after[0x5C:0x60]=struct.pack("<hh",*resolved)
        after[0x0B:0x0F]=struct.pack("<hh",*resolved)
    bp=bytearray(36)
    bp[0x16]=c["category"]; bp[0x18]=9; bp[0x1C:0x20]=struct.pack("<I",c["bp_count"])
    count=(c["bp_count"]+1)&0xFFFFFFFF
    bp[0x1B]=bp[0x18]; bp[0x20] |= 2; bp[0x1C:0x20]=struct.pack("<I",count)
    if count == 1 and c["category"] == 1: after[0x72]=40
    if head_kind:
        old_head_previous=bytes(actor_ref)
    else:
        old_head_previous=None
    raw_head=struct.pack("<hB",head_id,head_kind)
    return {"seed_after":draw,"cell":link_cell,"attempts":attempt_rows,"actor":bytes(after),
            "blueprint":bytes(bp),"head":raw_head,"new_head":bytes(actor_ref),
            "old_head_previous":old_head_previous,"actor_next":bytes(after[3:6])}


def write_harness() -> None:
    HARNESS.mkdir(parents=True,exist_ok=True)
    (HARNESS/"src").mkdir(exist_ok=True)
    (HARNESS/"Cargo.toml").write_text(
        '[package]\nname="associated-activation-harness"\nversion="0.1.0"\nedition="2021"\n'
        '[dependencies]\ngenewars-runtime={path="../../../rust/crates/genewars-runtime"}\n', encoding="utf-8")
    (HARNESS/"src"/"main.rs").write_text(HARNESS_RS,encoding="utf-8")


def run_harness(payload: bytes) -> list[str]:
    INPUT.parent.mkdir(parents=True,exist_ok=True)
    INPUT.write_bytes(payload)
    OUTPUT.unlink(missing_ok=True)
    cargo_py=ROOT/"scripts"/"cargo.py"
    spec=importlib.util.spec_from_file_location("project_cargo",cargo_py)
    cargo_mod=importlib.util.module_from_spec(spec); spec.loader.exec_module(cargo_mod)
    cargo=ROOT/"private"/"tools"/"rust"/"cargo"/"bin"/"cargo.exe"
    env=cargo_mod.environment()
    manifest=HARNESS/"Cargo.toml"
    lock=HARNESS/"Cargo.lock"
    if not lock.exists():
        subprocess.run([str(cargo),"generate-lockfile","--offline","--manifest-path",str(manifest)],
                       cwd=ROOT/"rust",env=env,check=True,capture_output=True,text=True)
    subprocess.run([str(cargo),"run","--offline","--locked","--manifest-path",str(manifest),"--",
                    str(INPUT),str(SINE),str(COSINE),str(OUTPUT)],
                   cwd=ROOT/"rust",env=env,check=True,capture_output=True,text=True)
    return OUTPUT.read_text(encoding="utf-8").splitlines()


def compare() -> dict:
    for path in (SINE,COSINE,ACTIVATION_SOURCE,COMMAND_SOURCE,INSTRUCTION_A,INSTRUCTION_B,ACTIVATION_EVIDENCE,MOVE_ORDER_EVIDENCE):
        if not path.is_file(): raise RuntimeError(f"required pinned file missing: {path}")
    activation_receipt=json.loads(ACTIVATION_EVIDENCE.read_text(encoding="utf-8"))
    move_receipt=json.loads(MOVE_ORDER_EVIDENCE.read_text(encoding="utf-8"))
    pin_a=next((p["sha256"] for p in activation_receipt["instruction_contracts"]
                if p["private_path"]=="private/ghidra/command-resolver/instructions-00080ae0.txt"),None)
    pin_b=next((p["sha256"] for p in move_receipt["instruction_contracts"]
                if p["private_path"]=="private/ghidra/plant-command/instructions-00074a50.txt"),None)
    if pin_a!=EXPECTED_80AE0 or pin_b!=EXPECTED_74A50 or sha(INSTRUCTION_A)!=EXPECTED_80AE0 or sha(INSTRUCTION_B)!=EXPECTED_74A50:
        raise RuntimeError("associated branch instruction pins differ from the approved public evidence")
    if activation_receipt.get("candidate_sha256")!=move_receipt.get("candidate_sha256"):
        raise RuntimeError("activation/move-resolver receipts do not refer to the same candidate binary")
    sine_bytes=SINE.read_bytes(); cosine_bytes=COSINE.read_bytes()
    if hashlib.sha256(sine_bytes).hexdigest()!=EXPECTED_SINE or hashlib.sha256(cosine_bytes).hexdigest()!=EXPECTED_COSINE:
        raise RuntimeError("private trig table hash admission failed")
    sin=[struct.unpack_from("<i",sine_bytes,i)[0] for i in range(0,len(sine_bytes),4)]
    cos=[struct.unpack_from("<i",cosine_bytes,i)[0] for i in range(0,len(cosine_bytes),4)]
    cs=cases()
    payload=struct.pack("<H",len(cs))+b"".join(put_case(c) for c in cs)
    actual=run_harness(payload)
    if len(actual)!=len(cs): raise AssertionError(f"harness output count {len(actual)} != {len(cs)}")
    outcomes=[]; success=0; errors=0; total_attempts=0
    for i,(case,line) in enumerate(zip(cs,actual)):
        cols=line.split("|")
        if int(cols[0])!=i: raise AssertionError(f"row identity mismatch {line}")
        expected=oracle(case,i,sin,cos)
        if "error" in expected:
            want=expected["error"]
            got="|".join(cols[2:]) if cols[1]=="ERR" else cols[1]
            if cols[1]!="ERR" or got!=want:
                raise AssertionError(f"{case['name']}: error mismatch expected {want!r}, got {line!r}")
            errors+=1; outcomes.append({"name":case["name"],"result":"rejected","error":want})
            continue
        if cols[1]!="OK": raise AssertionError(f"{case['name']}: expected success, got {line!r}")
        # seed, cell, attempt list, actor, blueprint, raw head, new head,
        # previous-head afterimage and actor-next bytes.
        if int(cols[2])!=expected["seed_after"]: raise AssertionError(f"{case['name']}: seed after mismatch")
        if int(cols[3])!=expected["cell"]: raise AssertionError(f"{case['name']}: link cell is not associated center")
        attempts=[]
        if cols[4]:
            for item in cols[4].split(";"):
                vals=tuple(map(int,item.split(","))); attempts.append(vals)
        if attempts!=expected["attempts"]: raise AssertionError(f"{case['name']}: candidate/retry receipt differs")
        comparisons=((5,"actor"),(6,"blueprint"),(7,"head"),(8,"new_head"),(10,"actor_next"))
        for col,key in comparisons:
            if bytes.fromhex(cols[col])!=expected[key]: raise AssertionError(f"{case['name']}: full {key} afterimage differs")
        want_prev=expected["old_head_previous"]
        if (cols[9]=="-") != (want_prev is None) or (want_prev is not None and bytes.fromhex(cols[9])!=want_prev):
            raise AssertionError(f"{case['name']}: old center-head previous link differs")
        success+=1; total_attempts+=len(attempts)
        outcomes.append({"name":case["name"],"result":"planned","attempts":len(attempts),
                         "center_cell":expected["cell"],"candidate_positions":[list(a[3:5]) for a in attempts],
                         "seed_after":expected["seed_after"]})
    return {"cases":len(cs),"planned":success,"rejected":errors,"candidate_attempts":total_attempts,
            "matrix":{"owners":4,"slots_per_owner":5,"seeds":[0,1,5428,65535],
                      "centers":[[0,0],[32767,32767],[20736,13568],[16383,16384]],
                      "head_states":["null with opaque index","nonnull resolved identity"],
                      "matrix_cases":4*5*4*4*2},
            "outcomes":outcomes,"source_pins":{
                "actor_activation_rs_sha256":sha(ACTIVATION_SOURCE),"actor_command_rs_sha256":sha(COMMAND_SOURCE),
                "80ae0_instructions_sha256":sha(INSTRUCTION_A),"74a50_instructions_sha256":sha(INSTRUCTION_B),
                "sine_table_sha256":sha(SINE),"cosine_table_sha256":sha(COSINE),
                "activation_evidence_sha256":sha(ACTIVATION_EVIDENCE),"move_order_evidence_sha256":sha(MOVE_ORDER_EVIDENCE),
                "candidate_binary_sha256":activation_receipt["candidate_sha256"]},
            "harness_source_sha256":hashlib.sha256(HARNESS_RS.encode()).hexdigest(),
            "private_harness_input_sha256":sha(INPUT),"private_harness_output_sha256":sha(OUTPUT)}


def main() -> int:
    write_harness()
    summary=compare()
    receipt={"schema":"associated-activation-validation-v1","status":"pass",
             "method":"generated private Rust harness + independently written Python oracle",
             "command":"python -B scripts/validate_associated_activation.py (harness invokes cargo run --offline --locked)",
             "scope":["explicit seeded candidate generation and retry ordering",
                      "associated position is the map insertion center while accepted candidate is pending destination",
                      "complete actor and blueprint afterimages","flag71 no-op precedence",
                      "bounded resolver obstacle rejection","raw null/non-null link triples"],
             "limitations":["does not execute original binary","does not establish landing schedule or shared RNG history",
                            "environment values are explicit inputs; no terrain/blocker query is modeled"],
             **summary}
    out=ROOT/"research"/"integration"/"associated-activation-validation.json"
    out.parent.mkdir(parents=True,exist_ok=True)
    out.write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps({k:v for k,v in receipt.items() if k!="outcomes"},indent=2))
    return 0


if __name__=="__main__":
    raise SystemExit(main())
