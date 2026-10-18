"""Explicit-state108E0 source-derived full-buffer comparison; no gameplay launch."""
from pathlib import Path
import ast,hashlib,json,math,struct,subprocess,sys,uuid
import validate_family8_state4_pixels as common
import validate_bturr_status_overlay_pixels as sprites
P=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(P/'scripts'));from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def functions(path,names,namespace):
 tree=ast.parse(path.read_text());nodes=[n for n in tree.body if isinstance(n,ast.FunctionDef)and n.name in names]
 assert len(nodes)==len(names);exec(compile(ast.Module(body=nodes,type_ignores=[]),str(path),'exec'),namespace)
geometry={}
functions(P/'scripts/ghidra/validate_mode5_triangle_bridge.py',['i32','div','sub','slope','classify'],geometry)
functions(P/'scripts/ghidra/validate_terrain_modes_bridge.py',['geometry','coverage'],geometry)
def icon(data,animation,frame):
 _,off=sprites.anb_sections(data);first,last,_=struct.unpack_from('<IIH',data,off[3]+10*animation)
 assert first+frame*14<=last
 f=off[2]+first+14*frame;assert data[f+11]==1
 c=off[1]+struct.unpack_from('<I',data,f)[0];sprite=struct.unpack_from('<I',data,c)[0]//6
 x,y,flags=struct.unpack_from('<hhB',data,c+4)
 w,h,px,mask,_=sprites.sprite_from_table(data,off,sprite)
 return w,h,px,mask,x>>1,y>>1,flags
def draw_icon(dst,sprite,origin,color,blink,gla):
 w,h,px,mask,dx,dy,flags=sprite
 left=origin[0]+dx;top=origin[1]+dy
 if color is None:left=common.i16(left);top=common.i16(top)
 for row in range(h):
  for col in range(w):
   src=row*w+(w-1-col if flags&1 else col)
   if not mask[src]:continue
   x,y=left+col,top+row
   if not(0<=x<640 and 0<=y<480):continue
   value=px[src]
   if color is not None and value>>4==12:value=(color+(value&15))&255
   at=y*648+x
   if blink:value=gla[dst[at]*256+value]if color is not None else gla[value*256+dst[at]]
   dst[at]=value
