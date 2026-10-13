"""Primary-listing and exact-mission environment contract audit. No emulation."""
from pathlib import Path
from datetime import datetime,timezone
import collections,hashlib,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
LAYOUTS=[(243038,300,116),(277838,512,41),(298830,20,140),(301630,256,59),(316734,200,73),(331334,32,41),(332646,32,89),(335494,16,94)]
def sha(b):return hashlib.sha256(b).hexdigest()
def cell(x,y):return ((y%128)*128+x%128)*14
def terrain_class(region,point):
 x,y=point;gx,gy=x//256,y//256
 value=region[cell(gx+(x%256>=128),gy+(y%256>=128))+5]&7
 if value:return value
 return int(any(region[cell(gx+dx,gy+dy)+5]&7 for dx,dy in [(-1,-1),(0,-1),(1,-1),(-1,0),(1,0),(-1,1),(0,1),(1,1)]))
def literal_terrain(region,point):
 # Independent byte/high-word form of the direct instruction paths.
 x,y=[n&65535 for n in point];xl,yl=x&255,y&255;xh,yh=x>>8,y>>8
 if xl<128:
  at=cell(xh,yh if yl<128 else (yh+1)&255)
 else:at=cell((xh+1)&255,yh if yl<128 else (yh+1)&255)
 value=region[at+5]&7
 if value:return value
 xm=((x-256)&65535)>>8;xp=((xh+1)&255);ym=((y-256)&65535)>>8;yp=((yh+1)&255)
 for a,b in [(xm,ym),(xh,ym),(xp,ym),(xm,yh),(xp,yh),(xm,yp),(xh,yp),(xp,yp)]:
  if region[cell(a,b)+5]&7:return 1
 return 0
def rawref(b,o):return struct.unpack_from('<hB',b,o)
def native_field(source,kind,index,offset):
 # Generic family loader copy [0,31)->[0,31),[31,..)->[39,..).
 # Actor type3 is a separate native overlay and must never use this accessor.
 if kind==3:raise ValueError('native-actor-overlay-required')
 base,count,stride=LAYOUTS[kind-1]
 if not 0<=index<count:raise ValueError('family-bound')
 if 31<=offset<39:raise ValueError('unserialized-native-gap')
 packed=offset if offset<31 else offset-8
 if packed>=stride:raise ValueError('native-field-bound')
 return source[base+index*stride+packed]
def initial_blocker(source,cell_index):
 # Current pinned mission has only kind1 and kind2 reachable. The known
 # type1 +10 predicate is pure byte4A mask04; type2 +10 alwaysfalse.
 index,kind=rawref(source,0x355e+cell_index*14+6);seen=set();weight=0
 while kind:
  if (kind,index)in seen:raise ValueError('cycle')
  seen.add((kind,index));base,count,stride=LAYOUTS[kind-1]
  if not 0<=index<count:raise ValueError('family-bound')
  pos=base+index*stride
  assert source[pos+6]==kind and struct.unpack_from('<h',source,pos+15)[0]==index
  if kind==1:ignored=bool(native_field(source,kind,index,0x4a)&4)
  elif kind==2:ignored=False
  else:raise ValueError('virtual-predicate-not-in-initial-fixture')
  if not ignored and kind==2 and not native_field(source,kind,index,0x2e)&8:weight+=10
  index,kind=rawref(source,pos+3)
 return weight
