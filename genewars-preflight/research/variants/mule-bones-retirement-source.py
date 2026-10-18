"""Final BONES numeric epilogue and two complete captured retirement inputs.

The112 numeric variants defer unlink/cleanup; the two actual captures execute
both services literally over every shared native pool. No original runs.
"""
from pathlib import Path
import importlib.util,sys,shutil,struct,json,hashlib,contextlib,io
P=Path(__file__).resolve().parents[2];W=P/'private/review/mule-bones-retirement'
s=importlib.util.spec_from_file_location('bones',Path(__file__).with_name('mule-bones-source.py'));bones=importlib.util.module_from_spec(s);s.loader.exec_module(bones);m=bones.m
ORIGINS=[P/'private/review/family8-trajectory/14559dc8cc54441781bb46166a55c8eb/frame22607/boundary',P/'private/review/family8-trajectory/40137f93149345a4b4d8d1f87b70709d/frame22543/boundary']
CURRENT=None;PURE=True;instances=[]
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
OVERLAYS={1:'creatures.bin',3:'actors.bin',5:'effects.bin',8:'family8.bin'}
class FinalModel(bones.WithBones):
 def __init__(self):
  super().__init__();ui=P/'private/ghidra/construction-command/instructions-00029100.txt';self.files+=[ui];self.code.update(m.m.m.Model([ui],b'').code);self.code[0x2e0000]=('RET',[]);order=sorted(self.code);self.next=dict(zip(order,order[1:]));self.cleanup_calls=[];instances.append(self)
 def get(self,o):
  if PURE and o in ['0x0008acc0','0x000724c0']:self.cleanup_calls.append(o);return 0x2e0000
  return super().get(o)
 def run(self,*args,**kwargs):
  state=kwargs['state_before'].copy();backing=(CURRENT/'backing.bin').read_bytes()
  for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
   if kind==1:continue
   overlay=(CURRENT/OVERLAYS[kind]).read_bytes()if kind in OVERLAYS else None
   for i in range(count):
    raw=overlay[i*width:(i+1)*width]if overlay is not None else backing[start+i*stride:start+(i+1)*stride]
    native=bytearray(raw if overlay is not None else raw[:31]+bytes(8)+raw[31:])
    if kind in {2,3,5,8}:struct.pack_into('<I',native,35,{2:0x1120ac,3:0x112194,5:0x112254,8:0x11242c}[kind])
    state.update({address+i*width+j:b for j,b in enumerate(native)})
  state.update({0x185ec4+i:0 for i in range(4)})
  # Explicit native-empty portrait list policy.29100 is executed literally;
  # this is not an original UI object's captured state.
  state.update({0x1860be+0x264+i:0 for i in range(2)});state[0x1860be+0x26d]=0
  kwargs['state_before']=state;return super().run(*args,**kwargs)

def one(origin,frame,label,index,before=None,pure=True):
 global CURRENT,PURE,instances
 CURRENT=W/label;CURRENT.mkdir(parents=True,exist_ok=True);PURE=pure;instances=[]
 for name in ['backing.bin','creatures.bin','actors.bin','effects.bin','family8.bin','players.bin','rng.bin']:shutil.copyfile(origin/name,CURRENT/name)
 if before is not None:
  data=bytearray((CURRENT/'creatures.bin').read_bytes());data[:124]=before;(CURRENT/'creatures.bin').write_bytes(data)
 before=(CURRENT/'creatures.bin').read_bytes()[:124];sys.argv=['source',str(frame),'--snapshot='+str(CURRENT.relative_to(P))]
 with contextlib.redirect_stdout(io.StringIO()):m.main()
 after=bytearray((CURRENT/f'source-after{frame}.bin').read_bytes());after[31:39]=before[31:39];trace=json.loads((CURRENT/f'source-trace{frame}.json').read_text());assert after[6]==0 and after[:6]==before[:6] and struct.unpack_from('<h',after,0x68)[0]==0
 if pure:assert all(model.cleanup_calls==['0x0008acc0','0x000724c0']for model in instances)
 return before,after,trace,instances[0]

