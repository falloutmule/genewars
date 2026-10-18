"""Literal6D830/738F0/73A50 moving-kind1 aim versus private numeric draft."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,random,shutil,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(command):
 r=subprocess.run(command,cwd=P,env=environment(),capture_output=True,timeout=240);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));return r.stdout.decode(errors='replace')
def main():
 w=P/'private/review/turret-geometry'/uuid.uuid4().hex;w.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);model=m.Source()
 files=[P/'private/ghidra'/f for f in ['construction-command/instructions-0006d830.txt','construction-command/instructions-0001f230.txt','callers/instructions-000738f0.txt','countdown-expiry/instructions-00073a50.txt']];model.code.update(m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]))
 mission=P/'private/integration/game-runtime/terrain/CONQ001.DAT';map_bytes=mission.read_bytes()[0x355e:0x3b55e];(w/'map.bin').write_bytes(map_bytes)
 rand=random.Random(0x6D830);data=bytearray();expected=bytearray();instructions=0
 for case in range(256):
  shooter=bytearray(67);target=bytearray(124);shooter[6]=4;shooter[45]=1;target[6]=1
  for record in [shooter,target]:struct.pack_into('<hh',record,7,rand.randrange(-32768,32768),rand.randrange(-32768,32768))
  struct.pack_into('<i',shooter,17,[-32768,0,10000,65536][case%4]);struct.pack_into('<i',target,39,[-32768,0,4096,65534][(case//4)%4])
  for axis,offset in enumerate([45,47,49]):struct.pack_into('<h',target,offset,[-32768,-563,-1,0,1,511,32767][(case+axis*3)%7])
  state={0x1645ae+i:b for i,b in enumerate(shooter)};state.update({0x1542be+i:b for i,b in enumerate(target)});state.update({0x1645ae+35+i:b for i,b in enumerate(struct.pack('<I',0x112120))});state.update({0x1542be+35+i:b for i,b in enumerate(struct.pack('<I',0x111fec))})
  regs=dict(EAX=0x1645ae+7,EDX=int.from_bytes(target[7:11],'little'),EBX=int.from_bytes(shooter[17:21],'little'),ECX=int.from_bytes(target[39:43],'little'))
  report,_,_=model.run(state_before=state,entry=0x738f0,registers=regs);distance=model.r['EAX'];instructions+=report['steps']
  state.update({0x300004+i:b for i,b in enumerate(struct.pack('<II',6400,distance))})
  regs=dict(EAX=0x1645ae,EDX=0x2c1000,EBX=0x1542be,ECX=6400)
  report,_,_=model.run(state_before=state,entry=0x6d830,registers=regs,stack_fill=[0,85,255][case%3]);instructions+=report['steps']
  angles=bytes(model.load(0x2c1000+i,1)for i in range(6));forecast=model.load(0x2fffc0+0x14,4).to_bytes(4,'little');height=model.load(0x2fffbc,4).to_bytes(4,'little')
  report,_,_=model.run(state_before=state,entry=0x7b140,registers={'EAX':0x1645ae+7});instructions+=report['steps'];launch=(model.r['EAX']+6400)&0xffffffff
  data+=shooter+target;expected+=angles+forecast+height+struct.pack('<I',launch)
 (w/'input.bin').write_bytes(data);(w/'expected.bin').write_bytes(expected)
 for name in ['angles.bin','bounce.bin']:(w/name).write_bytes((P/'private/review/mule-mortality'/name).read_bytes())
 draft=P/'private/review/turret-geometry/candidate.rs';pins={}
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  origin=P/f'rust/crates/genewars-{crate}/src';target=w/f'source/{crate}';shutil.copytree(origin,target)
  if crate=='runtime':
   shutil.copyfile(draft,target/'turret_projectile.rs');lib=target/'lib.rs';lib.write_bytes(lib.read_bytes()+b'\npub mod turret_projectile;\n')
  for file in target.rglob('*.rs'):pins[f'{crate}/{file.relative_to(target).as_posix()}']=sha(file)
  cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(target/'lib.rs'),'-L',f'dependency={w}','-o',str(w/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:cmd+=['--extern',f'genewars_{dep}={w}/libgenewars_{dep}.rlib']
  run(cmd)
 run([rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={w}','--extern',f'genewars_runtime={w}/libgenewars_runtime.rlib','-o',str(w/'probe.exe')]);print(run([str(w/'probe.exe'),str(w)]))
 actual=(w/'actual.bin').read_bytes();assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:16]
 paths=model.files+files+[P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',mission,draft,Path(__file__),Path(__file__).with_suffix('.rs'),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py')]
 receipt=dict(status='passed-private-turret-geometry-draft',utc=datetime.now(timezone.utc).isoformat(),cases=256,compared_bytes=len(actual),literal_instructions=instructions,implementation_pins=pins,pins={p.relative_to(P).as_posix():sha(p)for p in paths},private_run=w.relative_to(P).as_posix(),original_execution=False,gui_execution=False,scope='Private numeric draft only:kind1 virtualC=true two-pass velocity lead,fulli16points/velocities,explicit physical/source dwords,actualmap heightclamps,outputangles/speed/forecastpoint/height/launchheight. Source738F0/6D830/73A50/7B140 literal; no projectilefactory,physics,collision or damage claim.')
 (w/'result.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:receipt[k]for k in ['status','cases','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
