"""Fresh frame0 whole Mule callback compared to literal5ED20, no original run."""
from pathlib import Path
import importlib.util,hashlib,json,subprocess,sys,uuid,struct
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
w=P/'private/review/frame-zero-mule'/uuid.uuid4().hex;w.mkdir(parents=True)
subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
target=P/'private/build/rust-target/debug';exe=w/'review.exe';cmd=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/ghidra/mission_contract_frame_zero_fixture.rs','-L',f'dependency={target}/deps','-o',str(exe)]
for crate in ['assets','core','runtime']:cmd+=['--extern',f'genewars_{crate}={target}/libgenewars_{crate}.rlib']
r=subprocess.run(cmd,cwd=P,env=environment(),capture_output=True);assert r.returncode==0,r.stderr.decode()
r=subprocess.run([str(exe),str(P),str(w)],capture_output=True);assert r.returncode==0,r.stderr.decode()
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m);rows=[]
for case in list(range(8))+[9,10,11]:
 seed,ring,count=struct.unpack('<HBI',(w/f'{case}-before-rng.bin').read_bytes())
 backing=(w/f'{case}-before-backing.bin').read_bytes();native=(w/f'{case}-before-native.bin').read_bytes();state={0x11c2be+i:b for i,b in enumerate(backing[0x355e:0x3b55e])};state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',count))})
 for i in range(512):
  raw=backing[0x43d4e+i*41:0x43d4e+(i+1)*41];plant=raw[:31]+bytes(8)+raw[31:];state.update({0x15d40e+i*49+j:b for j,b in enumerate(plant) if not 35<=j<39})
 x=m.Source();report,after,trace=x.run(counter=case%4,seed=seed,native_before=native,state_before=state);after=bytearray(after);after[31:39]=native[31:39]
 assert after==(w/f'{case}-after-native.bin').read_bytes(),('native',case)
 assert bytes(x.load(0x11c2be+i,1)for i in range(229376))==(w/f'{case}-after-map.bin').read_bytes(),('map',case)
 assert struct.pack('<HBI',report['rng_seed_after'],report['rng_ring_after'],report['rng_draw_count_after'])==(w/f'{case}-after-rng.bin').read_bytes()
 rows.append({'case':case,'frame':case%4,'startup_visibility':4<=case<8,'development_frame1_start':case>=8,'instructions':report['steps'],'compared_bytes':229500,'draws':report['draws']})
r={'status':'passed-frame-zero-mule-callback','cases':rows,'private_run':w.relative_to(P).as_posix(),'production_sha256':{name:hashlib.sha256((P/'rust/crates/genewars-runtime/src'/name).read_bytes()).hexdigest()for name in ['creature_world.rs','actor_world/family_scheduler.rs']},'instruction_pins':{f.relative_to(P).as_posix():hashlib.sha256(f.read_bytes()).hexdigest()for f in x.files},'scope':'Initial Mule frames0..3 on exactCONQ001, before and after72AD0 visibility; full native record/map/RNG. Additional development frame1 start regression. All use same generic callback through owned family scheduler; no frame1 hash shortcut.','original_execution':False}
(P/'research/campaign/frame-zero-mule.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({'status':r['status'],'cases':rows,'private_run':r['private_run']}))
