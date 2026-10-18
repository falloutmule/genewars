"""Literal stationary Mule corpse-to-BONES transition and bounded decay.

Actual before19785 starts the continuation. Explicit timer/reserve variants
exercise state26/27 without claiming their synthetic counters are live traces.
"""
from pathlib import Path
import importlib.util,sys,shutil,struct,json,hashlib,contextlib,io
P=Path(__file__).resolve().parents[2];W=P/'private/review/mule-bones'
START=P/'private/review/family8-trajectory/f476ea69e3a340aeb52b007f58608b61/frame19785/boundary'
BONES=P/'private/integration/sprites/2106ce79deaafe53d5fbb6292c78a8c71fb6920b08d1a11b5c44ea3b0ab75e9f.anb'
s=importlib.util.spec_from_file_location('mule',Path(__file__).with_name('creature-hungry-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);Original=m.CapturedSource
class WithBones(Original):
 def run(self,*args,**kwargs):
  state=kwargs['state_before'].copy();db=bytearray(BONES.read_bytes());assert hashlib.sha256(db).hexdigest()==BONES.stem
  frames,animations=struct.unpack_from('<II',db,10)
  for i in range(struct.unpack_from('<H',db)[0]):
   for off in [0,4]:struct.pack_into('<I',db,animations+10*i+off,0x700000+frames+struct.unpack_from('<I',db,animations+10*i+off)[0])
  state.update({0x700000+i:b for i,b in enumerate(db)});state.update({0x116804+i:b for i,b in enumerate(struct.pack('<I',0x700000+animations))})
  if state[0x1542be+0x5e]>=26:state.update({0x1542be+31+i:b for i,b in enumerate(struct.pack('<I',0x700000+animations))})
  kwargs['state_before']=state;return super().run(*args,**kwargs)

def main():
 W.mkdir(parents=True,exist_ok=True);m.CapturedSource=WithBones
 initial=(START/'creatures.bin').read_bytes()[:124];actual=initial;cases=[]
 for frame in range(19785,19801):cases.append(('continuation',frame,None))
 for state in [25,26,27]:
  for timer in [0,15,16,31,32,32767]:
   for reserve in ([62,63]if state==25 else [30,31,32]):
    for parity in [0,1]:
     a=bytearray(initial);a[0x5e]=state;a[0x19]=18 if state==25 else 4;a[0x1b]=1 if state==27 else 0;struct.pack_into('<h',a,0x68,reserve);struct.pack_into('<h',a,0x6c,timer);cases.append(('explicit_timer_reserve',20000+parity,bytes(a)))
 inputs=bytearray();expected=bytearray();steps=0;timelines=[];pins={}
 for index,(label,frame,before)in enumerate(cases):
  if before is None:before=actual
  fixture=W/f'case{index}';fixture.mkdir(exist_ok=True)
  for name in ['backing.bin','creatures.bin','actors.bin','effects.bin','family8.bin','players.bin','rng.bin']:shutil.copyfile(START/name,fixture/name)
  creatures=bytearray((fixture/'creatures.bin').read_bytes());creatures[:124]=before;(fixture/'creatures.bin').write_bytes(creatures)
  sys.argv=['source',str(frame),'--snapshot='+str(fixture.relative_to(P))]
  with contextlib.redirect_stdout(io.StringIO()):m.main()
  after=bytearray((fixture/f'source-after{frame}.bin').read_bytes());after[31:39]=before[31:39]
  report=json.loads((fixture/f'source-trace{frame}.json').read_text());assert report['report']['draws']==0 and report['report']['map_changed_bytes']==0
  inputs+=struct.pack('<I',frame)+before;expected+=after;steps+=report['report']['steps'];pins.update({r['path']:r['sha256']for r in report['listing_pins']})
  if label=='continuation':actual=bytes(after)
  timelines.append(dict(label=label,frame=frame,state_before=before[0x5e],state_after=after[0x5e],animation=after[0x19],animation_frame=after[0x1b],reserve=struct.unpack_from('<h',after,0x68)[0],timer=struct.unpack_from('<h',after,0x6c)[0],native_sha256=hashlib.sha256(after).hexdigest()))
 (W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected);(W/'map.bin').write_bytes((START/'backing.bin').read_bytes()[0x355e:0x3b55e])
 report=dict(status='literal-bones-transition-fixtures',callbacks=len(cases),literal_instructions=steps,timeline=timelines,listing_pins=pins,input_pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in list(START.glob('*.bin'))+[BONES,Path(__file__)]},original_execution=False,scope='Actual before19785 then16 isolated continuations; explicit timer/reserve/parity variants for stationary states25/26/27. Final retirement excluded.')
 (W/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','callbacks','literal_instructions']}))
if __name__=='__main__':main()
