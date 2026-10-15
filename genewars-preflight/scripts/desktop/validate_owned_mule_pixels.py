"""Independent source-derived owned59470/94D70/94400 pixel comparison.
This is decoded source-model pixel equality, not original-execution screenshots.
"""
from pathlib import Path
import argparse,hashlib,importlib.util,json
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('neutral',P/'scripts/validate_mule_scene.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--input',type=Path,default=P/'private/review/owned-render-101.jsonl');ap.add_argument('--receipt',type=Path,default=P/'research/integration/owned-mule-render-101.json');a=ap.parse_args()
 anb,fade,gla=m.read_inputs();total=0;rows=[]
 for line in a.input.read_text().splitlines():
  case=json.loads(line);r=bytes.fromhex(case['record']);actual=bytes.fromhex(case['pixels']);dst=bytearray([0xa5]*(68*128+4));width,height,pitch=64,128,68
  assert r[121]<128 and (r[121]==0 or int.from_bytes(r[104:106],'little',signed=True)<=int.from_bytes(r[98:100],'little',signed=True)>>1), 'unimplemented lifecycle icon required'
  factor=256 if r[119]==255 and r[118]>=100 else min(256,max(150,r[118]*r[119]//100));flags=case['inherited'];comps=m.parse_frame(anb,r[25],r[27]);origin=[24,90];physical=case['height'];shade=0 if r[23]==0 else min(31,r[23]+(physical>>7))
  for c in comps:
   sprite=c['sprite'];sw,sh=sprite['width'],sprite['height'];ow=sw*factor>>8;ox=c['x']*factor>>8;oy=c['y']*factor>>9
   xoff=-ox-ow if flags&1 else ox;yoff=-oy-(sh*factor>>8) if flags&2 else oy
   cf=(c['flags']|(flags&~3))^(flags&3)
   m.draw_shadow(dst,width,height,pitch,sprite,[origin[0]+xoff,origin[1]+(factor>>5)+yoff],factor,cf,23,gla)
  for c in comps:
   sprite=c['sprite'];sw,sh=sprite['width'],sprite['height'];ow,oh=sw*factor>>8,sh*factor>>8
   #94D70 rounds component positions toward zero, unlike signed shifts.
   ox=int(c['x']*factor/256);oy=int(c['y']*factor/256);xoff=-ox-ow if flags&1 else ox;yoff=-oy-oh if flags&2 else oy
   cf=(c['flags']|(flags&~3))^(flags&3);left=origin[0]+xoff;top=origin[1]-(physical>>6)+yoff
   if not ow or not oh:continue
   xs=(sw<<16)//ow;ys=(sh<<16)//oh
   for dy in range(oh):
    sy=min(sh-1,dy*ys>>16);y=top+(oh-1-dy if cf&2 else dy)
    for dx in range(ow):
     sx=min(sw-1,dx*xs>>16);x=left+(ow-1-dx if cf&1 else dx);source=sy*sw+sx
     if not(0<=x<width and 0<=y<height)or not sprite['opaque'][source]:continue
     nibble=sprite['pixels'][source]&15;v=135-(nibble>>2) if nibble<3 else ((r[24]*16-64+nibble)&255)
     dst[y*pitch+x]=fade[shade*256+v] if r[21]&2==0 and shade<30 else v
  differences=[i for i,(x,y)in enumerate(zip(actual,dst))if x!=y];assert len(actual)==len(dst)and not differences,(case['owner'],case['variant'],differences[:8]);total+=len(dst);rows.append({'owner':r[24],'variant':case['variant'],'sha256':hashlib.sha256(dst).hexdigest()})
 files=[Path(__file__),P/'scripts/validate_mule_scene.py',P/'rust/crates/genewars-desktop/src/bin/support/creature_scene.rs',P/'private/ghidra/terrain-viewport/instructions-00059470.txt',P/'private/ghidra/bones-frame/instructions-00094d70.txt',P/'private/ghidra/bones-blit/instructions-00094400.txt']
 receipt={'status':'passed','cases':len(rows),'compared_bytes':total,'mismatch_count':0,'scope':'Independent decoded original-data/source-derived owned59470/94D70/94400 color and existing95100 shadow model; no original-execution/native acceptance claim','private_input':str(a.input),'cases_summary':rows,'pins':{str(p.relative_to(P)):sha(p)for p in files}}
 a.receipt.write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in receipt.items()if k not in ['pins','cases_summary']}))
if __name__=='__main__':main()
