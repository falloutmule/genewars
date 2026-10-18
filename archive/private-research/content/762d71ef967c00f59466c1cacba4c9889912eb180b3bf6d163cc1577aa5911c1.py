"""Private controlled context experiment; not a source-startup parity claim."""
from pathlib import Path
import hashlib, json, subprocess, sys, uuid
P=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(P/'scripts'))
from cargo import environment
frames=int(sys.argv[1]) if len(sys.argv)>1 else 10000
assert 1<=frames<=32768
work=P/'private/review/bootstrap-context'/uuid.uuid4().hex
work.mkdir(parents=True)
sha=lambda b:hashlib.sha256(b).hexdigest()
pins={}
for kind in ('assets','core','runtime'):
    for path in (P/f'rust/crates/genewars-{kind}/src').rglob('*.rs'):
        rel=path.relative_to(P);dest=work/'sources'/rel;dest.parent.mkdir(parents=True,exist_ok=True)
        raw=path.read_bytes();dest.write_bytes(raw);pins[rel.as_posix()]=sha(raw)
actor_fixture=work/'sources/rust/crates/genewars-runtime/src/actor_world.rs'
with actor_fixture.open('a',encoding='utf-8') as stream:
    stream.write('\nimpl ActorWorld { pub fn diagnostic_apply_postload_countdown(&mut self, value:u16) {self.backing[0x355c..0x355e].copy_from_slice(&value.to_le_bytes());} }\n')
probe=(P/'scripts/family8_trajectory_probe.rs').read_text()
changes={
 'let bytes=fs::read(runtime.join("terrain/CONQ001.DAT")).unwrap();': 'let bytes=fs::read(runtime.join("terrain/CONQ001.DAT")).unwrap();let active=std::env::args().nth(8).unwrap().parse::<u8>().unwrap();let initial_countdown=std::env::args().nth(9).unwrap().parse::<u16>().unwrap();',
 'world.initialize_mission_player_funds(src.active_count_byte()).unwrap();': 'world.diagnostic_apply_postload_countdown(initial_countdown);world.initialize_mission_player_funds(src.active_count_byte()).unwrap();',
 'world.admit_creatures(&[[0;124];300]).unwrap();': 'world.admit_buildings(&[[0;67];256],[None;256]).unwrap();let bturr=genewars_runtime::building_resources::BuildingResourceDatabase::load_selector1_bturr(fs::read(runtime.join("buildings/BTURR.ANB")).unwrap()).unwrap();world.admit_creatures(&[[0;124];300]).unwrap();',
 'for player in 0..4u8': 'for player in 0..active',
 'world.step_empty_building_bookkeeping(0)': 'world.step_tracking_selector1_building_callbacks(active,0,&bturr,frame,&physics)',
 'world.step_neutral_creature_callbacks(4,': 'world.step_neutral_creature_callbacks(active,',
 'world.step_family8_callbacks(0,4,0,': 'world.step_family8_callbacks(0,active,0x3e,',
}
for before,after in changes.items():
    assert probe.count(before)==1,before
    probe=probe.replace(before,after)
probe_path=work/'probe.rs';probe_path.write_text(probe,encoding='utf-8')
rustc=P/'private/tools/rust/cargo/bin/rustc.exe';env=environment()
def compile(args):
    r=subprocess.run(list(map(str,args)),cwd=P,env=env,capture_output=True,timeout=180)
    assert r.returncode==0,r.stderr.decode(errors='replace')
for kind in ('assets','core','runtime'):
    args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{kind}','--crate-type','rlib',work/f'sources/rust/crates/genewars-{kind}/src/lib.rs','-L',f'dependency={work}','-o',work/f'libgenewars_{kind}.rlib']
    if kind=='runtime':
        for dep in ('assets','core'):args+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
    compile(args)
binary=work/'probe.exe'
args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes',probe_path,'-L',f'dependency={work}','-o',binary]
for dep in ('assets','core','runtime'):args+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
compile(args)
metadata={'status':'running','requested_frames':frames,'scope':'Controlled variations of proven count/countdown values; all other development bootstrap conventions retained. Not original startup.','sources':pins,'private_fixture_world_sha256':sha(actor_fixture.read_bytes()),'fixture_operation':'Private-copy-only typed countdown setter after exact source-hash admission; no source-input mutation or weakened hash check.','binary_sha256':sha(binary.read_bytes()),'probe_sha256':sha(probe_path.read_bytes()),'cases':[]}
(work/'compilation.json').write_text(json.dumps(metadata,indent=2)+'\n')
print(work.relative_to(P),flush=True)
cases=[(5428,1,1000),(65535,1,1000),(5428,1,0),(5428,4,1000)] if frames==10000 else [(0,1,1000),(5428,1,1000),(65535,1,1000)]
for seed,active,countdown in cases:
    output=work/f'seed{seed}-players{active}-timer{countdown}';output.mkdir()
    r=subprocess.run([str(binary),str(P),'mule',str(seed),str(frames),str(output),str(frames),str(frames),str(active),str(countdown)],cwd=P,env=env,capture_output=True,timeout=300)
    (output/'stdout.txt').write_bytes(r.stdout);(output/'stderr.txt').write_bytes(r.stderr)
    lines=r.stdout.decode(errors='replace').splitlines()
    result={'seed':seed,'active':active,'initial_countdown':countdown,'exit':r.returncode,'last_line':lines[-1] if lines else None}
    metadata['cases'].append(result);print(json.dumps(result),flush=True)
metadata['status']='completed-experiment'
(work/'result.json').write_text(json.dumps(metadata,indent=2)+'\n')
