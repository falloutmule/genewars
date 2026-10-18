"""Captured Krell state10 marker against independent10EA0 common pixel oracle."""
from pathlib import Path
import json,hashlib,subprocess,sys,uuid,struct
import validate_family8_state4_pixels as oracle
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 record_path=P/'private/review/krell-zone-bootstrap-001/after/family8.bin';record=record_path.read_bytes()[:102]
 assert (record[6],record[0x55],record[0x56],record[0x19],record[0x1b],struct.unpack_from('<H',record,0x15)[0])==(8,2,10,0,0,0x4020)
 data=oracle.checked_bytes(oracle.ETHEREAL,oracle.ETHEREAL_SHA,91123);fade=oracle.checked_bytes(oracle.FADE,oracle.FADE_SHA,16384);gla=oracle.checked_bytes(oracle.GLA,oracle.GLA_SHA,65536)
 for rel,digest in oracle.LISTING_PINS.items():assert sha(P/rel)==digest
 listing=P/'private/ghidra/countdown-expiry/instructions-00010ea0.txt';text=listing.read_text()
 for line in ['00010f1e CMP DL,0x2','00010f21 JZ 0x000111db','000111fe CMP AL,0x2','00011200 JZ 0x0001115a','0001115d CMP BH,0xa','00011160 JZ 0x00011206']:assert line in text
 work=P/'private/integration/family8-zone-pixels'/uuid.uuid4().hex;work.mkdir(parents=True)
 deps=P/'private/build/rust-target/release/deps';libs={name:max(deps.glob(f'libgenewars_{name}-*.rlib'),key=lambda p:p.stat().st_mtime_ns)for name in ['runtime','render','assets']}
 harness=P/'scripts/desktop/family8_zone_pixel_harness.rs';implementation=P/'rust/crates/genewars-desktop/src/bin/support/family8_scene.rs'
 pins={p.relative_to(P).as_posix():sha(p)for p in [record_path,listing,oracle.ETHEREAL,oracle.FADE,oracle.GLA,harness,implementation,Path(__file__),P/'scripts/desktop/validate_family8_state4_pixels.py']}
 command=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(harness),'-L',f'dependency={deps}','-o',str(work/'probe.exe')]
 for name,lib in libs.items():command+=['--extern',f'genewars_{name}={lib}']
 built=subprocess.run(command,env=environment(),capture_output=True,text=True,timeout=90);(work/'build.txt').write_text(built.stdout+built.stderr);assert built.returncode==0,built.stderr
 run=subprocess.run([str(work/'probe.exe'),str(oracle.ETHEREAL),str(oracle.FADE),str(oracle.GLA),str(record_path),str(work/'pixels')],capture_output=True,text=True,timeout=90);(work/'probe.txt').write_text(run.stdout+run.stderr);assert run.returncode==0,(run.stdout,run.stderr)
 lines=run.stdout.splitlines();assert lines[-1]=='atomic_rejections=9';reports={line.split()[0]:dict(v.split('=')for v in line.split()[1:])for line in lines[:-1]}
 height=struct.unpack_from('<i',record,0x27)[0]-struct.unpack_from('<i',record,0x11)[0];comparisons=[]
 for frame in range(11):
  for variant,origin in enumerate([(280,430),(120,700),(1,1),(637,475)]):
   key=f'{frame}-{variant}';expected=oracle.oracle(data,fade,gla,(key,0,4,frame,origin,record[0x17],height));actual=(work/'pixels'/f'{key}.raw').read_bytes();assert actual==expected['pixels'],key
   for field in ['shadow','color','shadow_clipped','color_clipped','global_animation','virtual_height','effective_shade','color_y']:assert int(reports[key][field])==expected[field],(key,field)
   comparisons.append(dict(frame=frame,origin=origin,sha256=hashlib.sha256(actual).hexdigest(),shadow=int(reports[key]['shadow']),color=int(reports[key]['color'])))
 assert pins=={path:sha(P/path)for path in pins}
 receipt=dict(status='passed',private_run=work.relative_to(P).as_posix(),cases=len(comparisons),compared_bytes=len(comparisons)*311032,atomic_rejections=9,pins=pins,compiled_dependencies={p.relative_to(P).as_posix():sha(p)for p in libs.values()},comparisons=comparisons,scope='Captured marker0 selector2/state10/flags4020 at explicit diagnostic origins; actual frame0 and metadata-only variants frames1..10.10EA0 skips decoration and takes common shadow/color route with Offset80/global animation8.',limitations=['This state10-only API rejects11..13; states11/12 are verified separately with explicit108E0 context','Projected origin is explicit diagnostic input, not native camera acceptance','Factory height32000 puts foreground500 pixels above ground; preserve source height','No timing, marker callback, campaign or mission acceptance claim'],original_execution=False)
 (work/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');(P/'research/integration/family8-zone-pixels.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k]for k in ['status','cases','compared_bytes','atomic_rejections','private_run']}))
if __name__=='__main__':main()
