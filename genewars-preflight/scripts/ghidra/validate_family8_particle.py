"""Code1A factory and complete flat-land callback trajectories vs literal listings.

The retirement service is an explicit port; generic world cleanup is separately
tested by the world adapter. No bundled executable is launched.
"""
from pathlib import Path
import hashlib, importlib.util, json, random, struct, subprocess, uuid
P=Path(__file__).resolve().parents[2]
sha=lambda b:hashlib.sha256(b).hexdigest()
spec=importlib.util.spec_from_file_location('particle_source',P/'research/variants/initial-creature-source.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Source(m.Source):
    def get(self,o):
        if self.pc==0x4b7be and o=='0x00049d30':
            self.retired=True;self.store(self.r['EAX']+6,1,0);return 0x2e0000
        return super().get(o)
model=Source()
files=[P/'private/ghidra'/x for x in [
    'mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt',
    'mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt',
    'plant-command/instructions-0004b630.txt','mission-array-refs/instructions-0004b3b0.txt',
    'actor-activation/instructions-0004b090.txt','landing-contact/instructions-0001e600.txt',
    'landing-contact/instructions-0001f030.txt']]
model.code.update(m.m.Model(files,b'').code);model.code[0x2e0000]=('RET',[])
order=sorted(model.code);model.next=dict(zip(order,order[1:]))
mem=(P/'private/ghidra/validated/memory-.object3.bin').read_bytes()
code=(P/'private/ghidra/validated/memory-.object1.bin').read_bytes()
assert mem[0x4eb4+26*6:0x4eb4+27*6]==bytes([0,0,35,255,0,0])
assert struct.unpack_from('<I',code,0x3b524+20*4)[0]==0x4b7d5
sine=struct.unpack_from('<2048i',mem,0x7ee8);cosine=struct.unpack_from('<2048i',mem,0x86e8)
atan=struct.unpack_from('<257H',code,0x9e134)
row=next(x for x in json.loads((P/'research/integration/sprite-corpus.json').read_text())['results'] if x['entry_id']=='root/2/disc/59')
db=bytearray((P/row['private_database']).read_bytes());dbsha=sha(db)
assert dbsha=='f45f1201e50ce67199f1e1e8448d250f221cad71f1328ee44b7bacf9ee62c253'
_,_,_,frames,animations=struct.unpack_from('<H4I',db)
assert struct.unpack_from('<IIBB',db,animations+26*10)==(2030,2086,5,0)
assert all(db[frames+2030+i*14+10]==1 for i in range(5))
base=0x610000
for i in range(struct.unpack_from('<H',db)[0]):
    for o in [0,4]:struct.pack_into('<I',db,animations+i*10+o,base+frames+struct.unpack_from('<I',db,animations+i*10+o)[0])
flat=struct.pack('<HhBBBBBhhB',8192,3136,0,7,0,0,0,0,0,0)
state0={base+i:b for i,b in enumerate(db)}
for c in range(16384):state0.update({0x11c2be+c*14+i:b for i,b in enumerate(flat)})
state0.update({0x116818+i:b for i,b in enumerate(struct.pack('<I',base+animations))})
r=random.Random(26);factory_data=bytearray();factory_expected=bytearray();life_data=bytearray();life_expected=bytearray();steps=0;lifetimes=[]
for case in range(48):
    slot=case%8;at=[20864,13696];height=20000+case*17
    angles=[(case*29)%2048,(case*47)%2048,4+(case*5)%96]
    seed=(case*1367)&65535;ring=case%32;draws=0xffffffff-case
    prior=bytearray(r.randbytes(81));prior[6]=0
    native=bytearray(prior);struct.pack_into('<I',native,35,0x112254)
    state=state0.copy();state.update({0x1688ae+i*81+6:int(i<slot)*5 for i in range(200)})
    state.update({0x1688ae+slot*81+i:b for i,b in enumerate(native)})
    for addr,data in [(0x220000,struct.pack('<3H',*angles)),(0x11c2b4,struct.pack('<h',slot)),
        (0x300004,bytes(16)),(0x118782,struct.pack('<I',draws))]:state.update({addr+i:b for i,b in enumerate(data)})
    state[0x11872c]=ring
    report,_,_=model.run(entry=0x48790,seed=seed,state_before=state,
        registers={'EAX':26,'EDX':at[0]|at[1]<<16,'EBX':height,'ECX':0x220000})
    a=bytearray(model.load(0x1688ae+slot*81+i,1)for i in range(81))
    normalized=bytearray(a);normalized[31:35]=bytes(4);normalized[35:39]=prior[35:39]
    factory_data+=prior+struct.pack('<HhhiiB3H HBI',slot,*at,6272,height,0,*angles,seed,ring,draws)
    factory_expected+=normalized+struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
    assert report['draws']==1;steps+=report['steps']
    for frame in range(1,100):
        life_data+=a;state=state0.copy();state.update({0x1688ae+i:b for i,b in enumerate(a)})
        model.retired=False
        report,_,_=model.run(entry=0x4b630,state_before=state,registers={'EAX':0x1688ae})
        a=bytearray(model.load(0x1688ae+i,1)for i in range(81));a[6]=5
        life_expected+=bytes([model.retired])+a;steps+=report['steps'];assert report['draws']==0
        if model.retired:break
    else:raise AssertionError('particle did not retire')
    lifetimes.append(frame)
work=P/'private/review/family8-particle'/uuid.uuid4().hex;work.mkdir(parents=True)
runtime=P/'rust/crates/genewars-runtime/src';pins={x.name:sha(x.read_bytes())for x in [runtime/'landing.rs',runtime/'family8_particle.rs']}
h=work/'probe.rs';h.write_text(f'#[path="{(runtime/"landing.rs").as_posix()}"]mod landing;\n#[path="{(runtime/"family8_particle.rs").as_posix()}"]mod particle;\n'+r'''
use std::io::{Read,Write};use landing::*;
fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let q=*p;*p+=n;&b[q..*p]}
fn w(b:&[u8],p:&mut usize)->u16{u16::from_le_bytes(take(b,p,2).try_into().unwrap())}
fn d(b:&[u8],p:&mut usize)->i32{i32::from_le_bytes(take(b,p,4).try_into().unwrap())}
fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let mut sin=[0;2048];let mut cos=[0;2048];let mut atan=[0;257];for t in [&mut sin,&mut cos]{for v in t{*v=d(&b,&mut p)}}for v in &mut atan{*v=w(&b,&mut p)}let mut out=Vec::new();let n=d(&b,&mut p);for _ in 0..n{let prior=take(&b,&mut p,81).try_into().unwrap();let slot=w(&b,&mut p);let at=[w(&b,&mut p)as i16,w(&b,&mut p)as i16];let ground=d(&b,&mut p);let height=d(&b,&mut p);let shade=take(&b,&mut p,1)[0];let angles=[w(&b,&mut p),w(&b,&mut p),w(&b,&mut p)];let rng=LandingRng{seed:w(&b,&mut p),ring:take(&b,&mut p,1)[0],draw_count:d(&b,&mut p)as u32};let mut pool=[[0;81];200];for r in &mut pool[..slot as usize]{r[6]=5}pool[slot as usize]=prior;let a=particle::plan_factory(&pool,slot as i16,at,ground,shade,height,angles,None,1024,rng,&sin,&cos).unwrap();assert_eq!(a.factory.animation_index,26);assert_eq!(a.draw.journal_code,0x229);out.extend(a.factory.after_nonpointer);out.extend(a.rng_after.seed.to_le_bytes());out.push(a.rng_after.ring);out.extend(a.rng_after.draw_count.to_le_bytes());}let n=d(&b,&mut p);for _ in 0..n{let before=take(&b,&mut p,81).try_into().unwrap();let a=particle::plan_flight(&before,6272,0,1632,[3136;4],128,&atan).unwrap();let a=particle::plan_animation_after_physics(&a.after).unwrap();out.push(a.retirement_required as u8);out.extend(a.after);}assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}
''',encoding='utf-8')
exe=work/'probe.exe';subprocess.run([str(P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True)
assert all(sha((runtime/n).read_bytes())==v for n,v in pins.items())
data=struct.pack('<2048i2048i257HI',*sine,*cosine,*atan,48)+factory_data+struct.pack('<I',len(life_data)//81)+life_data
(work/'input.bin').write_bytes(data)
expected=factory_expected+life_expected
out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout
(work/'expected.bin').write_bytes(expected);(work/'actual.bin').write_bytes(out)
assert out==expected,(len(out),len(expected),next((i,a,b)for i,(a,b)in enumerate(zip(out,expected))if a!=b))
result={'status':'passed','factory_cases':48,'callback_cases':len(life_data)//81,'compared_bytes':len(out),
    'literal_instructions':steps,'lifetimes':lifetimes,'output_sha256':sha(out),'rust_sources':pins,
    'EFF26_sha256':dbsha,'source_pins':{f.relative_to(P).as_posix():sha(f.read_bytes())for f in files+model.files},
    'scope':'Full free-slot48790 and airborne4B630/6DF30 through EFF26 retirement request. No generic cleanup or occupied-ground collision claim.'}
(P/'research/integration/family8-particle-validation.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:result[k]for k in ['status','factory_cases','callback_cases','compared_bytes','literal_instructions']}))