def main():
 W.mkdir(parents=True,exist_ok=True);m.CapturedSource=FinalModel;inputs=bytearray();expected=bytearray();steps=0;proofs=[];pins={}
 for i,origin in enumerate(ORIGINS):
  frame=[22607,22543][i];before,after,trace,model=one(origin,frame,f'actual{frame}',i,pure=False);steps+=trace['report']['steps'];pins.update({r['path']:r['sha256']for r in trace['listing_pins']})
  output={name:bytearray((CURRENT/name).read_bytes())for name in ['backing.bin','creatures.bin','actors.bin','effects.bin','family8.bin','players.bin','rng.bin']}
  output['backing.bin'][:0x355e]=(CURRENT/f'source-prefix-after{frame}.bin').read_bytes();output['backing.bin'][0x355e:0x3b55e]=(CURRENT/f'source-map-after{frame}.bin').read_bytes();output['backing.bin'][0x43d4e:0x48f4e]=(CURRENT/f'source-plants-after{frame}.bin').read_bytes()
  for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
   if kind in OVERLAYS:
    name=OVERLAYS[kind];data=bytearray(model.load(address+j,1)for j in range(count*width))
    for offset in range(0,len(data),width):data[offset+31:offset+39]=output[name][offset+31:offset+39]
    output[name]=data
   elif kind!=2:
    for slot in range(count):
     raw=bytes(model.load(address+slot*width+j,1)for j in range(width));output['backing.bin'][start+slot*stride:start+(slot+1)*stride]=raw[:31]+raw[39:]
  output['players.bin']=bytes(model.load(0x118d74+j,1)for j in range(7220));output['rng.bin']=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
  out=CURRENT/'expected-world';out.mkdir(exist_ok=True)
  for name,data in output.items():(out/name).write_bytes(data)
  normalized=bytearray(after);normalized[6]=1;inputs+=struct.pack('<I',frame)+before;expected+=normalized;proofs.append(dict(frame=frame,label='actual-full-retirement',literal_instructions=trace['report']['steps'],current41=struct.unpack_from('<h',before,0x41)[0],source_after_sha256=hashlib.sha256(after).hexdigest(),input_pins={p.relative_to(P).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in origin.glob('*.bin')if not p.name.startswith('source-')},outputs={name:hashlib.sha256(data).hexdigest()for name,data in output.items()}))
 base=(ORIGINS[0]/'creatures.bin').read_bytes()[:124]
 for case in range(112):
  a=bytearray(base);reserve=case&1;timer=(case//4)*16;frame=22607+(2 if case&2 else 0);struct.pack_into('<h',a,0x68,reserve);struct.pack_into('<h',a,0x6c,timer)
  for offset in [0x4d,0x50,0x55,0x5b]:struct.pack_into('<h',a,offset,case*137-6000)
  before,after,trace,model=one(ORIGINS[0],frame,f'variant{case}',case,bytes(a),True);normalized=bytearray(after);normalized[6]=1;inputs+=struct.pack('<I',frame)+before;expected+=normalized;steps+=trace['report']['steps'];pins.update({r['path']:r['sha256']for r in trace['listing_pins']})
 (W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected);(W/'map.bin').write_bytes((ORIGINS[0]/'backing.bin').read_bytes()[0x355e:0x3b55e]);(W/'map-seed65535.bin').write_bytes((ORIGINS[1]/'backing.bin').read_bytes()[0x355e:0x3b55e])
 report=dict(status='literal-bones-retirement-fixtures',callbacks=114,numeric_variants=112,literal_instructions=steps,actual_proofs=proofs,listing_pins=pins,original_execution=False,scope='Two complete actual final callbacks with8ACC0/724C0;112 numeric variants defer those two services. Pure expected retainskind1; finalclear/count/maps are worldadapter work.')
 (W/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','callbacks','literal_instructions']}))
if __name__=='__main__':main()
