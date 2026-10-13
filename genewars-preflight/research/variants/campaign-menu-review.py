"""Independent instruction-based plans, packet transitions and title pixels.

Only compiles and executes a private native API harness; never opens a GUI.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'scripts'),str(Path(__file__).parent)]
from cargo import environment
from importlib.util import spec_from_file_location,module_from_spec
spec=spec_from_file_location('glyph_oracle',Path(__file__).with_name('font-menu-review.py'))
oracle=module_from_spec(spec);spec.loader.exec_module(oracle)
sha=lambda b:hashlib.sha256(b).hexdigest()
def record(path):
 b=path.read_bytes();return dict(path=path.relative_to(ROOT).as_posix(),size=len(b),sha256=sha(b))

def expected(c):
 captured,selected,count,b8,b9,first,hit=c
 if captured<0:
  if b8&1 and first>=0:return first,1,0,first,first,1,8
  return -1,0,0,-1,-1,count,0
 if b8&2:return captured if hit else -1,1,8,-1,-1,count,0
 held=(2 if b8&4 else 0)|(4 if b9&1 else 0)
 # A8AAC always writes held before the unchanged-hit branch at A8B32.
 # A8B40 overwrites with1 only when the prior hit/countdown was nonzero.
 event=captured if hit!=selected else -1
 return event,1,1 if selected and not hit else held,captured,captured if hit else -1,int(hit),8

def main():
 receipts=[ROOT/'research/ghidra'/n for n in ['campaign-entry-evidence.json','menu-input-evidence.json','font-menu-evidence.json']]
 evidence=[json.loads(x.read_bytes()) for x in receipts];listings=[]
 for proof in evidence:
  for item in proof.get('instruction_contracts',proof.get('private_listings',[])):
   r=record(ROOT/item['private_path']);assert r['sha256']==item['sha256'];listings.append(r)
 assert sha((ROOT/'private/ghidra/GW-CD-candidate.exe').read_bytes())==evidence[0]['candidate_sha256']
 checks={
 'private/ghidra/campaign-screen/instructions-00015b90.txt':['00015bd6 IDIV ECX','00015be5 JNZ 0x00015c52','00015bf5 MOV byte ptr [EBX + 0x11945c],AL','00015cbe MOV AL,byte ptr [EAX + 0x11945c]','00015ec4 IDIV EBX','00015f11 IDIV EBX'],
 'private/ghidra/ui-input/instructions-000a87d0.txt':['000a8871 TEST byte ptr [ECX + 0x8],0x2','000a88a7 AND EAX,EDX','000a8aac MOV dword ptr [0x0019ff70],EBX','000a8b32 CMP AX,DI','000a8b40 MOV dword ptr [0x0019ff70],0x1','000a8be2 MOV byte ptr [EAX + 0x27],DL']}
 for name,terms in checks.items():
  text=(ROOT/name).read_text()
  for term in terms:assert term in text
 work=ROOT/'private/review/campaign-menu';work.mkdir(parents=True,exist_ok=True)
 cases=[]
 states=[(-1,False,0),(-1,False,1)]+[(target,hit,int(hit)) for target in [0,1,3,2] for hit in [False,True]]
 for target,selected,count in states:
  for b8 in range(256):
   for b9 in [0,1,2,254,255]:
    for first in [-1,0,1,3,2]:
     for hit in [False,True]:cases.append((target,selected,count,b8,b9,first,hit))
 (work/'packet-cases.bin').write_bytes(b''.join(struct.pack('<bBBBBbB',*c) for c in cases))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';env=environment();exe=work/'review.exe'
 commands=[]
 for crate in ['assets','core','render']:
  args=['--crate-name','genewars_'+crate,'--crate-type','rlib',str(ROOT/f'rust/crates/genewars-{crate}/src/lib.rs'),'-o',str(work/f'libgenewars_{crate}.rlib')]
  if crate=='render':args+=['--extern','genewars_assets='+str(work/'libgenewars_assets.rlib')]
  commands.append(args)
 commands.append([str(Path(__file__).with_suffix('.rs')),'--extern','genewars_assets='+str(work/'libgenewars_assets.rlib'),'--extern','genewars_core='+str(work/'libgenewars_core.rlib'),'--extern','genewars_render='+str(work/'libgenewars_render.rlib'),'-L','dependency='+str(work),'-o',str(exe)])
 for command in commands:
  result=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',*command],env=env,capture_output=True,timeout=90)
  assert result.returncode==0,result.stderr.decode(errors='replace')
 corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results'];row=next(x for x in corpus if x['entry_id']=='root/2/disc/92' and x['status']=='passed')
 anb_path=ROOT/row['private_output'];anb=anb_path.read_bytes();assert sha(anb)==row['output_sha256']
 game=ROOT/'private/reference-run/run-003/drive-c/GAME';palette_path=ROOT/'private/integration/menu/titlescr.pal'
 header_path=game/'LEVELS/HDRCO1.DAT';map_path=game/'LEVELS/CONQ001.DAT'
 result=subprocess.run([str(exe),str(work),str(anb_path),str(palette_path),str(header_path),str(map_path)],capture_output=True,timeout=60)
 assert result.returncode==0,result.stderr.decode(errors='replace')
 native=(work/'packet-results.bin').read_bytes();assert len(native)==len(cases)*11
 for index,case in enumerate(cases):
  actual=struct.unpack_from('<hBBhhHB',native,index*11);assert actual==expected(case),(case,actual,expected(case))
 plans=(work/'plans.tsv').read_text().splitlines();assert len(plans)==96
 for raw in plans:
  a=raw.split('\t');progress,prior=int(a[0]),int(a[1]);quotient,remainder=divmod(progress,6);system=quotient if remainder==0 else prior
  wanted=[str(progress),str(prior),str(remainder+1),str(system),f'LEVELS/hdrco{progress+1}.dat',f'levels/conq{progress+1:03}.dat',f'data/pl{system+1}.dat',f'data/levscr0{quotient+1}.dat',f'data/levscr0{quotient+1}.pal',f'anims/system{system+1}.smk' if remainder==0 else '-',f'data/toPlnt{system}.dat',f'data/toPlnt{system}.pal']
  assert a==wanted,(a,wanted)
 n,table,components,frames,animations=struct.unpack_from('<H4I',anb);mask=bytearray(346*75)
 palette=palette_path.read_bytes();rgb=[bytes(v*255//63 for v in palette[i:i+3]) for i in range(0,768,3)]
 for selected,frameoff in [(False,0),(True,28)]:
  target=bytearray([17])*(640*480*3);frame=anb[frames+frameoff:frames+frameoff+14]
  start=struct.unpack_from('<I',frame)[0]//10;count=frame[11];assert count==3
  for i in range(start,start+count):
   sprite,cx,cy,flags=struct.unpack_from('<IhhB',anb,components+i*10);assert flags==0
   offset,w,h=struct.unpack_from('<IBB',anb,table+sprite);indices,opaque,consumed,_=oracle.decode(anb[:table],18+offset,w,h)
   x,y=cx>>1,cy>>1
   oracle.paint(target,640,480,indices,opaque,w,h,140+x,115+y,rgb)
   if not selected:
    for sy in range(h):
     for sx in range(w):
      if opaque[sy*w+sx]:mask[(y+sy)*346+x+sx]=1
  assert (work/('title-selected.rgb' if selected else 'title-normal.rgb')).read_bytes()==target
 assert bytes(mask)==(work/'title-mask.bin').read_bytes()
 assert sha(mask)==evidence[1]['normal_hit']['mask_sha256'] and sum(mask)==18827
 header=header_path.read_bytes();assert len(header)==699 and (work/'title.bytes').read_bytes()==header[32:header.index(0,32)]
 data=map_path.read_bytes();assert struct.unpack_from('<I',data)[0]==0x55336
 assert (work/'map-prefix.bin').read_bytes()==data[:0x3b55e]
 assert (work/'map-tail.bin').read_bytes()==data[0x3b55e:]
 assert (work/'map-region.bin').read_bytes()==data[0x355e:0x3b55e]
 for i in range(4):assert (work/f'player-{i}.bin').read_bytes()==data[0x14+i*0x70d:0x14+(i+1)*0x70d]
 receipt=dict(status='passed-bounded-independent-native-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
  campaign_plans=96,admission_pairs=65536,packet_cases=len(cases),normal_hit_pixels=len(mask),normal_opaque=sum(mask),normal_mask_sha256=sha(mask),
  full_rgb_title_frames=2,full_rgb_channels_per_frame=640*480*3,header_bytes=699,map_prefix_bytes=0x3b55e,map_tail_bytes=len(data)-0x3b55e,
  source_receipts=[record(x) for x in receipts],private_listings=listings,
  crate_sources=[record(ROOT/f'rust/crates/genewars-{crate}/src/{name}') for crate,name in [('core','campaign.rs'),('core','menu_control.rs'),('assets','lib.rs'),('render','lib.rs')]],
  inputs=[record(x) for x in [anb_path,palette_path,header_path,map_path]],harness=record(exe),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['Progress0..23/system0..3 are safe-port admissions, not original checks or completion policy','Title hit union supports zero component flags; transforms not admitted','Safe complete-row RLE validation stronger than original final-row early-return','Copied mission prefix exposes initial serialized bytes; later live-state mutations are not modeled','No menu input cadence, mission simulation or GUI parity inferred'])
 (ROOT/'research/variants/campaign-menu-review.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:v for k,v in receipt.items() if k not in ['private_listings','source_receipts','crate_sources','inputs','harness','review_sources']}))

if __name__=='__main__':main()
