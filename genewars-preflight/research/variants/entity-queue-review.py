"""Independent pointer-chain queue and pure packet comparisons; no game execution."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,itertools,json,random,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
WORK=ROOT/'private/review/entity-queue'
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def signed(v,bits):return(v+2**(bits-1))%2**bits-2**(bits-1)
def chain(active,count,margin,points):
 if count<0 or count>256 or margin not in [0,128]:return None
 heads=[None]*480;links=[];nodes=[]
 if active:
  for id,x,y,shade,flags in points:
   if count==256:break
   if not(-40<=x<680-margin and -10<=y<520):continue
   # Linked representation follows the recorded special-bucket insertion;
   # no sort/key equivalent or production bucket vectors are used.
   bucket=0 if y<=0 else 479 if y>=480 else y;residual=y if y<=0 else y-480 if y>=480 else 0
   new=len(nodes);nodes.append((id,x,bucket+residual,y,bucket,residual,shade,flags));links.append(None)
   if 0<y<480:links[new]=heads[bucket];heads[bucket]=new
   else:
    previous=None;current=heads[bucket]
    while current is not None and nodes[current][5]<=residual:previous=current;current=links[current]
    links[new]=current
    if previous is None:heads[bucket]=new
    else:links[previous]=new
   count+=1
 result=[]
 for head in heads:
  while head is not None:result.append(nodes[head]);head=links[head]
 return count,result
def main():
 WORK.mkdir(parents=True,exist_ok=True);source=ROOT/'rust/crates/genewars-core/src/entity_draw_queue.rs';sourcepin=pin(source);evidence=ROOT/'research/ghidra/entity-queue-evidence.json';proof=json.loads(evidence.read_bytes());instructionpins=[]
 for c in proof['instruction_contracts']:r=pin(ROOT/c['private_path']);assert r['sha256']==c['sha256'];instructionpins.append(r)
 code=(ROOT/'private/ghidra/validated/memory-.object1.bin').read_bytes();data=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes()
 assert struct.unpack_from('<I',code,0x83120-0x10000)[0]==0x8355b
 assert [struct.unpack_from('<I',data,0x1120ac+off-0x100000)[0] for off in [4,0x6c,0x70]]==[0x1f440,0x1f280,0x1e760]
 texts={Path(c['private_path']).name:(ROOT/c['private_path']).read_text() for c in proof['instruction_contracts']}
 for line in ['000832b9 JGE 0x000832f8','00083257 MOV word ptr [ESI + 0x4],DX','0008338d MOV dword ptr [EAX + 0xa3b],ESI','00083566 JLE 0x00083577','000835a0 OR byte ptr [EBP + 0x15],0x8','000836ed MOV byte ptr [EAX + 0x11d8],0x1']:assert line in texts['instructions-00083140.txt']
 for line in ['000839d3 AND byte ptr [EAX + 0x15],0xf3','00083a74 CMP byte ptr [ESI + 0x11d5],0x0','00083a80 TEST byte ptr [EAX + 0x15],0x8','00083aca MOV word ptr [ESI + 0x11bb],0x0']:assert line in texts['instructions-00083930.txt']
 level=ROOT/'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ001.DAT';b=level.read_bytes();assert hashlib.sha256(b).hexdigest()=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
 own=importlib.util.spec_from_file_location('own_height',Path(__file__).with_name('terrain-review.py'));height=importlib.util.module_from_spec(own);own.loader.exec_module(height)
 points=[]
 for i in range(512):
  at=0x43d4e+i*41
  if not b[at+6]:continue
  x,y=struct.unpack_from('<hh',b,at+7);ix,iy=x//256%128,y//256%128
  h=[struct.unpack_from('<h',b,0x355e+(((iy+dy)%128)*128+(ix+dx)%128)*14+2)[0] for dx,dy in [(0,0),(1,0),(0,1),(1,1)]];h2=height.weighted(h,x%256,y%256);dx=(x-66*256+16384)%32768-16384;dy=(y-48*256+16384)%32768-16384
  points.append((i,(dx-dy)//8-128,(dx+dy)//16-h2//64,b[at+0x17],b[at+0x15]))
 assert len(points)==40
 actual=chain(True,0,128,points);fixture=ROOT/proof['actual_fixture']['private_path'];assert pin(fixture)['sha256']==proof['actual_fixture']['sha256'];reported=json.loads(fixture.read_bytes());assert actual[1]==[(v['id'],v['x_i16'],v['draw_y_i16'],v['projected_y'],v['bucket'],v['residual_i16'],0,4) for v in reported]
 cases=[];categories={}
 def add(name,items):start=len(cases);cases.extend(items);categories[name]=len(cases)-start
 def p(id,x,y):return(id&0xffffffff,x,y,(id*43+7)%256,(id*13+128)%256)
 ys=[-1,0,1,1,-1,480,479,480,519,479,0];synthetic=[p(i,0,y) for i,y in enumerate(ys)]
 proof_order=chain(True,0,128,synthetic)[1]
 assert [q[:6] for q in proof_order]==[(v['id'],v['x_i16'],v['draw_y_i16'],v['projected_y'],v['bucket'],v['residual_i16']) for v in proof['synthetic_order_vector']['output']]
 for v in proof['packet_vectors']:
  flags=v['flags_before'];shade=v['shade_u8'];countdown=v['countdown_i8'];after=flags if shade==0 else flags|4 if countdown>0 else flags
  if shade and after&4:after|=8
  assert after==v['flags_after_admission'] and bool(shade)==v['raises_context_gate']
  for mode in [0,1]:
   draws=bool(shade) and(mode!=0 or bool(after&8));assert v[f'mode{mode}_draw_packet']==dict(draw=draws,dirty_class=(3 if after&4 else 1) if draws else None,flags_after=after&243 if draws else after)
 add('proof_and_actual_vectors',[(True,0,128,synthetic),(True,0,128,points),(True,0,0,points),(False,7,128,points)])
 add('all_signed_word_coordinate_axes',((True,0,v%2*128,[p(v,signed(v,16),479),p(v^0xffffffff,0,signed(v,16))]) for v in range(65536)))
 add('all_high_word_coordinates',((True,0,v%2*128,[p(v,signed((v<<16)|23,32),479),p(v^0xffffffff,0,signed((v<<16)|479,32))]) for v in range(65536)))
 extremes=[-2**31,-2**31+1,-65537,-65536,-32769,-32768,-41,-40,-39,-11,-10,-9,-1,0,1,478,479,480,481,519,520,521,551,552,553,599,600,639,640,679,680,681,32767,32768,65535,65536,2**31-2,2**31-1]
 add('signed32_boundary_crossproducts',((active,count,margin,[p(0xffffffff,x,y)]) for x,y in itertools.product(extremes,repeat=2) for active,count,margin in [(True,0,0),(True,255,128),(True,256,128),(False,0,128)]))
 add('all_initial_count_words',((bool(v%2),signed(v,16),128,synthetic) for v in range(65536)))
 add('all_margin_words',((bool(v%2),0,v,synthetic) for v in range(65536)))
 add('mixed_bottom_bucket_permutations',((True,0,128,[p(i,1,y) for i,y in perm]) for perm in itertools.permutations(enumerate([479,480,479,480,519,-1,0]))))
 sequence=[p(i,-41 if i%7==0 else 0,520 if i%11==0 else ys[i%len(ys)]) for i in range(1024)]
 add('every_capacity_reservation',((active,count,margin,sequence) for count in range(257) for active,margin in [(True,0),(True,128),(False,128)]))
 rng=random.Random(0x8314083930)
 random_cases=[]
 for trial in range(4096):
  length=rng.choice([0,1,2,17,255,256,257,512]);q=[p(rng.choice([0,1,2**32-1,rng.randrange(2**32)]),rng.choice(extremes),rng.choice(ys+[rng.randrange(-10,520)])) for _ in range(length)];random_cases.append((rng.choice([True,False]),rng.choice([0,1,127,255,256]),rng.choice([0,128]),q))
 add('mixed_random_explicit_sequences',random_cases)
 with (WORK/'queues.bin').open('wb') as out:
  out.write(struct.pack('<I',len(cases)))
  for active,count,margin,queue in cases:
   out.write(struct.pack('<BhHH',active,count,margin,len(queue)))
   for q in queue:out.write(struct.pack('<IiiBB',*q))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=WORK/'review.exe';r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(WORK)],capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace')
 raw=(WORK/'results.bin').read_bytes();at=0;accepted=invalid=entries=0
 for number,c in enumerate(cases):
  expected=chain(*c);status=raw[at];at+=1
  if expected is None:assert status==1,number;invalid+=1;continue
  assert status==0,number;count,length=struct.unpack_from('<HH',raw,at);at+=4;rows=[struct.unpack_from('<IhhiHhBB',raw,at+i*18) for i in range(length)];at+=length*18;assert(count,rows)==expected,(number,c[:3],rows,expected);accepted+=1;entries+=length
 draw_cases=0
 for flags in range(256):
  for mode in range(256):
   for gate in [False,True]:
    draws=gate and(mode!=0 or bool(flags&8));expected=bytes([draws,3 if draws and flags&4 else 1 if draws else 0,flags&243 if draws else flags]);assert raw[at:at+3]==expected;at+=3;draw_cases+=1
 def admission(type,context,gate,shade,countdown,flags,old):
  if type!=2 or context%2:return bytes(5)
  if not shade:return bytes([1,gate,flags,0,0])
  f=flags|4 if countdown>0 else flags;dirty=bool(f&4);return bytes([1,1,f|8 if dirty else f,dirty,dirty and old])
 admission_cases=0
 def check(args):
  nonlocal at,admission_cases
  expected=admission(*args);assert raw[at:at+5]==expected,args;at+=5;admission_cases+=1
 for flags in range(256):
  for shade in range(256):
   for countdown in [-128,-1,0,1,127]:
    for gate,old in itertools.product([False,True],repeat=2):check((2,0,gate,shade,countdown,flags,old))
 for countdown in range(-128,128):
  for flags in range(256):
   for shade in [0,1,255]:
    for gate,old in itertools.product([False,True],repeat=2):check((2,0,gate,shade,countdown,flags,old))
 for type in range(256):
  for context in range(256):
   for gate,old in itertools.product([False,True],repeat=2):check((type,context,gate,255,127,129,old))
 assert at==len(raw) and pin(source)==sourcepin
 result=dict(status='passed-independent-native-entity-queue-review',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,source=sourcepin,evidence=pin(evidence),instruction_pins=instructionpins,queue_cases=len(cases),categories=categories,valid_queue_results=accepted,rejected_queue_requests=invalid,ordered_entries=entries,draw_packet_cases=draw_cases,admission_packet_cases=admission_cases,actual_source_points=40,actual_admitted_points=len(actual[1]),source_mission=pin(level),native_output_sha256=hashlib.sha256(raw).hexdigest(),limits=['Explicit family1 sequence only; no full scene/live state inferred','Initial count reserves capacity only; no merge/traversal of existing other-family chains is modeled','Queue helper does not execute dirty prepass, draw callbacks or clear real global context','Admission packet only type2/context selection bit clear; draw packet selectedNone/family predicate true','Full signed32 projected inputs checked before narrowing; negative original counts excluded safely'])
 result['review_sources']=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))];result['native_harness']=pin(exe);result['private_cases']=pin(WORK/'queues.bin')
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n',encoding='utf8');tmp.replace(out);print(json.dumps({k:result[k] for k in ['status','queue_cases','ordered_entries','draw_packet_cases','admission_packet_cases']}))
if __name__=='__main__':main()
