from pathlib import Path
import importlib.util,struct,subprocess,json,hashlib,uuid
p=Path(__file__).resolve().parents[2];w=p/'private/ghidra/construction-command'/uuid.uuid4().hex;w.mkdir(parents=True);spec=importlib.util.spec_from_file_location('maintenance_source',p/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Source(m.Source):
 def get(self,o):
  if o in ['0x00084960','0x00084b40']:self.viewport.append(((self.r['EDX']>>24)&127)*128+((self.r['EDX']>>8)&127))
  if o=='0x00084e10':self.viewport.append(((self.r['EDX']>>24)&127)*128+((self.r['EDX']>>8)&127));return 0x2e0000
  return super().get(o)
model=Source();files=[p/'private/ghidra/construction-command'/f'instructions-000{x}.txt'for x in ['78e30','8c1c0','8c290','8cde0']]+[p/'private/ghidra/terrain-map-refs'/f'instructions-000{x}.txt'for x in ['8cb00','8c990','8b890','8d1b0','8d660','8cba0']]+[p/'private/ghidra/validated/instructions-00097910.txt'];pins={'private/ghidra/construction-command/instructions-00078e30.txt': '35df6f54b4c51a4b974cb11de1e2aac0357f5e31308f9c7a83ce4ed252fe44d6', 'private/ghidra/terrain-map-refs/instructions-0008cb00.txt': '2c35541f4b44258aec8419dda54367ad5ad45366e3030108b87eb8a4fb45a921', 'private/ghidra/terrain-map-refs/instructions-0008c990.txt': '1a60f5357184688c8da3b86e88d20f7b53cdf7aafafe46cfcb10ca527a978837', 'private/ghidra/terrain-map-refs/instructions-0008d660.txt': 'f6a40917d785effe8a93813bbbb45708d5d9dde3cd58c251e908899693309d0d', 'private/ghidra/terrain-map-refs/instructions-0008d1b0.txt': '0b70076ba33011160a5f0cb917d7c89dd0675f9ff33edd129469ef75a27405c8', 'private/ghidra/construction-command/instructions-0008c290.txt': 'dcd3cda09865c093a5e1df220935f2a51a6b2c33bd35a62d5528e4eac08a952b', 'private/ghidra/construction-command/instructions-0008cde0.txt': 'eb4efe6a450c354651410f3240dc5709035f28c44866dcd99251433c94cd4850', 'private/ghidra/terrain-map-refs/instructions-0008b890.txt': 'b1b285e56911bee26f33d6aa3c2a5ccd42481e25be7fc4f986f5e55bbb173ad3', 'private/ghidra/terrain-map-refs/instructions-0008cba0.txt': '8346e2e8cc123527be927e2b30cf62f629864ad94c5637a07d8b7e0297ea9c0e', 'private/ghidra/construction-command/instructions-0008c1c0.txt': 'c7be373421095700644a1fcb13f7e3de744c886bebc54ab304fbe708b3a1b586', 'private/ghidra/validated/instructions-00097910.txt': '8581fffdf51047b5d57eaa70409790ab8757ddaed1c977624a8f75958be3600a', 'private/ghidra/terrain-map-refs/instructions-000753f0.txt': 'ea2554be981b0acae18e192ba9827ebdf4e4dea2d8e6fbee83f889ec15f390c5', 'private/ghidra/validated/instructions-0008d580.txt': '08fc85698d73460faaef4a6dd81457eb199a60c9aacda9380239e5f3278034c3'};assert all(hashlib.sha256((p/q).read_bytes()).hexdigest()==digest for q,digest in pins.items()),'terrain maintenance source changed';files += [p/'private/ghidra/terrain-map-refs/instructions-000753f0.txt',p/'private/ghidra/validated/instructions-0008d580.txt'];ext=m.m.Model(files,b'');model.code.update(ext.code);model.code[0x2e0000]=('RET',[]);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.mem={};table=bytes(model.load(0x112d24+i,1)for i in range(8192));mission=(p/'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes();assert hashlib.sha256(mission).hexdigest()=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2','CONQ fixture changed';assert hashlib.sha256(table).hexdigest()=='9f1dcbc35c350d6027f98be0f5c8b43b42ca52b7604459c0c42be3aa88913d47','border table changed';mapbytes=mission[0x355e:0x355e+229376];job=bytearray(176);struct.pack_into('<4hBBhh',job,0,76*256,49*256,79*256,52*256,4,4,7,1)
# Preserve the historical mapped-prior proof unless generated startup input is requested.
import sys
generated='--generated-table' in sys.argv
if generated:
 patterns=(p/'private/ghidra/validated/memory-.object3.bin').read_bytes()[0x5060:0x52e0]
 assert hashlib.sha256(patterns).hexdigest()=='d848e423b7a76a2c2dbf517fc595e33a789df7382697b56f382cc7606b18939e'
 choices=[None]*4096
 for tile in range(160):
  a,b,c,d=[v&7 for v in patterns[tile*4:tile*4+4]]
  for corners,orientation in [((a,b,c,d),0),((b,a,d,c),16),((c,d,a,b),48),((d,c,b,a),32),((b,c,d,a),96),((c,b,a,d),112),((d,a,b,c),80),((a,d,c,b),64)]:
   key=sum(v<<shift for v,shift in zip(corners,[9,6,3,0]))
   if choices[key] is None:choices[key]=(tile,orientation)
 table=bytes(v for choice in choices for v in (choice if choice is not None else (151,0)))
 assert hashlib.sha256(table).hexdigest()=='422f124eac806197ba95d2438516284c3bf828a1db8e8aabe330661fff23ef8a'
for row in range(4):
 for col in range(4):struct.pack_into('<h',job,14+row*18+col*2,3264)
model.viewport=[];point=struct.pack('<hh',77*256+127,50*256+127);model.run(entry=0x753f0,state_before={0x220000+i:b for i,b in enumerate(point)},registers={'EAX':0x220000,'EDX':1,'EBX':1,'ECX':0});reserved_mapbytes=bytes(model.mem[0x11c2be+i]for i in range(229376));data=bytearray(table);expected=bytearray();total_steps=0

import sys
sys.path.insert(0,str(p/'scripts'));from cargo import environment
extra=[p/'private/ghidra/construction-command/instructions-000791c0.txt',p/'private/ghidra/keyboard-event-refs/instructions-00075f60.txt']
extra_pins={'private/ghidra/construction-command/instructions-000791c0.txt': 'da1a6046fa2a0a4153f54ff5f4cce68941a05aae834287ce856ee48a1867f9f2', 'private/ghidra/keyboard-event-refs/instructions-00075f60.txt': 'c238f6c3e2331244305dc67d0f7fcbec6496ae8cbb165da62222da9a8ad1e0cb'}
assert all(hashlib.sha256((p/q).read_bytes()).hexdigest()==v for q,v in extra_pins.items())
extra += [p/'private/ghidra'/n for n in ['terrain-map-refs/instructions-0008c550.txt','terrain-map-refs/instructions-0008bee0.txt','terrain-map-refs/instructions-00084960.txt','terrain-map-refs/instructions-00084b40.txt','construction-command/instructions-0008b840.txt','camera-player-input/instructions-0008b800.txt','terrain-map-refs/instructions-0007b240.txt','entity-order/instructions-0001f440.txt','entity-plant/instructions-0001f240.txt','entity-plant/instructions-0001f280.txt']]
model.code.update(m.m.Model(extra,b'').code)
model.code[0x847d0]=('MOV',['EAX','1']);model.code[0x847d1]=('RET',[])
for pc in [0x8c009,0x8c160]:model.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
order=sorted(model.code);model.next=dict(zip(order,order[1:]));(w/'border.bin').write_bytes(table)
cases=[]
for map_id,current_map in enumerate([mapbytes,reserved_mapbytes]):
 for counter in [1,2,4]:
  for pattern,slots in enumerate([[0],[1,24]]):
   name=f'map{map_id}-counter{counter}-pattern{pattern}';directory=w/name;directory.mkdir();current=bytearray(mission);current[0x355e:0x3b55e]=current_map
   pool=[bytearray([0xa5]*176)for _ in range(25)]
   for record in pool:struct.pack_into('<h',record,12,-1)
   for slot in slots:pool[slot]=bytearray(job);struct.pack_into('<h',pool[slot],12,counter)
   current[0x2ff7]=len(slots);assert current[0x34b8]==0
   (directory/'mission.bin').write_bytes(current);(directory/'pool.bin').write_bytes(b''.join(pool)+bytes([len(slots)]));cases.append(name)
for failure in ['count-mismatch','terrain-list']:
 name='reject-'+failure;directory=w/name;directory.mkdir();current=bytearray(mission);current[0x355e:0x3b55e]=reserved_mapbytes;pool=[bytearray([0xa5]*176)for _ in range(25)]
 for record in pool:struct.pack_into('<h',record,12,-1)
 for slot in [0,24]:pool[slot]=bytearray(job);struct.pack_into('<h',pool[slot],12,1)
 current[0x2ff7]=2
 if failure=='late-height':struct.pack_into('<h',pool[24],14,3265)
 if failure=='count-mismatch':current[0x2ff7]=1
 if failure=='terrain-list':current[0x34b8]=1
 (directory/'mission.bin').write_bytes(current);(directory/'pool.bin').write_bytes(b''.join(pool)+bytes([2]));cases.append(name)
for name,target,remaining in [('steep-slope',9000,2),('water-completion',0,1)]:
 directory=w/name;directory.mkdir();current=bytearray(mission);pool=[bytearray(176)for _ in range(25)]
 for record in pool:struct.pack_into('<h',record,12,-1)
 pool[0]=bytearray(job);struct.pack_into('<hh',pool[0],10,9000,remaining)
 for row in range(4):
  for col in range(4):struct.pack_into('<h',pool[0],14+row*18+col*2,target)
 current[0x2ff7]=1;(directory/'mission.bin').write_bytes(current);(directory/'pool.bin').write_bytes(b''.join(pool)+b'\x01');cases.append(name)
(w/'cases.txt').write_text('\n'.join(cases))
subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=p,capture_output=True,check=True)
target=p/'private/build/rust-target/debug';exe=w/'world.exe';copy=w/'runtime-src';copy.mkdir();copy_pins={}
for original in (p/'rust/crates/genewars-runtime/src').rglob('*.rs'):
 relative=original.relative_to(p/'rust/crates/genewars-runtime/src');dest=copy/relative;dest.parent.mkdir(parents=True,exist_ok=True);raw=original.read_bytes();dest.write_bytes(raw);copy_pins[relative.as_posix()]={'original':hashlib.sha256(raw).hexdigest(),'copy':hashlib.sha256(raw).hexdigest()}
constructor=copy/'actor_world.rs';raw=constructor.read_bytes();needle=b'    fn from_parts(';assert raw.count(needle)==1;changed=raw.replace(needle,b'    pub fn from_parts(');constructor.write_bytes(changed);copy_pins['actor_world.rs']['copy']=hashlib.sha256(changed).hexdigest()
fixture_lib=w/'libgenewars_runtime_fixture.rlib';rustc=str(p/'private/tools/rust/cargo/bin/rustc.exe');cmd=[rustc,'--edition=2021','--crate-name','genewars_runtime','--crate-type','rlib',str(copy/'lib.rs'),'-L',f'dependency={target}/deps','-o',str(fixture_lib)]
for crate in ['assets','core']:cmd+=['--extern',f'genewars_{crate}={target}/libgenewars_{crate}.rlib']
r=subprocess.run(cmd,cwd=p,env=environment(),capture_output=True);assert r.returncode==0,r.stderr.decode(errors='replace')
cmd=[rustc,'--edition=2021','scripts/ghidra/mission_contract_terrain_jobs_fixture.rs','-L',f'dependency={target}/deps','-o',str(exe),'--extern',f'genewars_assets={target}/libgenewars_assets.rlib','--extern',f'genewars_runtime={fixture_lib}']
r=subprocess.run(cmd,cwd=p,env=environment(),capture_output=True);assert r.returncode==0,r.stderr.decode(errors='replace');subprocess.run([str(exe),str(w)],cwd=p,env=environment(),check=True)
results=dict(line.split('\t',1)for line in(w/'results.tsv').read_text().splitlines());ACTORS=343958;CREATURES=ACTORS+4000;BUILDINGS=CREATURES+37200;PLAYERS=BUILDINGS+17152;RNG=PLAYERS+7220;JOBS=RNG+7;total=0;steps=0;events_count=0;height_calls=0;rng_draws=0
ranges=[(ACTORS,4000,0x16360e),(CREATURES,37200,0x1542be),(BUILDINGS,17152,0x1645ae),(PLAYERS,7220,0x118d74)]
for name in cases:
 d=w/name;before=(d/'before.bin').read_bytes();actual=(d/'after.bin').read_bytes();assert before[0x1ec7:0x2ff7]==actual[0x1ec7:0x2ff7],'serialized job priors changed'
 if name.startswith('reject-'):
  reason={'reject-count-mismatch':'CountMismatch','reject-terrain-list':'TerrainObjectRefreshRequired'}[name];assert reason in results[name]and before==actual,(name,results[name]);continue
 assert results[name]=='PASS',(name,results[name]);state={0x118d60+i:b for i,b in enumerate(before[:0x355e])};state.update({0x11c2be+i:b for i,b in enumerate(before[0x355e:0x3b55e])});state.update({0x11ac27+i:b for i,b in enumerate(before[JOBS:JOBS+4400])})
 for start,length,address in ranges:state.update({address+i:b for i,b in enumerate(before[start:start+length])})
 state.update({0x11a9c3+i:b for i,b in enumerate(struct.pack('<h',816))});state.update({0x11ac02+i:0 for i in range(4)});state[0x11c218]=0
 seed,ring,draws=struct.unpack_from('<HBI',before,RNG);state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))});model.viewport=[]
 state.update({0x112d24+i:b for i,b in enumerate(table)})
 report,_,trace=model.run(seed=seed,entry=0x79220,state_before=state,registers={'EAX':0x11ac27});expected=bytearray(before);expected[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376));expected[0x2ff7]=model.load(0x11bd57,1);expected[JOBS:JOBS+4400]=bytes(model.load(0x11ac27+i,1)for i in range(4400));expected[JOBS+4400]=model.load(0x11bd57,1)
 for start,length,address in ranges:expected[start:start+length]=bytes(model.load(address+i,1)for i in range(length))
 struct.pack_into('<HBI',expected,RNG,model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4));ordered=b''.join(struct.pack('<H',v)for v in model.viewport)
 (d/'source-after.bin').write_bytes(expected);assert actual==expected,(name,next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None));assert(d/'events.bin').read_bytes()==ordered,name
 total+=len(actual)+len(ordered);steps+=report['steps'];events_count+=len(model.viewport);height_calls+=trace.count(0x8c550);rng_draws+=report['draws'];print(name,'PASS',flush=True)
