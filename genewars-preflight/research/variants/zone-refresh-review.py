"""Literal zone refresh79D50 without marker creation; explicit renderer-cache and sqrt ports."""
from pathlib import Path
import argparse,importlib.util,json,hashlib,struct
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
OVER={1:'creatures.bin',3:'actors.bin',4:'buildings.bin',5:'effects.bin',8:'family8.bin'}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
class Source(m.Source):
 def get(self,o):
  if o=='0x00084b40':self.vertices.append(self.r['EDX'])
  return super().get(o)
def main():
 ap=argparse.ArgumentParser();ap.add_argument('fixture',type=Path);args=ap.parse_args();d=args.fixture
 before={f.name:f.read_bytes()for f in(d/'before').glob('*.bin')};actual={f.name:f.read_bytes()for f in(d/'after').glob('*.bin')}
 if not before or not actual:return {'status':'blocked','reason':'missing before/after fixture'}
 model=Source();extra=['terrain-map-refs/instructions-000'+x+'.txt'for x in ['79d50','8c290','84b40','7b240','101d0']]+['camera-player-input/instructions-0008b800.txt','countdown-expiry/instructions-0001ee80.txt','mission-tail-init/instructions-00010060.txt','entity-plant/instructions-0001f240.txt','entity-plant/instructions-0001f280.txt','landing-contact/instructions-0001f030.txt','countdown-expiry/instructions-0001ea90.txt']
 files=[P/'private/ghidra'/f for f in extra];model.code.update(m.m.Model(files,b'').code)
 # Cache lookup returns visible/handled; actual84B40 numeric tail runs.
 model.code[0x847d0]=('MOV',['EAX','1']);model.code[0x847d1]=('RET',[])
 zone=0x118d60+0x2ff8
 wrapper=[('MOV',['EAX',hex(zone)]),('MOV',['EDX','1']),('CALL',['0x00079d50']),('RET',[])]
 for i,op in enumerate(wrapper):model.code[0x2d0000+i]=op
 order=sorted(model.code);model.next=dict(zip(order,order[1:]))
 b=before['backing.bin'];state={0x118d60+i:v for i,v in enumerate(b[:0x355e])};state.update({0x11c2be+i:v for i,v in enumerate(b[0x355e:0x3b55e])});state.update({0x118d74+i:v for i,v in enumerate(before['players.bin'])})
 for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
  for slot in range(count):
   if kind in OVER:native=bytearray(before[OVER[kind]][slot*width:(slot+1)*width])
   else:raw=b[start+slot*stride:start+(slot+1)*stride];native=bytearray(raw[:31]+bytes(8)+raw[31:])
   vt={1:0x111fec,2:0x1120ac,3:0x112194,5:0x112254,8:0x11242c}.get(kind)
   if kind==4:
    vt=0x2e1400;state.update({vt+0x6c+i:v for i,v in enumerate(struct.pack('<I',0x1f280))})
   if vt:struct.pack_into('<I',native,35,vt)
   state.update({address+slot*width+j:v for j,v in enumerate(native)})
 #10060 writes only a typed animation pointer; no frame data consumed by factory.
 state.update({0x116820+i:v for i,v in enumerate(struct.pack('<I',0x500000))})
 context=json.loads((d/'context.json').read_text()) if(d/'context.json').exists() else {}
 water=context.get('water_depth',struct.unpack_from('<h',b,0x1c63)[0]);state.update({0x11a9c3+i:v for i,v in enumerate(struct.pack('<h',water))})
 state.update({0x106604+i:v for i,v in enumerate(struct.pack('<3i',*context.get('lighting',[181,0,-181])))})
 seed,ring,draws=struct.unpack('<HBI',before['rng.bin']);state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))});model.vertices=[]
 report,_,trace=model.run(counter=0,seed=seed,state_before=state,entry=0x2d0000,instruction_budget=3000000)
 assert before['family8.bin'][0x55:0x57]==bytes([2,12]),'required occupied marker fixture'
 assert trace.count(0x101d0)==0,'refresh unexpectedly creates marker'
 expected={k:bytearray(v)for k,v in before.items()};out=expected['backing.bin'];out[0x3559]=model.load(0x11c2b9,1);out[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
 for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
  for slot in range(count):
   native=bytearray(model.load(address+slot*width+j,1)for j in range(width))
   if kind in OVER:
    name=OVER[kind];native[31:39]=before[name][slot*width+31:slot*width+39];expected[name][slot*width:(slot+1)*width]=native
   elif kind==2:out[start+slot*stride+23]=native[23]
 expected['rng.bin']=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
 assert set(expected)==set(actual), 'fixture member mismatch'
 assert all(len(expected[k])==len(actual[k])for k in expected), 'fixture size mismatch'
 diffs={name:[(i,a,z)for i,(a,z)in enumerate(zip(data,actual[name]))if a!=z][:20]for name,data in expected.items()if name in actual and data!=actual[name]}
 for name,data in expected.items():(d/('source-'+name)).write_bytes(data)
 return {'status':'error'if diffs else'passed','fixture':str(d),'harness_sha256':sha(Path(__file__)),'context':{'water_depth':water,'lighting':context.get('lighting',[181,0,-181]),'lighting_origin':'root frozen loader context'},'reference_executable_sha256':'4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba','object_pins':{f:sha(P/f)for f in ['private/ghidra/validated/memory-.object1.bin','private/ghidra/validated/memory-.object3.bin']},'differences':diffs,'steps':report['steps'],'vertices':len(model.vertices),'shade_samples':trace.count(0x7b240),'factory_calls':trace.count(0x101d0),'compared_bytes':sum(len(v)for k,v in expected.items()if k in actual),'source_pins':{str(f.relative_to(P)):sha(f)for f in model.files+files},'fixture_pins':{str(f.relative_to(d)):sha(f)for role in ['before','after']for f in(d/role).glob('*.bin')},'ports':['847D0 cachelookup return1; actual84B40 numeric tail','97910 integer square root via math.isqrt','typed family vtables and10060 animation pointer'],'limits':['one occupied zone index0, existing selector2 marker state12; no marker factory','pointerbytes31..39 normalized; resource loading not tested','no missionacceptance claim']}
if __name__=='__main__':
 try:r=main()
 except Exception as e:r={'status':'error','error':repr(e)}
 Path(__file__).with_suffix('.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items()if k not in ['source_pins','fixture_pins']},indent=2));sys_exit=0 if r['status']=='passed'else 1
 raise SystemExit(sys_exit)
