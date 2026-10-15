"""Source-parametric hunger40 independence, never campaign progress.
Creates immutable metadata-labeled synthetic worlds from already valid captures;
compares one original callback and existing exact8buffer replay for each case.
"""
from pathlib import Path
import argparse,hashlib,json,os,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
BUFFERS=['backing.bin','players.bin','actors.bin','creatures.bin','effects.bin','family8.bin','buildings.bin','rng.bin']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--source-only',action='store_true');ap.add_argument('--test-exe',type=Path,help='existing compiled runtime test executable; no compilation');args=ap.parse_args()
 work=P/'private/review/mule-pair-hunger-flags'/uuid.uuid4().hex;work.mkdir(parents=True)
 pair=P/'private/review/krell-mule-callback/c8cbf4a2af7f477da4c960f08762f1f0';retry=P/'private/review/authentic-krell-134/frame-1927-slot-0-blocked-before';assert all((pair/name).is_file()and(retry/name).is_file()for name in BUFFERS)
 pins={p.relative_to(P).as_posix():sha(p)for root in [pair,retry]for p in [root/name for name in BUFFERS]};rows=[]
 configs=[('state10-positive-countdown',pair,2668,4),('postbirth-state8-positive-delay',pair,2668,4),('retained-pair-retry',retry,1927,0)]
 for name,base,frame,slot in configs:
  case=work/name;case.mkdir();before=case/'before';before.mkdir();buffers={n:bytearray((base/n).read_bytes())for n in BUFFERS};changes=[];creatures=buffers['creatures.bin']
  def patch(actor,offset,data,label):
   at=actor*124+offset;old=bytes(creatures[at:at+len(data)]);creatures[at:at+len(data)]=data
   if old!=data:changes.append({'slot':actor,'offset':hex(offset),'field':label,'before':old.hex(),'after':data.hex()})
  def word(actor,offset,value,label):patch(actor,offset,struct.pack('<h',value),label)
  if name=='retained-pair-retry':
   assert creatures[97]==0x60;patch(0,97,b'\x20','hungerbit40off-only')
  else:
   for actor,high in [(4,0x20),(0,0x60)]:
    assert creatures[actor*124+6]==1 and creatures[actor*124+87]==1
    patch(actor,94,bytes([10 if name.startswith('state10')else 8]),'synthetic-state')
    patch(actor,96,bytes([0x0c,high]),'readiness2004-retain-hunger-asymmetry')
    patch(actor,82,b'\0','association-kind-cleared')
    word(actor,108,32 if name.startswith('state10')else 10,'positive-countdown')
    if name.startswith('state10'):
     for offset in [45,47,49,61,63,65]:word(actor,offset,0,'stationary-speed-or-velocity')
  for n,data in buffers.items():(before/n).write_bytes(data)
  metadata={'classification':'synthetic_source_parametric_fixture_NOT_campaign_progress','base':base.relative_to(P).as_posix(),'frame':frame,'slot':slot,'changes':changes,'scope':'Only one callback; all other families/creatures frozen. Pairworld derives from isolated146 source-compared rendezvous. No authentic countdown/birth reachability claim.'};(case/'fixture.json').write_text(json.dumps(metadata,indent=2)+'\n')
  source_run=subprocess.run([sys.executable,'-B','research/variants/krell-mule-callback-review.py','--before',str(before),'--frame',str(frame),'--slot',str(slot)],cwd=P,capture_output=True);(case/'source.log').write_bytes(source_run.stdout+source_run.stderr);source_run.check_returncode();source=json.loads(source_run.stdout);source_directory=Path(source['output']);assert source['status']=='source_prepared',source.get('reason')
  row={'case':name,'frame':frame,'slot':slot,'classification':metadata['classification'],'input':before.relative_to(P).as_posix(),'source_output':source_directory.relative_to(P).as_posix(),'source_instructions':source['literal_instructions'],'source_draws':source['draws']};rows.append(row)
  if args.source_only:continue
  after=case/'rust-after';env=environment();env={k:v for k,v in env.items()if not k.startswith('GENEWARS_CAPTURED_MULE_')and k!='GENEWARS_MULE_CALLBACK_FIXTURE'};env.update(CARGO_BUILD_JOBS='2',CARGO_PROFILE_TEST_OPT_LEVEL='1',CARGO_PROFILE_TEST_DEBUG='0',GENEWARS_CAPTURED_MULE_INPUT=str(before),GENEWARS_CAPTURED_MULE_OUTPUT=str(after),GENEWARS_CAPTURED_MULE_FRAME=str(frame),GENEWARS_CAPTURED_MULE_SLOT=str(slot),GENEWARS_CAPTURED_MULE_EXPECT='passed')
  if args.test_exe:command=[str(args.test_exe.resolve()),'actor_world::captured_mule_tests::captured_mule_callback','--ignored','--nocapture','--test-threads=1']
  else:command=[sys.executable,'-B','scripts/cargo.py','test','-p','genewars-runtime','captured_mule_callback','--offline','--locked','--','--ignored','--nocapture','--test-threads=1']
  rust_run=subprocess.run(command,cwd=P,env=env,capture_output=True);(case/'rust.log').write_bytes(rust_run.stdout+rust_run.stderr);rust_run.check_returncode();rust=json.loads((after/'result.json').read_text());assert rust['status']=='passed'and rust['input_roundtrip'],rust
  mismatches={};compared=0
  for n in BUFFERS:
   expected=(source_directory/n).read_bytes();actual=(after/n).read_bytes();compared+=len(expected);offsets=[i for i,(x,y)in enumerate(zip(expected,actual))if x!=y]
   if len(expected)!=len(actual)or offsets:mismatches[n]={'expected':len(expected),'actual':len(actual),'count':len(offsets),'first':offsets[:12]}
  row.update(rust_output=after.relative_to(P).as_posix(),rust_draws=rust['rng_draws'],compared_bytes=compared,mismatches=mismatches);(case/'comparison.json').write_text(json.dumps(row,indent=2)+'\n');assert not mismatches,row;assert rust['rng_draws']==source['draws']
 result={'status':'source_prepared'if args.source_only else 'passed','scope':'Explicit SYNTHETIC hungerbit40-off domains, one original/live Rust callback per case with validlinked8buffer roundtrip. State10/postbirth8 fields artificially varied; retryonly61bit40off. NOT campaign progress, native acceptance, full shared scheduler or authentic birth reachability.','rows':rows,'input_pins':pins,'validator_sha256':sha(Path(__file__)),'private_run':work.relative_to(P).as_posix()};(work/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'cases':len(rows),'compared_bytes':sum(r.get('compared_bytes',0)for r in rows),'source_draws':[r['source_draws']for r in rows],'private_run':result['private_run'],'scope':'SYNTHETIC parameter proof only; not campaign progress'}))
if __name__=='__main__':main()