def minimap(origin,base,point):
 dx,dy=point[0]-origin[0],point[1]-origin[1]
 if not 0<=dx<88 or not 0<=dy<89:return None
 delta=dx-43;v=dy+6-(delta//2);u=delta+v
 out=[((u+base[0])<<8)&65535,((v+base[1])<<8)&65535]
 return out,((out[1]>>8)&127)*128+((out[0]>>8)&127)
def main():
 files=['terrain-map-refs/instructions-0008b520.txt','terrain-map-refs/instructions-0008b130.txt','actor-activation/instructions-00074cc0.txt','actor-activation/instructions-00074160.txt','command-resolver/instructions-00074270.txt','landing-contact/instructions-0008b460.txt','actor-activation/instructions-0001ee10.txt','actor-activation/instructions-0001ee20.txt','actor-activation/instructions-0001f410.txt','landing-contact/instructions-000874b0.txt','plant-command/instructions-00074a50.txt','mission-tail-constructors/instructions-0001e450.txt']
 pins=[dict(private_path='private/ghidra/'+s,sha256=sha((G/s).read_bytes()))for s in files]
 sourcepath=P/'private/integration/game-runtime/terrain/CONQ001.DAT';source=sourcepath.read_bytes();assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
 region=source[0x355e:0x355e+229376];hist=collections.Counter();cases=0
 for y in range(128):
  for x in range(128):
   for fx,fy in [(0,0),(127,127),(128,0),(0,128),(128,128),(255,255),(127,128),(128,127),(255,0)]:
    point=[x*256+fx,y*256+fy];a=terrain_class(region,point);assert a==literal_terrain(region,point);hist[a]+=1;cases+=1
 for x,y in [(0,0),(127,127),(-1,-1),(63,64)]:
  for fx in range(256):
   for fy in range(256):
    point=[x*256+fx,y*256+fy];assert terrain_class(region,point)==literal_terrain(region,point);cases+=1
 synthetic=bytearray(229376);synthetic[cell(0,0)+5]=2;assert terrain_class(synthetic,[128,0])==0
 synthetic[cell(-1,-1)+5]=7;assert terrain_class(synthetic,[128,0])==1
 weights=collections.Counter(initial_blocker(source,i)for i in range(16384));assert sum(k*v for k,v in weights.items())==400
 assert all(not region[i*14+11]&1 for i in range(16384))
 memory=(G/'validated/memory-.object3.bin').read_bytes();vtable=[]
 for kind,base,expected in [(1,0x111fec,0x1ee20),(2,0x1120ac,0x1f410),(3,0x112194,0x1ee20),(5,0x112254,0x1f410)]:
  raw=memory[base-0x100000+16:base-0x100000+20];assert struct.unpack('<I',raw)[0]==expected;vtable.append(dict(kind=kind,vtable_va=hex(base),slot=16,target=hex(expected),sha256=sha(raw)))
 raw=memory[0x12194+92:0x12194+96];assert struct.unpack('<I',raw)[0]==0x1ee10
 order=memory[0x60b8:0x60b8+9];offsets=[struct.unpack_from('<hh',memory,0x4564+4*i)for i in order];assert offsets[-1]==(7680,0)
 mmcases=0
 for base in [[0,0],[127,127],[255,255]]:
  for y in range(89):
   for x in range(88):
    result=minimap([10,20],base,[10+x,20+y]);assert result is not None;point,ci=result;assert all(n%256==0 for n in point)and 0<=ci<16384;mmcases+=1
 for point in [[9,20],[10,19],[98,20],[10,109]]:assert minimap([10,20],[0,0],point)is None
 result=dict(status='passed-static-actor-environment-contract',utc=datetime.now(timezone.utc).isoformat(),native_implementation_compared=False,original_execution=False,gui_execution=False,instruction_pins=pins,mission=dict(private_path=sourcepath.relative_to(P).as_posix(),sha256=sha(source)),terrain_cases=cases,terrain_actual_histogram=dict(hist),blocker_cells=16384,initial_blocker_histogram=dict(weights),map_obstacle_flag_clear_cells=16384,typed_virtual_slots=vtable,obstacle_neighbor_order=list(order),obstacle_resolved_offsets=offsets,minimap_vectors=mmcases,minimap_outside_vectors=4,limits=['Current map bit11mask01 clear proves obstacleNULL for all existing owned states that preserve it','Nonclear obstacle-flag path and virtual-kind8 are not admitted by this first-mission contract','Native actor+7F/category1 exemption requires typed actor/blueprint and actual live plant identity','874B0 is minimap-only, no true terrain unprojection claim'])
 out=Path(__file__).with_suffix('.json');out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k]for k in ['status','terrain_cases','blocker_cells','initial_blocker_histogram','map_obstacle_flag_clear_cells','minimap_vectors']}))
if __name__=='__main__':main()