for name,digest in copy_pins.items():
 copied=(copy/name).read_bytes();assert hashlib.sha256(copied).hexdigest()==digest['copy']

assert height_calls>0 and rng_draws>0,'height-changing RNG path not covered'
r={'status':'passed','border_table_input':'generated884D0' if generated else 'mapped-zero-prior','border_table_sha256':hashlib.sha256(table).hexdigest(),'height_setter_calls':height_calls,'rng_draws':rng_draws,'cases':len(cases)-2,'rollback_cases':2,'compared_bytes':total,'literal_instructions':steps,'ordered_viewport_requests':events_count,'scope':'Whole79220/78E30 height-changing jobs on actual raw/reservedCONQ4x4site76,49 counters1/2/4 sparse overlapping jobs, including retirement. Actual8C550/8BEE0/8B840, shader andterrainmaintenance; viewport cache847D0 isexplicitport. Externalnative jobs authoritative.','fixture_admission_delta':'Only fn from_parts visibility widened in private runtime copy; production hash admission unchanged','runtime_source_copy_pins':copy_pins,'probe_sha256':hashlib.sha256((p/'scripts/ghidra/mission_contract_terrain_jobs_fixture.rs').read_bytes()).hexdigest(),'validator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'dependency_rlib_sha256':{crate:hashlib.sha256((target/f'libgenewars_{crate}.rlib').read_bytes()).hexdigest()for crate in ['assets','core']},'source_pins':{f.relative_to(p).as_posix():hashlib.sha256(f.read_bytes()).hexdigest()for f in files+extra},'private_run':w.relative_to(p).as_posix()}
(p/('research/campaign/terrain-jobs-height-generated-world.json' if generated else 'research/campaign/terrain-jobs-height-world.json')).write_text(json.dumps(r,indent=2)+'\n');print({k:v for k,v in r.items()if k not in ['runtime_source_copy_pins','source_pins']})
