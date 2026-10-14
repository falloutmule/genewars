"""Literal8C550/8BEE0 ordered numeric height plan, explicit refresh ports."""
from pathlib import Path
import hashlib,importlib.util,json,random,struct,subprocess,uuid
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Source(m.Source):
 events=bytearray()
 def store(self,a,n,v):
  pc=getattr(self,'pc',0)
  if pc in [0x8c596,0x8c01c,0x8c178] and n==2 and 0x11c2be<=a<0x1542be:
   self.events+=struct.pack('<BHHH',1,(a-0x11c2be)//14,self.load(a,2),v&65535)
  if pc in [0x8c009,0x8c160] and a==0x11878a:
   self.events+=struct.pack('<BHHBH',3,self.load(a,2),v&65535,(self.load(0x11872c,1)-1)&31,0x3c1 if pc==0x8c009 else 0x3d8)
  return super().store(a,n,v)
 def get(self,o):
  if self.pc in [0x8c5a2,0x8c023,0x8c17e,0x8c048,0x8c1a0,0x8c5ed,0x8c605,0x8c626,0x8c638] and o.startswith('0x000'):
   if self.pc==0x8c5a2:self.events+=bytes([2])+struct.pack('<I',self.r['EDX'])
   elif self.pc in [0x8c023,0x8c17e]:self.events+=struct.pack('<BHI',4,(self.r['EAX']-0x11c2be)//14,self.r['EDX'])
   elif self.pc in [0x8c048,0x8c1a0]:self.events+=bytes([5])+struct.pack('<I',self.r['EDX'])
   else:self.events+=struct.pack('<BH',6,(self.r['EAX']-0x11c2be)//14)
  return super().get(o)
x=Source();files=[P/'private/ghidra/terrain-map-refs'/f'instructions-000{v}.txt'for v in ['8c550','8bee0']];x.code.update(m.m.Model(files,b'').code)
for pc in [0x84960,0x8c290,0x84b40,0x8b840]:x.code[pc]=('RET',[])
for pc in [0x8c009,0x8c160]:x.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
order=sorted(x.code);x.next=dict(zip(order,order[1:]));rng=random.Random(0x8c550);inputs=bytearray();expected=bytearray();instructions=0
for case in range(64):
 heights=[rng.randrange(0,7000)for _ in range(16384)];index=[0,127,16256,16383,6855][case%5];target=[-32768,-1,0,800,3300,9000,32767][case%7];seed=[0,1,5428,65535][case%4];ring=case%32;draws=(0xffffffff-case)&0xffffffff
 state={0x11c2be+i*14+2+j:b for i,v in enumerate(heights)for j,b in enumerate(struct.pack('<H',v))};state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))})
 x.events=bytearray();x.pc=0;report,_,_=x.run(seed=seed,entry=0x8c550,state_before=state,registers={'EAX':0x11c2be+index*14,'EDX':target&65535});instructions+=report['steps']
 payload=bytes(x.events)+struct.pack('<HBIH',report['rng_seed_after'],report['rng_ring_after'],report['rng_draw_count_after'],x.r['EAX']&65535)
 expected+=struct.pack('<I',len(payload))+payload;inputs+=struct.pack('<HhHBI',index,target,seed,ring,draws)+struct.pack('<16384H',*heights)
work=P/'private/review/terrain-height'/uuid.uuid4().hex;work.mkdir(parents=True);h=work/'review.rs';terrain=P/'rust/crates/genewars-runtime/src/terrain_height.rs';construction=P/'rust/crates/genewars-runtime/src/construction_terrain.rs'
h.write_text(f'#[path="{terrain.as_posix()}"]mod terrain_height;#[path="{construction.as_posix()}"]mod construction_terrain;'+r'''mod blueprint{#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct BlueprintRng{pub seed:u16,pub ring:u8,pub draw_count:u32}}use std::io::{Read,Write};use terrain_height::TerrainHeightStep::*;fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(32779){let w=|i|u16::from_le_bytes(c[i..i+2].try_into().unwrap());let mut map=vec![0u8;229376];for i in 0..16384{map[i*14+2..i*14+4].copy_from_slice(&c[11+i*2..13+i*2]);}let q=terrain_height::plan_height_change(&map,w(0),w(2)as i16,blueprint::BlueprintRng{seed:w(4),ring:c[6],draw_count:u32::from_le_bytes(c[7..11].try_into().unwrap())}).unwrap();let mut p=Vec::new();for step in q.steps{match step{SetHeight(h)=>{p.push(1);p.extend(h.cell.to_le_bytes());p.extend(h.before.to_le_bytes());p.extend(h.after.to_le_bytes());},RefreshVertex{position}=>{p.push(2);for v in position{p.extend(v.to_le_bytes());}},RngDraw{journal_line,seed_before,seed_after,ring_slot}=>{p.push(3);p.extend(seed_before.to_le_bytes());p.extend(seed_after.to_le_bytes());p.push(ring_slot);p.extend(journal_line.to_le_bytes());},RefreshSlopeShade{cell,source_index}=>{p.push(4);p.extend(cell.to_le_bytes());p.extend(source_index.to_le_bytes());},RefreshSlopeViewport{position}=>{p.push(5);for v in position{p.extend(v.to_le_bytes());}},RefreshLinkedObjects{cell}=>{p.push(6);p.extend(cell.to_le_bytes());}}}p.extend(q.rng_after.seed.to_le_bytes());p.push(q.rng_after.ring);p.extend(q.rng_after.draw_count.to_le_bytes());p.extend(q.returned_height_delta.to_le_bytes());out.extend((p.len()as u32).to_le_bytes());out.extend(p);}std::io::stdout().write_all(&out).unwrap();}''')
s=importlib.util.spec_from_file_location('cargo_local',P/'scripts/cargo.py');cargo=importlib.util.module_from_spec(s);s.loader.exec_module(cargo);exe=work/'review.exe';compiler=P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'
subprocess.run([str(compiler),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],env=cargo.environment(),check=True)
a=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert a==expected,(len(a),len(expected),[(i,a,b)for i,(a,b)in enumerate(zip(a,expected))if a!=b][:16])
r={'status':'passed-ordered-terrain-height-plan','cases':64,'compared_event_bytes':len(a),'literal_instructions':instructions,'production_sha256':hashlib.sha256(terrain.read_bytes()).hexdigest(),'slope_sha256':hashlib.sha256(construction.read_bytes()).hexdigest(),'source_pins':{f.relative_to(P).as_posix():hashlib.sha256(f.read_bytes()).hexdigest()for f in files},'ports':['84960 vertex viewport/shade refresh','8C290 slope shade refresh','84B40 slope viewport refresh','8B840 actual linked-object height refresh'],'scope':'Exact mutation/request order, before/after heights, RNG seed/ring/count and signed return lowword; no world refresh implementation claimed.'}
(P/'research/campaign/terrain-height-plan.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
