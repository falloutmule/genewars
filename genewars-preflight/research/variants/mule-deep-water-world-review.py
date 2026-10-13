"""Compare the complete captured Mule5859 transaction with literal source."""
from pathlib import Path
from datetime import datetime,timezone
import subprocess,sys,json,hashlib
P=Path(__file__).resolve().parents[2];W=P/'private/review/mule-deep-water-world'
TRAJECTORY=P/'private/review/family8-trajectory/76d5f278b3234c69ac0f562782193e51'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 source=Path(__file__).with_name('mule-deep-water-world-source.py')
 r=subprocess.run([sys.executable,'-B',str(source)],cwd=P,capture_output=True,timeout=120);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'))
 proof=json.loads((W/'source.json').read_text());provenance=json.loads((TRAJECTORY/'result.json').read_text());assert provenance['seed']==65535 and provenance['completed_frames']>=5862
 assert sha(TRAJECTORY/'probe.exe')==provenance['binary_sha256']
 compared=0;inputs={};outputs={}
 for name in ['backing.bin','creatures.bin','actors.bin','effects.bin','family8.bin','players.bin','rng.bin']:
  captured_before=TRAJECTORY/'frame5859/before-mule'/name;captured_after=TRAJECTORY/'frame5859/after-mule'/name
  assert sha(captured_before)==provenance['captures'][f'frame5859/before-mule/{name}']
  assert sha(captured_after)==provenance['captures'][f'frame5859/after-mule/{name}']
  assert (W/'before'/name).read_bytes()==captured_before.read_bytes(),name
  actual=captured_after.read_bytes();expected=(W/name).read_bytes();assert actual==expected,(name,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:8]);compared+=len(actual)
  inputs[name]=sha(captured_before);outputs[name]=sha(captured_after)
 result=dict(status='passed-whole-mule-deep-water-transaction',utc=datetime.now(timezone.utc).isoformat(),frame=5859,compared_bytes=compared,literal_instructions=proof['literal_instructions'],input_hashes=inputs,output_hashes=outputs,
  trajectory_provenance_sha256=sha(TRAJECTORY/'result.json'),captured_binary_sha256=provenance['binary_sha256'],compilation_source_pins=provenance['source_pins'],listing_pins=proof['listing_pins'],
  review_pins={x.relative_to(P).as_posix():sha(x)for x in [Path(__file__),source,Path(__file__).with_name('creature-hungry-source.py'),Path(__file__).with_name('initial-creature-source.py'),P/'private/integration/game-runtime/effects/EFF.ANB']},
  original_execution=False,gui_execution=False,scope='Full owned Mule callback5859 including water15 factory, map/count/links/native pools/players/RNG. Pointer gaps normalized to prior owned bytes. Otherfamilycallbacks and subsequent effects scheduler are separate captured-pass checks.')
 Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:result[k]for k in ['status','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