def expected(eth,plg,fade,gla,case):
 r,z,frames,origin,source_frame,white,colors=case
 _,_,sw,sh,dx,dy,indices,opacity=common.decode_resource(eth,8,r[27])
 dst=bytearray([0x35])*311032
 # Source96AC0 common shadow, before the108E0 overlay.
 for y in range(sh//2):
  for x in range(sw):
   if opacity[(2*y+1)*sw+x]:
    px,py=origin[0]+dx+x,origin[1]+int(dy/2)+y
    if 0<=px<640 and 0<=py<480:at=py*648+px;dst[at]=gla[dst[at]]
 ty=z[4]-7;blink=bool(r[21]&16 and (source_frame+r[15])&1)
 draw_icon(dst,icon(plg,ty+28,frames[ty+1]),origin,colors[ty],blink,gla)
 progress=struct.unpack_from('<i',z,14)[0];count=int(progress/1024)+1
 for i in range(max(count,0)):
  point=(common.i16(origin[0]-7*count+7+14*i),common.i16(origin[1]-10))
  draw_icon(dst,icon(plg,35,frames[0]),point,None,blink,gla)
 height=(struct.unpack_from('<i',r,39)[0]-struct.unpack_from('<i',r,17)[0])>>6
 timer=struct.unpack_from('<h',r,100)[0];delta=((int(((20-timer)<<8)/20)*height)>>8)if r[86]==11 else 0
 rx,ry=30,15
 xx=[-rx,-(rx*213>>8),-(rx>>1)]
 yy=[0]+[math.isqrt(int(ry*ry*(256-int(a*a*65536/(rx*rx*256)))/256))for a in xx[1:]]
 edge=list(zip(xx,yy))+[(-xx[2],yy[2]),(-xx[1],yy[1]),(rx,0)]
 extra=[(0,ry),(xx[1],0),(xx[2],yy[1]),(0,yy[1]),(-xx[2],yy[1]),(-xx[1],0)]
 v=[(origin[0]+x,origin[1]-delta+y)for x,y in edge+extra]
 shades=[fade[28*256+white],fade[30*256+white],white]
 # Source980C0 five column strips, then E1900 six wedges in exact call order.
 for i,c in enumerate([0,1,2,1,0]):
  bottom=min(v[i][1],v[i+1][1])
  for y in range(max(origin[1]-height,0),min(bottom,480)):
   for x in range(max(v[i][0],0),min(v[i+1][0],640)):
    at=y*648+x;dst[at]=gla[dst[at]*256+shades[c]]
 for ids,c in [([0,7,1],0),([11,5,4],0),([1,8,2],1),([10,4,3],1),([2,9,6],2),([9,3,6],2)]:
  for y,left,right in geometry['geometry']([list(v[i])for i in ids],480):
   begin,end=geometry['coverage'](640,left,right)
   for x in range(begin,end):at=y*648+x;dst[at]=gla[dst[at]*256+shades[c]]
 # Source10EA0 ordinary foreground after decoration; markerblink does not alter it.
 shade=0 if r[23]==0 else r[23]+((struct.unpack_from('<i',r,39)[0]-struct.unpack_from('<i',r,17)[0])>>7)
 for y in range(sh):
  for x in range(sw):
   at=y*sw+x
   if not opacity[at]:continue
   px,py=common.i16(origin[0]+dx)+x,common.i16(common.i16(origin[1]-height)+dy)+y
   if 0<=px<640 and 0<=py<480:dst[py*648+px]=indices[at]if shade>=30 else fade[shade*256+indices[at]]
 return dst
def main():
 work=P/'private/integration/family8-zone-decoration'/uuid.uuid4().hex;work.mkdir(parents=True)
 plgpath=P/'private/integration/rnc-decoded/cce0f7de88f2dc8ec251e70925de93d6e406c109d0bea9a82f4ee7f1592c9a30.bin'
 plg=common.checked_bytes(plgpath,'a39cc8ac8c987de5c4865ec77a4459221f276286f254653e4a3f3231424f6444',40414)
 eth=common.checked_bytes(common.ETHEREAL,common.ETHEREAL_SHA,91123);fade=common.checked_bytes(common.FADE,common.FADE_SHA,16384);gla=common.checked_bytes(common.GLA,common.GLA_SHA,65536)
 mapped=P/'private/ghidra/validated/memory-.object3.bin';colors=list(mapped.read_bytes()[0x2e38:0x2e3f]);assert colors==[240,208,192,208,240,224,16]
 recordpath=P/'private/review/krell-zone-bootstrap-001/after/family8.bin';original=recordpath.read_bytes()[:102]
 cases=[]
 for ty,length in enumerate([8,4,8,4,4,4,8]):
  for frame in range(length):
   for state in [11,12]:
    r=bytearray(original);r[86]=state;r[27]=frame%11;r[21]|=16;r[100:102]=struct.pack('<h',frame%21)
    r[39:43]=struct.pack('<i',struct.unpack_from('<i',r,17)[0]+3200)
    z=bytearray(19);z[4]=ty+7;z[14:18]=struct.pack('<i',1024*(frame%4));f=[0]*8;f[0]=frame%4;f[ty+1]=frame
    cases.append((r,z,f,(320,240),frame,145,colors))
 for origin in [(1,1),(-25,230),(637,475),(120,700)]:
  for count in [-2048,-1,4096]:
   r=bytearray(original);r[86]=11;r[100:102]=struct.pack('<h',10);z=bytearray(19);z[4]=9;z[14:18]=struct.pack('<i',count)
   cases.append((r,z,[3,0,0,7,0,0,0,0],origin,1,145,colors))
 motion=P/'private/review/zone-marker-motion/c59459093bcf4e978d5e0e5dc0b22706/expected.bin'
 motion_data=motion.read_bytes();captured_cases=0
 for index in range(105):
  r=motion_data[index*112:index*112+102]
  if r[86] not in [11,12]:continue
  z=bytearray(19);z[4]=9;z[14:18]=struct.pack('<i',1000)
  cases.append((r,z,[1,0,0,1,0,0,0,0],(320,300),index+1,145,colors));captured_cases+=1
 assert captured_cases>40
 raw=b''.join(bytes(r)+bytes(z)+bytes(f)+struct.pack('<hhIB',*o,fr,w)+bytes(c)for r,z,f,o,fr,w,c in cases);(work/'cases.bin').write_bytes(raw)
 deps=P/'private/build/rust-target/release/deps';libs={n:max(deps.glob(f'libgenewars_{n}-*.rlib'),key=lambda p:p.stat().st_mtime_ns)for n in ['runtime','render','assets']}
 harness=P/'scripts/desktop/family8_zone_decoration_harness.rs';impl=P/'rust/crates/genewars-desktop/src/bin/support/family8_scene.rs'
 command=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(harness),'-L',f'dependency={deps}','-o',str(work/'probe.exe')]
 for name,lib in libs.items():command+=['--extern',f'genewars_{name}={lib}']
 built=subprocess.run(command,env=environment(),capture_output=True,text=True,timeout=90);(work/'build.txt').write_text(built.stdout+built.stderr);assert built.returncode==0,built.stderr
 run=subprocess.run([str(work/'probe.exe'),str(common.ETHEREAL),str(plgpath),str(common.FADE),str(common.GLA),str(work/'cases.bin'),str(work/'pixels')],capture_output=True,text=True,timeout=90);(work/'probe.txt').write_text(run.stdout+run.stderr);assert run.returncode==0,run.stderr
 comparisons=[]
 for index,case in enumerate(cases):
  want=expected(eth,plg,fade,gla,case);got=(work/'pixels'/f'{index}.raw').read_bytes()
  assert want==got,(index,next((i,a,b)for i,(a,b)in enumerate(zip(want,got))if a!=b))
  comparisons.append(dict(case=index,state=case[0][86],type=case[1][4],origin=case[3],sha256=hashlib.sha256(got).hexdigest()))
 sources=[P/'private/ghidra'/x for x in ['countdown-expiry/instructions-00010ea0.txt','terrain-viewport-refs/instructions-000108e0.txt','display-geometry/instructions-000b5c50.txt','actor-draw/instructions-00095390.txt','ui-frame/instructions-00097a90.txt','validated/instructions-00097910.txt']]
 receipt=dict(status='passed',cases=len(cases),compared_bytes=len(cases)*311032,atomic_rejections=9,captured_motion_records=captured_cases,private_run=work.relative_to(P).as_posix(),comparisons=comparisons,
  pins={p.relative_to(P).as_posix():sha(p)for p in [Path(__file__),harness,impl,plgpath,common.ETHEREAL,common.FADE,common.GLA,mapped,recordpath,motion]+sources},
  scope='Explicit-state selector2 states11/12 fullshadow+108E0 icons/rectangles/triangles+foreground. All40 type-icon frames,4progress frames,flipped icon,blink parity,clipping,paddedframebuffer and negativeprogress.',
  limitations=['Diagnostic record modifications are pixel fixtures, not gameplay acceptance','Live animation timing and host binding tested separately','Source-derived Python draw model plus existing independently proved triangle raster; not original-executable framebuffer'],original_execution=False)
 (work/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');(P/'research/integration/family8-zone-decoration-pixels.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k]for k in ['status','cases','compared_bytes','atomic_rejections','private_run']}))
if __name__=='__main__':main()
