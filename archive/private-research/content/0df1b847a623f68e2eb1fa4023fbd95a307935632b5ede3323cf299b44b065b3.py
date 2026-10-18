from pathlib import Path
import sys,subprocess,shutil,uuid,json,hashlib
P=Path.cwd();sys.path.insert(0,str(P/'scripts'));from cargo import environment
W=P/'private/review/actor-idle-relocation'/uuid.uuid4().hex;W.mkdir()
def run(a):
 r=subprocess.run(a,cwd=P,env=environment(),capture_output=True,text=True);assert r.returncode==0,(r.stdout,r.stderr);return r.stdout
pins={}
for c in ['assets','core','runtime']:
 t=W/c;shutil.copytree(P/f'rust/crates/genewars-{c}/src',t)
 for f in t.rglob('*.rs'):pins[f'{c}/{f.relative_to(t)}']=hashlib.sha256(f.read_bytes()).hexdigest()
 if c=='runtime':
  for f in ['actor_pool.rs','creature_world.rs','effect_pool.rs','family8_pool.rs']:
   q=t/f;s=q.read_text();s=s.replace('\n    records: [','\n    pub(crate) records: [').replace('\n    bindings: [','\n    pub(crate) bindings: [');q.write_text(s)
  q=t/'actor_world.rs';q.write_text(q.read_text()+'\nmod relocation_review {use super::*;\n'+(P/'private/review/turret-projectile/relocation-review.inc.rs').read_text()+'\n}\npub use relocation_review::review_relocation;\n')
 a=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-type','rlib','--crate-name',f'genewars_{c}',str(t/'lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{c}.rlib')]
 if c=='runtime':
  for dep in ['assets','core']:a+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(a)
d=P/'private/review/bootstrap-context/fbc3cdc5f63a4ebfa01d0e78c2cf5354/seed65535-players1-timer1000/frame14111/boundary'
f=W/'probe.rs';f.write_text('fn main(){genewars_runtime::actor_world::review_relocation(std::path::Path::new(r"'+str(P)+'"),std::path::Path::new(r"'+str(d)+'"),std::path::Path::new(r"'+str(W)+'"));}')
run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(f),'-L',f'dependency={W}','--extern',f'genewars_runtime={W}/libgenewars_runtime.rlib','-o',str(W/'probe.exe')]);print(run([str(W/'probe.exe')]))
a=bytearray((W/'rust-callback.bin').read_bytes());b=bytearray((W.parent/'source-callback.bin').read_bytes());ff=bytearray((W.parent/'source-callback-ff.bin').read_bytes());assert b==ff
for start,end in [(31,39),(176,180),(192,200)]:a[start:end]=b[start:end]=bytes(end-start)
assert a==b,[(i,x,y)for i,(x,y)in enumerate(zip(a,b))if x!=y]
(W/'result.json').write_text(json.dumps(dict(status='passed',production_pins=pins),indent=2));print(W)
