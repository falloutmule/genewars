"""Independent source-derived actor indexed8 compositor comparison. No GUI."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,struct,subprocess,sys
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
WORK=P/'private/review/actor-render';INPUT=P/'private/integration/actor-resources'
def sha(b):return hashlib.sha256(b).hexdigest()
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes()))
def decode(data,table,i,end):
 offset,w,h=struct.unpack_from('<IBB',data,table+i*6);values=bytearray(w*h);mask=bytearray(w*h)
 if not w or not h:return w,h,0,values,mask
 pos=18+offset;start=pos
 for y in range(h):
  x=0
  while True:
   assert pos<end;run=struct.unpack_from('<b',data,pos)[0];pos+=1
   if not run:break
   if run<0:x-=run;continue
   assert pos+run<=end
   for j in range(run):
    if x+j<w:values[y*w+x+j]=data[pos+j];mask[y*w+x+j]=1
   x+=run;pos+=run
 return w,h,pos-start,values,mask
FADE=bytes((i*73+17)&255 for i in range(256))
def blend(a,b):return (a*3+b*7+11)&255
def composite(components,owner,shade,origin,globalflags,w,h,p,height=128):
 size=(h-1)*p+w;image=bytearray((i*37+19)&255 for i in range(size+9));prior=image[:];written=clipped=opaque=0
 for x,y,flags,sw,sh,indices,mask in components:
  f=(globalflags|flags)if not globalflags&3 else ((globalflags&252)|flags)
  if f&240:return bytes(13)+prior
  x//=2;y//=2
  if globalflags&1:x=-x-sw;f^=1
  if globalflags&2:y=-y-sh;f^=2
  # Oracle walks source pixels to destinations (native walks destinations).
  for sy in range(sh):
   for sx in range(sw):
    at=sy*sw+sx
    if not mask[at]:continue
    opaque+=1;dx=origin[0]+x+(sw-1-sx if f&1 else sx);dy=origin[1]+y+(sh-1-sy if f&2 else sy)
    if not(0<=dx<w and 0<=dy<h):clipped+=1;continue
    value=indices[at]
    if value>>4==12:value=((owner<<4)+192+(value&15))&255
    if shade<30:value=FADE[value]
    dest=dy*p+dx
    if f&8:value=blend(value,image[dest])
    elif f&4:value=blend(image[dest],value)
    image[dest]=value;written+=1
 return bytes([1])+struct.pack('<III',written,clipped,opaque)+image
def policy(n):
 owner=n>>14&3;shade=n&255;class_=n&3;mode=n>>2&3;draw=n>>4&7;flags15=n>>7&18;word=n*257&65535;signed59=(n>>8)-256 if n>>8>=128 else n>>8
 parity=((n>>13)+(n&255))&1;visible=class_!=3 or n>>8<224 or(owner==n>>12&3 and parity==0)
 f=(4 if n>>8>=171 else 12 if n>>8>=85 else 0)if class_==3 else 0
 if draw==1:mirror=bool(word&4)
 elif draw==7:mirror=word&260==260
 elif mode==0:mirror=bool((n>>5)&1)and((n>>6)&7)in(0,1,4)and bool(word&4)
 elif mode==1:mirror=word&132==132
 elif mode==2:mirror=bool((n>>10)&1)and word&132==132 and signed59 in(13,14)
 else:mirror=False
 f|=int(mirror)
 if flags15&16 and parity:f|=4
 height=n*97%1024-512;effective=shade+(height//128);fade=effective if not flags15&2 and shade<30 else -2147483648
 if fade<0 and fade!=-2147483648:return bytes(20)
 shadow=not bool(n&1);y=32767-(height//64)if shadow else 32767
 return bytes([1,int(visible),f,int(shadow)])+struct.pack('<iiii',-32768,y,effective,fade)
def main():
 WORK.mkdir(parents=True,exist_ok=True)
 proof=json.loads((P/'research/ghidra/actor-draw-contract.json').read_bytes())
 for item in proof['pins']:assert sha((P/item['path']).read_bytes())==item['sha256'],item
 sources=[p for c in ['assets','core','runtime'] for p in (P/f'rust/crates/genewars-{c}/src').rglob('*.rs')];before=[pin(p)for p in sources]
 rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
 for crate in ['assets','core','runtime']:
  args=[str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={WORK}','-o',str(WORK/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for d in ['assets','core']:args+=['--extern',f'genewars_{d}={WORK}/libgenewars_{d}.rlib']
  r=subprocess.run(args,env=environment(),capture_output=True,timeout=120);assert r.returncode==0,r.stderr.decode(errors='replace')
 args=[str(rustc),'--edition=2021','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={WORK}','-o',str(WORK/'review.exe')]
 for d in ['assets','core','runtime']:args+=['--extern',f'genewars_{d}={WORK}/libgenewars_{d}.rlib']
 for args in [args,[str(WORK/'review.exe'),str(INPUT),str(WORK)]]:
  r=subprocess.run(args,env=environment(),capture_output=True,timeout=120);assert r.returncode==0,r.stderr.decode(errors='replace')
 expected_sprites=bytearray();sprite_count=frame_count=component_count=0;resource_pins=[]
 actual_frames=(WORK/'frames.bin').read_bytes();frame_at=0;pixel_bytes=0
 for key in range(16):
  path=INPUT/f'key-{key:02}.anb';resource_pins.append(pin(path));data=path.read_bytes();table,cs,fs,anims=struct.unpack_from('<IIII',data,2)
  sprites=[]
  for i in range((cs-table)//6):
   s=decode(data,table,i,table);sprites.append(s);sw,sh,consumed,values,mask=s
   expected_sprites.extend(struct.pack('<BBI',sw,sh,consumed)+values+mask);sprite_count+=1
  for a in range(struct.unpack_from('<H',data)[0]):
   first,last=struct.unpack_from('<II',data,anims+10*a)
   for at in range(first,last+1,14):
    co=struct.unpack_from('<I',data,fs+at)[0];count=data[fs+at+11];components=[]
    for i in range(count):
     so,x,y,flags,extra=struct.unpack_from('<IhhBB',data,cs+co+10*i);sw,sh,_,values,mask=sprites[so//6];components.append((x,y,flags,sw,sh,values,mask));component_count+=1
    for n,origin in enumerate([(40,32),(-1,0),(79,63),(2147483647,-2147483648)]):
     expected=composite(components,key%4,29 if n%2==0 else 30,origin,[0,3,4,12][n],80,64,84)
     actual=actual_frames[frame_at:frame_at+len(expected)];assert actual==expected,('frame',key,a,at,n,next((j,x,y)for j,(x,y)in enumerate(zip(actual,expected))if x!=y))
     frame_at+=len(expected);pixel_bytes+=len(expected)-13
    frame_count+=1
 assert frame_at==len(actual_frames)
 actual_sprites=(WORK/'sprites.bin').read_bytes();assert actual_sprites==expected_sprites,'full sprite index/mask/consumption mismatch'
 syn=(WORK/'synthetic.bin').read_bytes();syn_at=0;synthetic_count=0
 values=bytes([0,192,207,208,255,193,1,194,195,196,2,197,198,199,3]);mask=bytes([1,1,1,1,0,1,0,1,1,1,1,1,0,1,1])
 for g in range(16):
  for c in range(16):
   for owner in range(4):
    for shade in [0,29,30,255]:
     for origin in [(3,2),(-2,-1),(8,6),(-2147483648,2147483647),(2147483647,-2147483648)]:
      expected=composite([(-3,3,c,5,3,values,mask),(1,-1,c,5,3,values,mask)],owner,shade,origin,g,9,7,13)
      assert syn[syn_at:syn_at+len(expected)]==expected,('synthetic',g,c,owner,shade,origin)
      syn_at+=len(expected);synthetic_count+=1;pixel_bytes+=len(expected)-13
 prior=bytes((i*37+19)&255 for i in range(96));expected=bytes(13)+prior
 for i in range(8):assert syn[syn_at:syn_at+109]==expected,('atomic',i);syn_at+=109
 assert syn_at==len(syn)
 actual=(WORK/'policies.bin').read_bytes();expected=b''.join(policy(n)for n in range(65536));assert actual==expected,next((i//20,i%20,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b)
 assert before==[pin(p)for p in sources],'sources changed while comparison ran'
 result=dict(status='passed-independent-native-actor-render-review',utc=datetime.now(timezone.utc).isoformat(),candidate_sha256=proof['candidate_sha256'],contract=pin(P/'research/ghidra/actor-draw-contract.json'),source_pins=before,review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],resource_pins=resource_pins,resources=16,sprite_records=sprite_count,sprite_index_mask_bytes=len(actual_sprites)-6*sprite_count,animation_frame_records=frame_count,frame_component_references=component_count,real_frame_comparisons=frame_count*4,synthetic_draw_cases=synthetic_count,atomic_rejections=8,wrapper_policy_cases=65536,compared_frame_bytes=pixel_bytes,native_outputs=[pin(WORK/n)for n in ['sprites.bin','frames.bin','synthetic.bin','policies.bin']],original_execution=False,gui_execution=False,limits=['Ordinary world shadow pass and source labels remain excluded; explicit compositor only','Synthetic fade/blend tables exercise ordering; no claim these are original table bytes','Public i32 extremes clip using widened arithmetic; source wrapping equivalence restricted to ordinary source screen domain','High flag bits reject transactionally; no unsupported source clipping path inferred','Strict decoded streams consume complete final EOL, stronger than original width-limited final-row return'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','sprite_records','real_frame_comparisons','synthetic_draw_cases','wrapper_policy_cases','compared_frame_bytes']}))
if __name__=='__main__':main()
