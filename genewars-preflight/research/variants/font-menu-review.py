"""Independent instruction-derived glyph oracle and full static menu comparison.

Only builds/runs a private native API harness. Never opens a GUI or original.
"""
from datetime import datetime,timezone
import hashlib,json,struct,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
sha=lambda b:hashlib.sha256(b).hexdigest()
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=sha(b))

def decode(data,offset,width,height):
 """Model A7E9B raw/no-flags writes, then separately validate complete rows.

 Positive control advances the full source run (A7EC3..A7ECB), while its
 destination copy is min(run, remaining width) (A7EA6..A7EAA). Negative
 controls only move the destination. EOL scanning realigns subsequent rows.
 The safe API additionally scans the final row's EOL (runtime may return).
 """
 indices=bytearray(width*height);mask=bytearray(width*height);cursor=offset
 stats=dict(rows=0,short=0,exact=0,overshoot=0,literals=0,opaque_zero=0,last_row_filled=False)
 if not width or not height:return bytes(indices),bytes(mask),0,stats
 for y in range(height):
  remaining=width;destination=y*width;logical=0
  while True:
   assert cursor<len(data);control=data[cursor];cursor+=1
   signed=control if control<128 else control-256
   if not signed:break
   logical+=abs(signed)
   if signed>0:
    end=cursor+signed;assert end<=len(data)
    visible=max(0,min(signed,remaining))
    for j in range(visible):
     indices[destination+j]=data[cursor+j];mask[destination+j]=1
     stats['opaque_zero']+=data[cursor+j]==0
    destination+=visible;remaining-=visible;cursor=end;stats['literals']+=signed
   else:
    skip=max(0,min(-signed,remaining));destination+=skip;remaining-=skip
  stats['rows']+=1;stats['short']+=logical<width;stats['exact']+=logical==width;stats['overshoot']+=logical>width
  if y==height-1:stats['last_row_filled']=logical>=width
 return bytes(indices),bytes(mask),cursor-offset,stats

def paint(target,width,height,indices,mask,sw,sh,x,y,palette):
 for sy in range(sh):
  for sx in range(sw):
   s=sy*sw+sx;dx=x+sx;dy=y+sy
   if mask[s] and 0<=dx<width and 0<=dy<height:
    target[(dy*width+dx)*3:(dy*width+dx)*3+3]=palette[indices[s]]

def line(font,text,target,x,y,palette):
 data,tab=font;cursor=x
 for code in text:
  assert code>=32;offset,w,h=struct.unpack_from('<IBB',tab,(code-31)*6);assert w
  if code!=32:
   indices,mask,_,_=decode(data,offset,w,h);paint(target,640,480,indices,mask,w,h,cursor,y,palette)
  cursor+=w-1
 return cursor

