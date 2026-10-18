"""Full47F80 with literal451F0 incomplete callbacks, over owned-world captures."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,shutil,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
ACTORS=343958;CREATURES=ACTORS+4000;BUILDINGS=CREATURES+37200;PLAYERS=BUILDINGS+17152;RNG=PLAYERS+7220;BINDINGS=RNG+7
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(command):
 r=subprocess.run(command,cwd=P,env=environment(),capture_output=True,timeout=240)
 assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'))
 return r.stdout.decode(errors='replace')
def main():
 work=P/'private/review/building-scheduler'/uuid.uuid4().hex;work.mkdir(parents=True)
 pins={}
 # Freeze the exact implementation compiled here; concurrent production work
 # cannot silently change the meaning of these recorded native captures.
 for crate in ['assets','core','runtime']:
  origin=P/f'rust/crates/genewars-{crate}/src';target=work/f'source/{crate}'
  shutil.copytree(origin,target)
  for f in target.rglob('*.rs'):pins[(origin/f.relative_to(target)).relative_to(P).as_posix()]=sha(f)
 # Fixture-only slot permutation in the private copy: no production API or
 # callback behavior is changed. Rewrite every common map-list reference.
 fixture='''
impl ActorWorld {
 pub fn review_sparse_buildings(&mut self) {
  fn target(slot:usize)->usize{slot/64*64+[1,3,7,15,23,31,47,63][slot%64]}
  fn remap(raw:&mut[u8]){if raw[2]==4{let slot=u16::from_le_bytes([raw[0],raw[1]])as usize;raw[..2].copy_from_slice(&(target(slot)as u16).to_le_bytes());}}
  let pool=self.buildings.as_ref().unwrap();let old=*pool.records();let mut records=old;let mut bindings=[None;256];
  for r in &mut records{if r[6]!=0{r[6]=0;}}
  for(slot,r)in old.iter().enumerate(){if r[6]==0{continue}let next=target(slot);let mut copy=*r;
   copy[15..17].copy_from_slice(&(next as u16).to_le_bytes());remap(&mut copy[..3]);remap(&mut copy[3..6]);
   records[next]=copy;bindings[next]=pool.binding(slot).unwrap();}
  self.buildings=Some(Box::new(crate::building_pool::BuildingPool::admit(records,bindings).unwrap()));
  for cell in self.backing[MAP_START..MAP_START+MAP_BYTES].chunks_exact_mut(14){remap(&mut cell[6..9]);}
  for(kind,layout)in self.layouts.iter().enumerate(){if matches!(kind+1,1|3|4){continue}
   for row in self.backing[layout.file_offset..layout.file_offset+layout.count*layout.packed_stride].chunks_exact_mut(layout.packed_stride){remap(&mut row[..3]);remap(&mut row[3..6]);}}
  for slot in 0..20{let before=*self.actors.record(slot).unwrap();let mut after=before;remap(&mut after[..3]);remap(&mut after[3..6]);if after!=before{self.actors.replace_record_checked(slot,&before,&after).unwrap();}}
  for slot in 0..300{let pool=self.creatures.as_mut().unwrap();let before=*pool.record(slot).unwrap();let mut after=before;remap(&mut after[..3]);remap(&mut after[3..6]);if after!=before{pool.replace_record_checked(slot,&before,&after).unwrap();}}
  self.validate_building_counts().unwrap();self.validate_links().unwrap();
 }
}
'''
 fixture_path=work/'source/runtime/actor_world.rs'
 with fixture_path.open('a',encoding='utf-8',newline='\n')as f:f.write(fixture)
 (work/'sparse-fixture.rs').write_text(fixture,encoding='utf-8')
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  command=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(work/f'source/{crate}/lib.rs'),'-L',f'dependency={work}','-o',str(work/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:command+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
  run(command)
 command=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={work}','-o',str(work/'probe.exe')]
 for dep in ['assets','runtime']:command+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
 run(command);print(run([str(work/'probe.exe'),str(P),str(work)]),flush=True)
 spec=importlib.util.spec_from_file_location('initial_source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 class Source(m.Source):
  selected=0
  def get(self,operand):
   if operand=='byte ptr [0x0011873d]':return self.selected
   return super().get(operand)
 model=Source();files=[P/'private/ghidra'/f for f in ['mission-array-refs/instructions-00047f80.txt','mission-array-refs/instructions-000437a0.txt','building-update/instructions-000451f0.txt','landing-contact/instructions-0007ac00.txt']]
 model.code.update(m.m.Model(files,b'').code)
 # Presentation services preserve their source calling convention.451F0,
 # 7AC00,47F80 and437A0 numeric bodies remain literal.
 for address,cleanup in [(0x981d3,0),(0x41ce0,20),(0x41910,16),(0x984c0,0)]:model.code[address]=('RET',[hex(cleanup)]if cleanup else[])
 for address in [0x47e21,0x47f34]:model.code[address]=('CALL',['0x2e0010'])
 model.code[0x2e0010]=('RET',[]);order=sorted(model.code);model.next=dict(zip(order,order[1:]))
 resource=P/'private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB';assert sha(resource)=='c26df60af3a3166aec54112f197a4334966eb214eeaa48e0d9284e0510b8cd20'
 anb=bytearray(resource.read_bytes());frames,animations=struct.unpack_from('<II',anb,10)
 for i in range(5):
  for offset in [0,4]:struct.pack_into('<I',anb,animations+i*10+offset,0x620000+frames+struct.unpack_from('<I',anb,animations+i*10+offset)[0])
 cases=[];total=steps=0;stream=hashlib.sha256()
 for number,line in enumerate((work/'cases.tsv').read_text().splitlines()):
  name,count,selected,callbacks=line.split('\t');count=int(count);model.selected=int(selected);callbacks=int(callbacks);folder=work/name
  before=(folder/'before.bin').read_bytes();actual=(folder/'after.bin').read_bytes();assert len(before)==len(actual)==BINDINGS+256
  state={0x118d60+i:b for i,b in enumerate(before[:0x355e])};state.update({0x11c2be+i:b for i,b in enumerate(before[0x355e:0x3b55e])})
  for kind,(start,n,stride,address,width)in enumerate(LAYOUTS,1):
   for slot in range(n):
    base={1:CREATURES,3:ACTORS,4:BUILDINGS}.get(kind)
    if base is None:
     raw=before[start+slot*stride:start+(slot+1)*stride];native=bytearray(raw[:31]+bytes(8)+raw[31:])
    else:native=bytearray(before[base+slot*width:base+(slot+1)*width])
    if kind==4:
     struct.pack_into('<I',native,35,0x112120)
     if native[6]!=0:struct.pack_into('<I',native,31,0x620000+animations)
    state.update({address+slot*width+j:b for j,b in enumerate(native)})
  state.update({0x118d74+j:b for j,b in enumerate(before[PLAYERS:RNG])});state.update({0x620000+j:b for j,b in enumerate(anb)});state[0x11878c]=count
  seed,ring,draws=struct.unpack_from('<HBI',before,RNG);state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))})
  report,_,trace=model.run(seed=seed,state_before=state,entry=0x47f80,stack_fill=[0,85,255][number%3])
  expected=bytearray(before);expected[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
  for kind,(start,n,stride,address,width)in enumerate(LAYOUTS,1):
   for slot in range(n):
    native=bytearray(model.load(address+slot*width+j,1)for j in range(width));base={1:CREATURES,3:ACTORS,4:BUILDINGS}.get(kind)
    if base is None:expected[start+slot*stride:start+(slot+1)*stride]=native[:31]+native[39:]
    else:
     native[31:39]=before[base+slot*width+31:base+slot*width+39];expected[base+slot*width:base+(slot+1)*width]=native
  expected[PLAYERS:RNG]=bytes(model.load(0x118d74+i,1)for i in range(7220));struct.pack_into('<HBI',expected,RNG,model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
  assert expected==actual,(name,[(i,a,b)for i,(a,b)in enumerate(zip(expected,actual))if a!=b][:16])
  assert trace.count(0x451f0)==callbacks,(name,'callback order/count');assert trace.count(0x437a0)==count
  (folder/'source-after.bin').write_bytes(expected);stream.update(before+actual);total+=len(actual);steps+=report['steps']
  cases.append(dict(case=name,player_count=count,selected=model.selected,callbacks=callbacks,literal_instructions=report['steps']));print(name,'PASS',flush=True)
 receipt=dict(status='passed-bounded-incomplete-building-scheduler',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=total,literal_instructions=steps,implementation_pins=pins,
  listing_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files},review_pins={p.relative_to(P).as_posix():sha(p)for p in [Path(__file__),Path(__file__).with_suffix('.rs'),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py')]},
  private_run=work.relative_to(P).as_posix(),probe_sha256=sha(work/'probe.exe'),resource_sha256=sha(resource),fixture_stream_sha256=stream.hexdigest(),original_execution=False,gui_execution=False,
  fixture_sha256=sha(work/'sparse-fixture.rs'),
  scope='64 complete owned-world47F80 passes with literal451F0/7AC00/437A0,player counts0/1/2/4,all selectedplayers,dense and sparse0/1/3/8 buildings including slot63, and incomplete progress0/1/62/119. Full backing,native actors/creatures/buildings,players,RNG,bindings compared. Sparse fixtures permute initial slots and intrusive links in a private-only helper; callback implementation is unchanged. Audio,queue,format/UI consumers explicit ports. Newborn/selector rejection checked. Does not admit destruction,completed buildings,newborn scheduling or full gameplay.')
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');tmp.replace(out)
 print(json.dumps({k:receipt[k]for k in ['status','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
