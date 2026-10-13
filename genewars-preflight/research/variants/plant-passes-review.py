"""Independent indexed8 RLE/foreground/shadow oracle; never executes original code."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
WORK=ROOT/'private/review/plant-passes'
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def signed(x):return (x+32768)%65536-32768
def decode(b,record,sections):
 off,w,h=struct.unpack_from('<IBB',b,sections[0]+record*6);start=18+off;pos=start;indices=bytearray(w*h);mask=bytearray(w*h)
 if w and h:
  for y in range(h):
   x=0
   while True:
    assert pos<sections[0];n=struct.unpack_from('b',b,pos)[0];pos+=1
    if n==0:break
    if n<0:x-=n
    else:
     assert pos+n<=sections[0] and x+n<=w
     indices[y*w+x:y*w+x+n]=b[pos:pos+n];mask[y*w+x:y*w+x+n]=bytes([1])*n;pos+=n;x+=n
    assert x<=w
 return dict(w=w,h=h,indices=bytes(indices),mask=bytes(mask),consumed=pos-start if w and h else 0)
def frame(b,animation,selector):
 s=struct.unpack_from('<4I',b,2);first,last=struct.unpack_from('<II',b,s[3]+animation*10);fo=first+selector*14;assert fo<=last
 component_offset,w,h=struct.unpack_from('<Ihh',b,s[2]+fo);count=b[s[2]+fo+11];components=[]
 for c in range(count):
  sprite,x,y,flags,_=struct.unpack_from('<IhhBB',b,s[1]+component_offset+c*10);assert flags==0 and sprite%6==0
  components.append(dict(x=x//2,y=y//2,**decode(b,sprite//6,s)))
 return dict(offset=fo,dimensions=(w//2,h//2),components=components)
def synthetic():
 # Opaque zero, odd heights, short/empty rows, skips, and overlapping components.
 rasters=[bytes([1,0,0,1,9,0]),bytes([3,0,2,255,0,255,2,7,0,0,0,3,4,5,6,0,254,1,0,0])]
 dimensions=[(1,2),(3,5)];raster=b''.join(rasters);s0=18+len(raster);s1=s0+12;s2=s1+30;s3=s2+42
 b=bytearray(struct.pack('<H4I',2,s0,s1,s2,s3)+raster);off=0
 for data,(w,h) in zip(rasters,dimensions):b.extend(struct.pack('<IBB',off,w,h));off+=len(data)
 for record,x,y in [(0,0,0),(1,-3,-5),(1,1,1)]:b.extend(struct.pack('<IhhBB',record*6,x,y,0,0))
 for component,count in [(0,1),(10,2),(20,1)]:b.extend(struct.pack('<Ihh3xB2x',component,6,10,count))
 b.extend(struct.pack('<IIH',14,28,7));b.extend(struct.pack('<IIH',0,0,9));return bytes(b)
def oracle(case,models,fade,gla,initial=None):
 kind,model,ox,oy,dx,dy,color,w,h,pitch,prefix,delta,ctx,flags,native,shade,hasfade,seed,table=case
 minimum=(h-1)*pitch+w if w and h else 0;extent=max(0,minimum+delta);dest=bytearray((i*37+seed)%256 for i in range(prefix+extent+3)) if initial is None else bytearray(initial);head=[0]*6
 invalid=pitch<w or extent<minimum
 if kind in [0,2]:
  invalid|=bool(ctx&2 or flags&16 or native&2 or not(flags&2 or shade>=30) and not hasfade)
  if kind==2:invalid|=not(1<=w<=640 and 1<=h<=480)
  if invalid:return 1,head,bytes(dest)
  route=-1 if flags&2 or shade>=30 else shade;head[0]=route
  if kind==2:
   for c in models[model]['components']:
    left=signed(ox)+c['x'];top=signed(oy)+(abs(c['y'])//2*(-1 if c['y']<0 else 1))
    for y in range(1,c['h']-c['h']%2,2):
     for x in range(c['w']):
      if not c['mask'][y*c['w']+x]:continue
      sy=(y-1)//2
      if 0<=left+x<w and 0<=top+sy<h:
       at=prefix+(top+sy)*pitch+left+x;dest[at]=gla[dest[at]];head[1]+=1
      else:head[2]+=1
  for c in models[model]['components']:
   left=signed(ox+c['x']);top=signed(oy+c['y'])
   for y in range(c['h']):
    for x in range(c['w']):
     at=y*c['w']+x
     if not c['mask'][at]:continue
     head[3]+=1
     if 0<=left+x<w and 0<=top+y<h:
      dest[prefix+(top+y)*pitch+left+x]=c['indices'][at] if route<0 else fade[route*256+c['indices'][at]];head[4]+=1
     else:head[5]+=1
 else:
  invalid|=not(1<=w<=640 and 1<=h<=480)
  if invalid:return 1,head,bytes(dest)
  c=models[model]['components'][0];left=signed(ox)+signed(dx);half=abs(signed(dy))//2*(-1 if signed(dy)<0 else 1);top=signed(oy)+half;head[:3]=[left,top,c['h']//2]
  # Iterate original stream rows and select odd source rows, rather than a reduced cached image.
  for y in range(1,c['h']-c['h']%2,2):
   sy=(y-1)//2
   for x in range(c['w']):
    if not c['mask'][y*c['w']+x]:continue
    head[3]+=1
    if 0<=left+x<w and 0<=top+sy<h:
     at=prefix+(top+sy)*pitch+left+x;dest[at]=gla[color*256+dest[at]];head[4]+=1
    else:head[5]+=1
 return 0,head,bytes(dest)
def main():
 WORK.mkdir(parents=True,exist_ok=True)
 sources=[ROOT/'rust/crates/genewars-render/src'/n for n in ['plant.rs','entity_shadow.rs','terrain.rs','terrain_modes.rs','plant_draw.rs']]+[ROOT/'rust/crates/genewars-assets/src/lib.rs',ROOT/'rust/crates/genewars-assets/src/mission_source.rs']
 sourcepins=[pin(p) for p in sources];evidence=[ROOT/'research/ghidra'/n for n in ['plant-color-evidence.json','plant-shadow-evidence.json']];proofs=[json.loads(p.read_bytes()) for p in evidence]
 contracts={v['private_path']:v['sha256'] for e in proofs for v in e['instruction_contracts']}
 for file,sha in contracts.items():assert pin(ROOT/file)['sha256']==sha,file
 corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results'];resources=[];resourcepins=[]
 for r in proofs[0]['resources']:
  p=ROOT/next(v['private_output'] for v in corpus if v['status']=='passed' and v['entry_id']==r['entry_id']);b=p.read_bytes();assert len(b)==r['decoded_size'] and hashlib.sha256(b).hexdigest()==r['decoded_sha256'];resources.append(b);resourcepins.append(pin(p))
 paths=[ROOT/p['path'] for p in resourcepins];synthetic_path=WORK/'synthetic.anb';synthetic_path.write_bytes(synthetic());resources.append(synthetic_path.read_bytes());paths.append(synthetic_path)
 (WORK/'resources.txt').write_text('\n'.join(str(p) for p in paths),encoding='utf8')
 selections=[(i,a,0) for i in range(12) for a in range(9)]+[(12,0,0),(12,0,1),(12,1,0)];models=[frame(resources[r],a,f) for r,a,f in selections]
 (WORK/'frames.bin').write_bytes(bytes(v for s in selections for v in s))
 for i,r in enumerate(proofs[0]['resources']):
  c=models[i*9]['components'][0];assert hashlib.sha256(c['indices']).hexdigest()==r['indices_sha256'];assert hashlib.sha256(c['mask']).hexdigest()==r['opacity_sha256']
 tables=[];tablepins=[]
 for id,name in [(61,'fade'),(107,'gla')]:
  p=ROOT/next(v['private_output'] for v in corpus if v['status']=='passed' and v['entry_id']==f'root/2/disc/{id}');b=p.read_bytes();(WORK/f'{name}.bin').write_bytes(b);tables.append(b);tablepins.append(pin(p))
 assert hashlib.sha256(tables[1]).hexdigest()==proofs[1]['resource']['decoded_sha256']
 fades=[tables[0],bytes((r*13+i*7+3)%256 for r in range(64) for i in range(256))];glas=[tables[1],bytes((r*19+i*7+5)%256 for r in range(256) for i in range(256))]
 def case(kind,m=110,origin=(0,0),offset=(0,0),color=0,geometry=(1,1,1),prefix=0,delta=0,ctx=0,flags=2,native=0,shade=0,hasfade=1,seed=11,table=1):return(kind,m,*origin,*offset,color,*geometry,prefix,delta,ctx,flags,native,shade,hasfade,seed,table)
 cases=[];categories={}
 def add(name,items):
  start=len(cases);cases.extend(items);categories[name]=len(cases)-start
 placements=[(0,0),(24,24),(-1,-1),(47,35),(-40,0),(0,-40),(65536+24,-65536+24),(32767,32767),(-32768,-32768),(2147483647,-2147483648)]
 add('real_frames_foreground',(case(0,m,origin=p,geometry=(48,36,53),prefix=7,flags=f,shade=s,table=t) for m in range(108) for p in placements for f,s,t in [(4,0,0),(0,29,1),(0,30,0),(2,0,1)]))
 add('real_frames_shadow',(case(1,m,origin=p,offset=(models[m]['components'][0]['x'],models[m]['components'][0]['y']),geometry=(48,36,53),prefix=7,color=c,table=t) for m in range(108) for p in placements[:6] for c,t in [(0,0),(201,1)]))
 add('synthetic_multicomponent',(case(k,m,origin=p,offset=d,geometry=g,prefix=5,color=173,flags=4,shade=29) for k in [0,1] for m in [108,109,110] for p in placements for d in [(-32768,-32767),(32767,-1),(0,0),(-2,3)] for g in [(1,1,1),(5,4,8),(48,36,53)]))
 add('all_entity_flag_shade_pairs',(case(0,flags=f,shade=s,hasfade=(s+f)%2) for f in range(256) for s in range(256)))
 add('all_context_native_flag_bytes',(case(0,ctx=c,native=n) for c,n in [(v,0) for v in range(256)]+[(0,v) for v in range(256)]))
 add('all_foreground_word_origins',(case(0,m=108,origin=(v,v^32768)) for v in range(65536)))
 add('all_shadow_y_offset_words',(case(1,offset=(0,v)) for v in range(65536)))
 add('all_shadow_gla_pairs',(case(1,color=c,seed=d) for c in range(256) for d in range(256)))
 add('view_bounds',(case(k,m=0,origin=(30,30),geometry=g,prefix=p,delta=d) for k in [0,1] for g in [(512,480,640),(1,1,2),(48,36,53),(0,1,1),(1,0,1),(5,4,4),(641,1,641),(1,481,1)] for p in [0,128] for d in [-1,0,1]))
 add('composed_real_frames',(case(2,m,origin=p,geometry=(48,36,53),prefix=7,flags=f,shade=s,table=t) for m in range(108) for p in placements for f,s,t in [(4,0,0),(0,29,1),(0,30,0),(2,0,1)]))
 add('composed_multicomponent',(case(2,m,origin=p,geometry=g,prefix=5,flags=4,shade=29) for m in [108,109,110] for p in placements for g in [(1,1,1),(5,4,8),(48,36,53)]))
 add('composed_dispatch_atomicity',(case(2,flags=f,shade=s,hasfade=(s+f)%2) for f in range(256) for s in range(256)))
 add('composed_context_native_atomicity',(case(2,ctx=c,native=n) for c,n in [(v,0) for v in range(256)]+[(0,v) for v in range(256)]))
 add('composed_view_bounds',(case(2,m=0,origin=(30,30),geometry=g,prefix=p,delta=d) for g in [(512,480,640),(1,1,2),(48,36,53),(0,1,1),(1,0,1),(5,4,4),(641,1,641),(1,481,1)] for p in [0,128] for d in [-1,0,1]))
 (WORK/'cases.bin').write_bytes(b''.join(struct.pack('<19i',*c) for c in cases))
 # Independent malformed selectors, section extents, component references and RLE limits.
 bad=[];b=resources[12];s=struct.unpack_from('<4I',b,2)
 for v in range(1,256):
  z=bytearray(b);z[s[1]+10+8]=v;bad.append(z)
 for at,values in [(2,[0,17,len(b)+1]),(6,[0,s[0]-1,len(b)+1]),(10,[s[1]-1,len(b)+1]),(14,[s[2]-1,len(b)+1]),(s[3],[1,42,0xffffffff]),(s[3]+4,[0,15,0xffffffff]),(s[2]+14,[1,30,0xffffffff]),(s[1]+10,[1,12,0xffffffff]),(s[0]+6,[s[0]-18,0xffffffff])]:
  for value in values:z=bytearray(b);struct.pack_into('<I',z,at,value);bad.append(z)
 invalid=[]
 for i,b in enumerate(bad):p=WORK/f'invalid-{i}.anb';p.write_bytes(b);invalid.append(str(p))
 (WORK/'invalid.txt').write_text('\n'.join(invalid),encoding='utf8')
 missions=[ROOT/f'private/reference-run/run-004/drive-c/GAME/LEVELS/CONQ{i:03}.DAT' for i in [1,2,3,24,40]];missionpins=[pin(p) for p in missions];missionbytes=[p.read_bytes() for p in missions];assert missionpins[0]['sha256']==proofs[0]['conq001_fixture']['sha256']
 data=bytearray(missionbytes[0])
 for i in range(512):
  at=0x43d4e+i*41;struct.pack_into('<hh',data,at+7,signed(i*127),signed(i*313));
  for j,off in enumerate([0x15,0x17,0x19,0x1b,0x23,0x26]):data[at+off]=(i*(2*j+1)+j*23)%256
 mp=WORK/'synthetic-mission.dat';mp.write_bytes(data);missions.append(mp);missionbytes.append(bytes(data));(WORK/'missions.txt').write_text('\n'.join(str(p) for p in missions),encoding='utf8')
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';lib=WORK/'libgenewars_assets.rlib';exe=WORK/'review.exe'
 for args in [['--crate-name','genewars_assets','--crate-type','rlib',str(ROOT/'rust/crates/genewars-assets/src/lib.rs'),'-o',str(lib)],[str(Path(__file__).with_suffix('.rs')),'--extern','genewars_assets='+str(lib),'-o',str(exe)]]:
  r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(WORK)],capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace')
 def read(s,n):b=s.read(n);assert len(b)==n;return b
 def u32(s):return struct.unpack('<I',read(s,4))[0]
 pixels=written=clipped=success=errors=0;metadata_pixels=0;digest=hashlib.sha256();fixturechecks=0
 with (WORK/'results.bin').open('rb') as native:
  for model in models:
   off=u32(native);dimensions=struct.unpack('<hh',read(native,4));count=u32(native);assert(off,dimensions,count)==(model['offset'],model['dimensions'],len(model['components']))
   for c in model['components']:
    x,y,w,h=struct.unpack('<hhBB',read(native,6));consumed=u32(native);indices=read(native,w*h);mask=read(native,w*h);assert(x,y,w,h,consumed,indices,mask)==(c['x'],c['y'],c['w'],c['h'],c['consumed'],c['indices'],c['mask']);metadata_pixels+=w*h
  for number,casevalue in enumerate(cases):
   table=casevalue[-1];expected=oracle(casevalue,models,fades[table],glas[table]);status=u32(native);header=list(struct.unpack('<6q',read(native,48)));actual=read(native,u32(native));assert(status,header,actual)==expected,(number,casevalue,status,header,expected[:2]);digest.update(actual);pixels+=len(actual);written+=header[4];clipped+=header[5];success+=status==0;errors+=status!=0
  atomic=u32(native);parse_rejections=u32(native)
  for b in missionbytes:
   for i in range(512):
    at=0x43d4e+i*41;expected=b[at+7:at+11]+bytes(b[at+off] for off in [0x15,0x17,0x19,0x1b,0x23,0x23,0x26]);assert read(native,11)==expected,(i,'source fields')
  assert not native.read(1)
 for f in proofs[1]['actual_fixtures']:
  m=f['slot']*9;expected=oracle(case(1,m,origin=(3,4),geometry=(64,48,64),table=0),models,fades[0],glas[0])[2][:-3];assert hashlib.sha256(expected).hexdigest()==f['output_sha256'];fixturechecks+=1
 assert sourcepins==[pin(p) for p in sources],'sources changed during comparison'
 report=dict(status='passed',classification='Corroborated static contract and native pure API comparisons',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,source_pins=sourcepins,evidence_pins=[pin(p) for p in evidence],instruction_contracts=len(contracts),resources=resourcepins,tables=tablepins,real_frames=108,synthetic_frames=3,metadata_pixels=metadata_pixels,cases=len(cases),case_categories=categories,successful_cases=success,rejected_cases=errors,compared_destination_bytes=pixels,opaque_writes=written,clipped_opaque_samples=clipped,atomic_malformed_cases=atomic,parser_rejections=parse_rejections,independent_shadow_fixture_hashes=fixturechecks,mission_inputs=missionpins,mission_source_field_records=3072,mission_source_bounds_rejected=12,output_aggregate_sha256=digest.hexdigest(),limitations=['No original execution or entity ordering/visibility/timing claim','Cached full EOL decode is stricter than clipped original early return','Shadow zero clip dimensions rejected by native API; original zero-clip behavior not asserted','Normal component/context path only; unsupported dispatch rejects atomically','Resource hash admission belongs to caller; this API accepts explicit structurally valid ANB'])
 report['review_sources']=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))];report['native_harness']=pin(exe);report['synthetic_anb']=pin(synthetic_path);report['private_cases']=pin(WORK/'cases.bin')
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');tmp.replace(out)
 print(json.dumps({k:report[k] for k in ['status','real_frames','cases','compared_destination_bytes','atomic_malformed_cases','parser_rejections']}))
if __name__=='__main__':main()
