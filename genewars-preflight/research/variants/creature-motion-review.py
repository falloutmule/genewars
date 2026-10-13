"""Independent instruction-derived numeric physics comparison; no original run."""
from pathlib import Path
import subprocess,sys,struct,importlib.util,json,hashlib
from datetime import datetime,timezone
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/creature-motion';W.mkdir(parents=True,exist_ok=True)
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def source_module():
 s=importlib.util.spec_from_file_location('source_model',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def generate_cases(m):
 model=m.Source();state=None;seed=5428;previous=None;physics=bytearray();full=bytearray();expected=bytearray();reports=[]
 for n in range(1,139):
  report,after,trace=model.run(n,seed,state_before=state,capture_physics=True);state=model.saved_state;seed=report['rng_seed_after'];reports.append(report)
  snap=model.physics_snapshots[0];c=model.contact_snapshots;header=struct.pack('<IBh',n,bool(c),int.from_bytes(c[0]['after'][0x41:0x43],'little',signed=True)if c else 0)
  physics+=header+snap['before']+snap['after']
  if previous is not None:full+=header+previous+after
  previous=after;normalized=bytearray(after);normalized[31:39]=bytes(8);expected+=normalized+bytes(state[0x11c2be+i]for i in range(229376))
 (W/'physics-cases.bin').write_bytes(physics);(W/'followup-cases.bin').write_bytes(full);(W/'expected-owned.bin').write_bytes(expected)
 return model,reports
def main():
 (W/'mission.bin').write_bytes((P/'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes())
 obj=(P/'private/ghidra/validated/memory-.object1.bin').read_bytes();obj3=(P/'private/ghidra/validated/memory-.object3.bin').read_bytes()
 (W/'angles.bin').write_bytes(obj[0xae134-0x10000:0xae134-0x10000+514]);(W/'bounce.bin').write_bytes(obj3[0x104530-0x100000:0x104530-0x100000+16])
 (W/'mule.anb').write_bytes((P/'private/reference-run/run-006/drive-c/GAME/DATA/CREATURE/MULE.ANB').read_bytes())
 m=source_module();native_only='--native-only'in sys.argv
 model,reports=(m.Source(),[])if native_only else generate_cases(m)
 paths=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];source_pins=[pin(p)for p in paths]
 rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
 for crate in ['assets','core','runtime']:
  args=[str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for d in ['assets','core']:args+=['--extern',f'genewars_{d}={W}/libgenewars_{d}.rlib']
  r=subprocess.run(args,env=environment(),capture_output=True,timeout=120);assert r.returncode==0,r.stderr.decode(errors='replace')
 args=[str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for d in ['assets','core','runtime']:args+=['--extern',f'genewars_{d}={W}/libgenewars_{d}.rlib']
 for cmd in [args,[str(W/'review.exe'),str(W)]]:
  r=subprocess.run(cmd,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace');print(r.stdout.decode(errors='replace'))
 if native_only:return
 actual=(W/'owned-afterimages.bin').read_bytes();assert actual==(W/'expected-owned.bin').read_bytes();print('138 owned full map/native record comparisons',len(actual))
 # The second context uses the independently reviewed owned reveal/factory
 # result as explicit input. This comparison audits subsequent Mule updates;
 # it is not another proof of that prerequisite transition.
 startup=(W/'startup-context.bin').read_bytes();state={0x11c2be+i:b for i,b in enumerate(startup[0x355e:0x3b55e])};state.update({0x11872c:31,0x118782:64,0x118783:0,0x118784:0,0x118785:0})
 for i in range(512):
  raw=startup[0x43d4e+i*41:0x43d4e+(i+1)*41];native=raw[:31]+bytes(8)+raw[31:]
  state.update({0x15d40e+i*49+j:b for j,b in enumerate(native)if not 35<=j<39})
 model=m.Source();seed=10548;expected=bytearray();startup_reports=[]
 for n in range(1,315):
  report,after,trace=model.run(n,seed,state_before=state);state=model.saved_state;seed=report['rng_seed_after'];startup_reports.append(report)
  normalized=bytearray(after);normalized[31:39]=bytes(8);expected+=normalized+bytes(state[0x11c2be+i]for i in range(229376))
 got=(W/'startup-afterimages.bin').read_bytes();assert got==expected,('startup',next((i for i,(a,b)in enumerate(zip(got,expected))if a!=b),None));(W/'expected-startup.bin').write_bytes(expected)
 assert source_pins==[pin(p)for p in paths]
 result=dict(status='passed-owned-bounded-mule-motion',utc=datetime.now(timezone.utc).isoformat(),source_pins=source_pins,prior_gravity16_receipt="research/variants/creature-motion-gravity16-superseded.json",
  instruction_pins=[pin(p)for p in model.files],review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('initial-creature-source.py'))],
  inputs=[pin(W/f)for f in ['mission.bin','angles.bin','bounce.bin','mule.anb','startup-context.bin']],
  isolated_physics_cases=138,pure_followup_cases=137,owned_callbacks=138,startup_context_callbacks=314,gravity_i16=1024,
  owned_compared_bytes=len(actual)+len(got),rejection_checks=5,first_seed=5428,startup_seed_after64_draws=10548,
  first_context_rng_draws=sum(r['draws']for r in reports),startup_rng_draws=sum(r['draws']for r in startup_reports),
  source_instruction_steps=sum(r['steps']for r in reports+startup_reports),
  outputs=[pin(W/f)for f in ['owned-afterimages.bin','startup-afterimages.bin','physics-cases.bin','followup-cases.bin']],
  original_execution=False,gui_execution=False,
  limits=['Both contexts hold plant callbacks fixed; this is not a whole mission scheduler','Render queue remains an explicit pending request; source83140 body is not executed','Runtime pointer gap31..39 is normalized; opaque gap bytes remain preserved by owned storage',
   'Startup reveal/factory context is an explicit prerequisite from separately reviewed code','Correct1024 gravity oldseed5428 stops before required water ripple factorycode30 at139; frozen startupseed10548 stops before hungry scorer at315','Non-neutral neighborhoods, required water effect factories/damage and unsupported callbacks reject; no silent omission'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out)
 print(json.dumps({k:result[k]for k in ['status','owned_callbacks','startup_context_callbacks','owned_compared_bytes','source_instruction_steps']}))
if __name__=='__main__':main()