def main():
 proof_path=ROOT/'research/ghidra/font-menu-evidence.json';proof=json.loads(proof_path.read_bytes())
 listings=[]
 for p in proof['private_listings']:
  actual=record(ROOT/p['private_path']);assert actual['sha256']==p['sha256'];listings.append(actual)
 assert record(ROOT/'private/ghidra/GW-CD-candidate.exe')['sha256']==proof['candidate_sha256']
 # Corroborate the descriptor bindings directly in reconstructed object bytes.
 obj=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes()
 for va,name,dest in [(0x106244,b'data/font2.dat',0x118c94),(0x106270,b'data/font2.tab',0x118c8c),(0x106218,b'data/titlescr.pal',0x18500a)]:
  at=va-0x100000;assert obj[at:obj.index(0,at)]==name and struct.unpack_from('<I',obj,at+28)[0]==dest
 raw_listing=(ROOT/'private/ghidra/display-geometry/instructions-000a7730.txt').read_text()
 for term in ['000a7ea6 CMP EAX,EBP','000a7eaa MOV EAX,EBP','000a7ec3 MOVSX EAX,byte ptr [EDX]','000a7ecb ADD EDX,EAX','000a7f08 MOV EAX,0x1','000a7f24 JNZ 0x000a7f15']:assert term in raw_listing
 scratch=ROOT/'private/review/font-menu';scratch.mkdir(parents=True,exist_ok=True)
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';env=environment();source=Path(__file__).with_suffix('.rs')
 assets=scratch/'libgenewars_assets.rlib';render=scratch/'libgenewars_render.rlib';exe=scratch/'font-menu-review.exe'
 commands=[['--crate-name','genewars_assets','--crate-type','rlib',str(ROOT/'rust/crates/genewars-assets/src/lib.rs'),'-o',str(assets)],
  ['--crate-name','genewars_render','--crate-type','rlib',str(ROOT/'rust/crates/genewars-render/src/lib.rs'),'--extern','genewars_assets='+str(assets),'-o',str(render)],
  [str(source),'--extern','genewars_assets='+str(assets),'--extern','genewars_render='+str(render),'-L','dependency='+str(scratch),'-o',str(exe)]]
 for args in commands:
  result=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*args],capture_output=True,env=env,timeout=60)
  assert result.returncode==0,result.stderr.decode(errors='replace')
 manifest=json.loads((ROOT/'research/game-files-manifest.json').read_bytes())['entries']
 corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results']
 runtime=ROOT/'private/reference-run/run-003/drive-c/GAME';menu=ROOT/'private/integration/menu'
 text=(runtime/'DATA/TEXT.DAT').read_bytes();assert sha(text)=='23e60305a9fa028a0f6541456ab453b7e7cc439f787a2b75e4b8d088e0b01786'
 strings=[];at=0
 for _ in range(604):end=text.index(0,at);strings.append(text[at:end]);at=end+1
 assert at==36609 and text[at:]==b'\0'
 pal=(menu/'titlescr.pal').read_bytes();palette=[bytes(v*255//63 for v in pal[i:i+3]) for i in range(0,768,3)]
 assert max(pal)<=63
 fonts=[];totalglyphs=totalpixels=totalopaque=totalrows=0
 for number in [0,1,2,3,4,5,7]:
  inputs=[]
  for extension in ['DAT','TAB']:
   path=runtime/'DATA'/f'FONT{number}.{extension}';raw=path.read_bytes()
   entry=next(e for e in manifest if e['kind']=='file' and e.get('normalized_iso_path')==f'GAME/DATA/FONT{number}.{extension}')
   assert len(raw)==entry['size'] and sha(raw)==entry['sha256']
   if raw.startswith(b'RNC'):
    cached=next(r for r in corpus if r['entry_id']==entry['entry_id'] and r['status']=='passed');data=(ROOT/cached['private_output']).read_bytes();assert sha(data)==cached['output_sha256']
   else:data=raw
   destination=scratch/f'font{number}.{extension.lower()}';destination.write_bytes(data);inputs.append((data,record(path),record(destination)))
  dat,tab=inputs[0][0],inputs[1][0];assert len(tab)%6==0 and len(tab)>=6
  output=scratch/f'font{number}';output.mkdir(exist_ok=True)
  result=subprocess.run([str(exe),str(scratch/f'font{number}.dat'),str(scratch/f'font{number}.tab'),str(runtime/'DATA/TEXT.DAT'),str(menu/'titlescr.pal'),str(output)],capture_output=True,timeout=30)
  assert result.returncode==0,result.stderr.decode(errors='replace')
  native=json.loads(result.stdout);blob=(output/'glyphs.bin').read_bytes();position=0;glyphs=[]
  for r in range((len(tab)-6)//6):
   offset,w,h=struct.unpack_from('<IBB',tab,(r+1)*6);indices,mask,consumed,stats=decode(dat,offset,w,h)
   header=struct.unpack_from('<6I',blob,position);position+=24
   assert header==(r,offset,w,h,consumed,w*h)
   assert blob[position:position+w*h]==indices;position+=w*h
   assert blob[position:position+w*h]==mask;position+=w*h
   glyphs.append(dict(record=r,offset=offset,width=w,height=h,consumed=consumed,end=offset+consumed,indices_sha256=sha(indices),mask_sha256=sha(mask),opaque=sum(mask),**stats))
   totalglyphs+=1;totalpixels+=w*h;totalopaque+=sum(mask);totalrows+=h
  assert position==len(blob) and native['glyphs']==len(glyphs)
  widths={}
  for index,s in enumerate(strings):
   if all(32<=code<32+len(glyphs) and tab[(code-31)*6+4]>0 for code in s):widths[index]=sum(tab[(code-31)*6+4]-1 for code in s)
  actual={int(i):int(w) for i,w in (s.split('\t') for s in (output/'lines.tsv').read_text().splitlines())};assert actual==widths
  surfaces=[]
  for i,(index,x,y) in enumerate([(255,17,11),(258,-9,37),(259,610,456)]):
   if index in widths:
    expected=bytearray([91]*(640*480*3));end=line((dat,tab),strings[index],expected,x,y,palette);assert end==x+widths[index]
    path=output/f'line-{i}.rgb';assert path.read_bytes()==expected;surfaces.append(record(path))
  fonts.append(dict(font=number,inputs=[dict(source=v[1],decoded=v[2]) for v in inputs],native_checks=native,glyphs=glyphs,supported_text_lines=len(widths),line_surfaces=surfaces))
 # Independently compose an indexed background and the traced three controls.
 for asset in proof['assets']:
  name=Path(asset['source']).name.lower();path=menu/name
  assert len(path.read_bytes())==asset['decoded_size'] and sha(path.read_bytes())==asset['decoded_sha256']
 background=(menu/'titlescr.dat').read_bytes();expected=bytearray().join(palette[index] for index in background)
 anb=(menu/'iftitle.anb').read_bytes();_,t,c,f,a=struct.unpack_from('<H4I',anb)
 first,last,count,flags=struct.unpack_from('<2I2B',anb,a);assert (first,last,count,flags)==(0,28,3,0)
 co,fw,fh=struct.unpack_from('<Ihh',anb,f);assert (fw//2,fh//2,anb[f+11])==(346,75,3)
 parts=[]
 for i in range(3):
  sp,dx,dy,flags,_=struct.unpack_from('<IhhBB',anb,c+co+10*i);assert flags==0
  offset,w,h=struct.unpack_from('<IBB',anb,t+sp);indices,mask,consumed,_=decode(anb[:t],18+offset,w,h)
  assert (sp//6,dx//2,dy//2)==[(1,0,0),(2,116,11),(3,236,3)][i]
  for y in [115,186,257]:paint(expected,640,480,indices,mask,w,h,140+dx//2,y+dy//2,palette)
  parts.append(dict(sprite=sp//6,width=w,height=h,consumed=consumed,indices_sha256=sha(indices),mask_sha256=sha(mask)))
 font2=(scratch/'font2.dat').read_bytes(),(scratch/'font2.tab').read_bytes();labels=[]
 for index,y in [(255,140),(258,211),(259,282)]:
  width=sum(font2[1][(v-31)*6+4]-1 for v in strings[index]);x=312-(width//2);line(font2,strings[index],expected,x,y,palette)
  labels.append(dict(index=index,length=len(strings[index]),text_sha256=sha(strings[index]),width=width,left=x,y=y))
 bmp_path=menu/'menu-first.bmp';bmp=bmp_path.read_bytes();start=struct.unpack_from('<I',bmp,10)[0];width,height=struct.unpack_from('<ii',bmp,18)
 assert (width,height)==(640,480) and struct.unpack_from('<H',bmp,28)[0]==24
 actual=bytearray()
 for y in range(480):
  row=bmp[start+(479-y)*1920:start+(480-y)*1920]
  for x in range(640):actual.extend(row[3*x:3*x+3][::-1])
 assert actual==expected
 (scratch/'menu-oracle.rgb').write_bytes(expected)
 result=dict(status='passed',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
  evidence=record(proof_path),candidate_sha256=proof['candidate_sha256'],listing_hash_checks=listings,
  assets_source=record(ROOT/'rust/crates/genewars-assets/src/lib.rs'),render_source=record(ROOT/'rust/crates/genewars-render/src/lib.rs'),menu_source=record(ROOT/'rust/crates/genewars-desktop/src/bin/genewars-menu.rs'),menu_art_source=record(ROOT/'rust/crates/genewars-desktop/src/bin/support/menu_art.rs'),harness=record(source),native_harness=record(exe),
  font_count=len(fonts),glyph_count=totalglyphs,indices_compared=totalpixels,opacity_bytes_compared=totalpixels,opaque_pixels=totalopaque,rows=totalrows,fonts=fonts,
  text=dict(source=record(runtime/'DATA/TEXT.DAT'),entries=604,consumed=at,trailing_bytes=1),
  menu=dict(width=640,height=480,pixels_compared=307200,channels_compared=921600,bmp=record(bmp_path),rgb_sha256=sha(expected),normal_frame=0,animation=0,parts=parts,labels=labels),
  limits=['Original runtime may return before final EOL; safe decoder consumption is complete-row validation, not runtime source-pointer parity',
   'Single unkerned lines only; control bytes and zero-width glyphs rejected deliberately',
   'Menu oracle covers zero draw flags, full640x480 clip and three-button normal frame0, not hover/input/timing/network branch',
   'RGB6 display policy is floor(v*255/63), not analog DAC or monitor fidelity; no original runtime comparison claimed'])
 Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps(dict(status='passed',fonts=len(fonts),glyphs=totalglyphs,indices=totalpixels,opaque=totalopaque,menu_pixels=307200,menu_channels=921600)))

if __name__=='__main__':main()
