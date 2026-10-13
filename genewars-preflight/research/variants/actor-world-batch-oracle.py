"""Independent byte oracle for the explicit no-associated-unit five-slot batch.

This is not an original startup/landing model. Graph membership and sampled
environment values are fixture inputs. Native ActorWorld comparison is separate.
"""
from pathlib import Path
import copy,hashlib,json,struct
P=Path(__file__).resolve().parents[2]
ACTOR_SIZE=200;PLAYER_SIZE=1805;MAP_SIZE=229376
def word(v):return (v+32768)%65536-32768
def read16(b,i):return struct.unpack_from('<h',b,i)[0]
def read32(b,i):return struct.unpack_from('<I',b,i)[0]
def triple(index,kind):return struct.pack('<hB',index,kind)
def reference(raw):return None if raw[2]==0 else (raw[2],read16(raw,0))
def candidate(seed,center,tables):
 new=seed*58925%65536;angle=(new//256)*8;ix=(angle+1536)%2048
 dx=(1024*tables[1][ix])//65536;dy=(1024*tables[0][ix])//65536
 return new,angle,[(word(2*c+d)//2)%32768 for c,d in zip(center,[dx,dy])]
def lookup(state,ref):
 kind,index=ref
 records=state['actors'] if kind==3 else state['foreign'].get(kind,[])
 if index<0 or index>=len(records):raise ValueError('family-bound')
 record=records[index]
 if record[6]!=kind or read16(record,15)!=index:raise ValueError('resolved-identity')
 return record
def membership(state):
 seen=set()
 for cell in range(16384):
  raw=state['map'][cell*14+6:cell*14+9];ref=reference(raw);prior=None
  while ref is not None:
   if ref in seen:raise ValueError('duplicate-or-cycle')
   seen.add(ref);record=lookup(state,ref);prev=record[:3]
   if prior is None:
    if prev[2]!=0 or read16(prev,0)!=cell:raise ValueError('head-previous')
   elif reference(prev)!=prior:raise ValueError('interior-previous')
   prior=ref;ref=reference(record[3:6])
 return seen
def activate(before,player_id,tables,environments,attempt_cap=4096):
 """Return complete independent after-state; never mutate before on failure.

 environments is five explicit sequences of (class,weight). Every consumed
 sample advances seed/ring/count, including rejected samples. Unsupported
 work raises ValueError before an after-state is returned.
 """
 if not 0<=player_id<4:raise ValueError('player-bound')
 s=copy.deepcopy(before);player=s['players'][player_id]
 if len(player)!=PLAYER_SIZE or player[1]!=player_id:raise ValueError('player-identity')
 if player[0x498]==5:raise ValueError('associated-type5')
 if not 0<=s['rng']['ring']<32:raise ValueError('rng-ring')
 center=list(struct.unpack_from('<hh',player,9))
 if any(x<0 or x>32767 for x in center):raise ValueError('center-domain')
 linked=membership(s);trace=[]
 for slot in range(5):
  index=player_id*5+slot;record=s['actors'][index]
  flags=struct.unpack_from('<H',record,112)[0]
  if record[6]==0 or not flags&1024:continue
  if record[6]!=3 or read16(record,15)!=index:raise ValueError('actor-identity')
  if (3,index) in linked:raise ValueError('already-linked')
  selector=record[116]
  if selector>=32:raise ValueError('blueprint-selector')
  bp=0x16+36*selector;accepted=None
  for attempt,(terrain,weight) in enumerate(environments[slot]):
   if attempt>=attempt_cap:raise ValueError('attempt-cap')
   if terrain not in range(8) or weight<0:raise ValueError('environment-domain')
   seed_before=s['rng']['seed'];ring_before=s['rng']['ring']
   next_seed,angle,point=candidate(seed_before,center,tables)
   s['rng']['seed']=next_seed;s['rng']['ring']=(ring_before+1)%32
   s['rng']['draw_count']=(s['rng']['draw_count']+1)%2**32
   ok=not record[83]&(1<<terrain) and weight<10
   trace.append(dict(slot=slot,seed_before=seed_before,seed_after=next_seed,ring_before=ring_before,draw_count_after=s['rng']['draw_count'],angle=angle,point=point,accepted=ok))
   if ok:accepted=point;break
  if accepted is None:raise ValueError('candidate-exhaustion')
  x,y=accepted;cell=y//256*128+x//256;head=s['map'][cell*14+6:cell*14+9];old=reference(head)
  if old is not None:
   previous_head=lookup(s,old);previous_head[:3]=triple(index,3)
   record[3:6]=triple(read16(previous_head,15),previous_head[6])
  else:record[5]=0
  record[:3]=triple(cell,0);s['map'][cell*14+6:cell*14+9]=triple(index,3);linked.add((3,index))
  struct.pack_into('<HH',record,7,x,y);struct.pack_into('<H',record,112,(flags&0xfbfe)|1);record[22]&=0xf7
  player[bp+27]=player[bp+24];player[bp+32]|=2
  count=(read32(player,bp+28)+1)%2**32;struct.pack_into('<I',player,bp+28,count)
  if count==1 and player[bp+22]==1:record[114]=40
 struct.pack_into('<I',player,0x49d,(read32(player,0x49d)+1)%2**32)
 membership(s)
 return s,trace
def fixture(player_id,eligible_mask,seed=0,count=0):
 players=[bytearray((i*13+owner*41)%256 for i in range(PLAYER_SIZE))for owner in range(4)]
 for owner,b in enumerate(players):b[1]=owner;b[0x498]=0;struct.pack_into('<hh',b,9,20736,13568)
 actors=[bytearray((i*37+index*19)%256 for i in range(ACTOR_SIZE))for index in range(20)]
 for index,b in enumerate(actors):b[6]=0;struct.pack_into('<h',b,15,index);b[2]=b[5]=0
 for slot in range(5):
  index=player_id*5+slot;b=actors[index];b[6]=3;b[83]=1;b[116]=0
  struct.pack_into('<H',b,112,1024 if eligible_mask&(1<<slot)else 0)
 bp=0x16;b=players[player_id];b[bp+22]=1;b[bp+24]=97;struct.pack_into('<I',b,bp+28,count);struct.pack_into('<I',b,0x49d,0xffffffff)
 terrain=bytearray((i*11+71)%256 for i in range(MAP_SIZE))
 # All raw null indices deliberately remain nonzero and often signednegative.
 for cell in range(16384):terrain[cell*14+8]=0
 return dict(players=players,actors=actors,map=terrain,foreign={},rng=dict(seed=seed,ring=31,draw_count=0xffffffff))
def state_bytes(s):
 foreign=b''.join(bytes(b)for kind in sorted(s['foreign'])for b in s['foreign'][kind])
 return b''.join(bytes(x)for x in s['players'])+b''.join(bytes(x)for x in s['actors'])+bytes(s['map'])+foreign+struct.pack('<HBI',s['rng']['seed'],s['rng']['ring'],s['rng']['draw_count'])
def main():
 proof=json.loads((P/'research/ghidra/actor-activation-evidence.json').read_bytes())
 for r in proof['instruction_contracts']:assert hashlib.sha256((P/r['private_path']).read_bytes()).hexdigest()==r['sha256']
 data=(P/'private/ghidra/validated/memory-.object3.bin').read_bytes();tables=[struct.unpack_from('<2048i',data,x)for x in [0x7ee8,0x86e8]]
 digest=hashlib.sha256();cases=0;draws=0
 for player in range(4):
  for mask in range(32):
   for count in [0,1,0xffffffff]:
    before=fixture(player,mask,count=count);raw=state_bytes(before);after,trace=activate(before,player,tables,[[(1,9)]]*5)
    assert state_bytes(before)==raw
    eligible=[player*5+slot for slot in range(5)if mask&(1<<slot)]
    assert len(trace)==len(eligible)and after['rng']['draw_count']==(0xffffffff+len(eligible))%2**32
    if eligible:
     point=trace[-1]['point'];cell=point[1]//256*128+point[0]//256
     chain=[];ref=reference(after['map'][cell*14+6:cell*14+9])
     while ref is not None:chain.append(ref[1]);ref=reference(lookup(after,ref)[3:6])
     assert chain==eligible[::-1]
     for ordinal,index in enumerate(eligible):
      special=(count+ordinal+1)%2**32==1
      assert after['actors'][index][114]==(40 if special else before['actors'][index][114])
      assert after['actors'][index][27]==before['actors'][index][27]
    assert read32(after['players'][player],0x49d)==0
    digest.update(state_bytes(after));cases+=1;draws+=len(trace)
 # A rejection after earlier staged success must not leak any state.
 rejected=0
 for slot in range(5):
  before=fixture(0,31);raw=state_bytes(before);env=[[(1,9)]]*5;env[slot]=[(1,10)]
  try:activate(before,0,tables,env)
  except ValueError as e:assert str(e)=='candidate-exhaustion'
  else:raise AssertionError('rejection expected')
  assert state_bytes(before)==raw;rejected+=1
 receipt=dict(status='passed-independent-synthetic-batch-oracle-selfcheck',native_actor_world_compared=False,cases=cases,candidate_draws=draws,late_rejection_cases=rejected,after_state_sha256=digest.hexdigest(),original_execution=False,gui_execution=False,limits=['No original startup/landing route','Synthetic graph fixtures; no native batch comparison yet','Explicit no-associated-unit path only'])
 target=Path(__file__).with_suffix('.json');target.write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
if __name__=='__main__':main()
