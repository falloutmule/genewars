"""Literal final Mule callbacks versus archived, exact native world captures.

The seven world buffers are compared without rebuilding the archived binary.
29100 executes with an explicit empty portrait list;2C200 remains a UI port.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,struct
P=Path(__file__).resolve().parents[2]
CASES=[(5428,22607,'b424c6e83c584b6c9f22c5cd31fa6303'),
       (65535,22543,'6fae162871d44f5e81faafa8a0fceb7d')]
NAMES=['backing.bin','creatures.bin','actors.bin','effects.bin','family8.bin','players.bin','rng.bin']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 spec=importlib.util.spec_from_file_location('final_source',Path(__file__).with_name('mule-bones-retirement-source.py'))
 s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s);s.m.CapturedSource=s.FinalModel
 cases=[];total=steps=0;pins={};digest=hashlib.sha256()
 for seed,frame,identifier in CASES:
  run=P/'private/review/family8-trajectory'/identifier
  provenance=json.loads((run/'result.json').read_text());compiled=json.loads((run/'compilation.json').read_text())
  assert provenance['status']=='passed' and provenance['seed']==seed and provenance['completed_frames']>=frame
  assert provenance['binary_sha256']==compiled['binary_sha256']==sha(run/'probe.exe')
  for name,expected_hash in compiled['source_pins'].items():
   assert sha(run/'compilation_sources'/name)==expected_hash,('archived source',name)
  before=run/f'frame{frame}'/'before-mule';actual=run/f'frame{frame}'/'after-mule'
  for role,folder in [('before-mule',before),('after-mule',actual)]:
   for name in NAMES:
    assert sha(folder/name)==provenance['captures'][f'frame{frame}/{role}/{name}']
    digest.update(f'{seed}/{frame}/{role}/{name}\0'.encode()+(folder/name).read_bytes())
  _,after,trace,model=s.one(before,frame,f'captured-{seed}-{frame}',0,pure=False)
  output={name:bytearray((before/name).read_bytes())for name in NAMES};work=s.CURRENT
  output['backing.bin'][:0x355e]=(work/f'source-prefix-after{frame}.bin').read_bytes()
  output['backing.bin'][0x355e:0x3b55e]=(work/f'source-map-after{frame}.bin').read_bytes()
  output['backing.bin'][0x43d4e:0x48f4e]=(work/f'source-plants-after{frame}.bin').read_bytes()
  for kind,(start,count,stride,address,width)in enumerate(s.LAYOUTS,1):
   if kind in s.OVERLAYS:
    name=s.OVERLAYS[kind];data=bytearray(model.load(address+j,1)for j in range(count*width))
    for offset in range(0,len(data),width):data[offset+31:offset+39]=output[name][offset+31:offset+39]
    output[name]=data
   elif kind!=2:
    for slot in range(count):
     raw=bytes(model.load(address+slot*width+j,1)for j in range(width))
     output['backing.bin'][start+slot*stride:start+(slot+1)*stride]=raw[:31]+raw[39:]
  output['players.bin']=bytes(model.load(0x118d74+j,1)for j in range(7220))
  output['rng.bin']=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
  destination=work/'expected-world';destination.mkdir(exist_ok=True)
  for name,data in output.items():
   observed=(actual/name).read_bytes()
   assert data==observed,(seed,frame,name,[(i,a,b)for i,(a,b)in enumerate(zip(data,observed))if a!=b][:16])
   (destination/name).write_bytes(data);total+=len(data)
  assert after[6]==0 and after[:6]==(before/'creatures.bin').read_bytes()[:6]
  assert struct.unpack_from('<h',after,0x6c)[0]==32766
  pins.update({r['path']:r['sha256']for r in trace['listing_pins']});steps+=trace['report']['steps']
  cases.append(dict(seed=seed,frame=frame,trajectory=run.relative_to(P).as_posix(),
   trajectory_sha256=sha(run/'result.json'),compilation_sha256=sha(run/'compilation.json'),
   binary_sha256=sha(run/'probe.exe'),compilation_source_pins=compiled['source_pins'],
   literal_instructions=trace['report']['steps'],compared_bytes=sum(map(len,output.values())),
   expected_pins={name:sha(destination/name)for name in NAMES},
   final_kind=after[6],current_speed=struct.unpack_from('<h',after,0x41)[0],
   final_count=struct.unpack_from('<h',output['backing.bin'],0x3550)[0]))
  print(seed,frame,'PASS',flush=True)
 scripts=[Path(__file__),Path(__file__).with_name('mule-bones-retirement-source.py'),
  Path(__file__).with_name('mule-bones-source.py'),Path(__file__).with_name('mule-mortality-source.py'),
  Path(__file__).with_name('creature-hungry-source.py'),Path(__file__).with_name('initial-creature-source.py'),
  Path(__file__).with_name('actor-environment-terrain-target.py')]
 result=dict(status='passed-archived-whole-world-retirement',utc=datetime.now(timezone.utc).isoformat(),
  cases=cases,compared_bytes=total,literal_instructions=steps,listing_pins=pins,
  review_pins={p.relative_to(P).as_posix():sha(p)for p in scripts},fixture_stream_sha256=digest.hexdigest(),
  original_execution=False,gui_execution=False,
  scope='Complete two captured final callbacks: numeric epilogue,8ACC0,724C0,kind clear,count,all seven owned buffers. Only process-pointer gaps31..38 are restored to caller priors.29100 executes against explicit empty native UI;2C200 is an explicit presentation port. Display phase11 and mandatory29100/2C200 requests are metadata, not widget execution. Archived compiled source hashes are verified; current source identity is not claimed.')
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8');tmp.replace(out)
 print(json.dumps({k:result[k]for k in ['status','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
