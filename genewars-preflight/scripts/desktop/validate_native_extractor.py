"""Native ordinary-input Extractor probe; explicit development startup, not mission acceptance.
Use a frozen ordinary-command probe binary; this tool never rebuilds production.
"""
import argparse,hashlib,json,shutil,subprocess,sys,uuid
from datetime import datetime,timezone
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
INPUT='''observe menu
pointer 312 140
left down
left up
wait
observe loaded
key left 4
key up 3
key space 189
key tab
observe selected
pointer 320 234
right down
right up
observe ordered
key space 100
observe arrived
key c
observe choices
pointer 200 445
left down
left up
observe assigned
key space 100
observe construction100
key space 200
observe construction300
key space 300
observe construction600
key space 400
observe final
'''
NAMES=['menu','loaded','selected','ordered','arrived','choices','assigned','construction100','construction300','construction600','final']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--probe',required=True,type=Path);parser.add_argument('--expect',required=True,choices=['completed','blocked']);parser.add_argument('--border-policy',required=True,choices=['mapped-prior','generated']);args=parser.parse_args()
 probe=args.probe if args.probe.is_absolute() else ROOT/args.probe
 binary_source=probe/'genewars-game.exe'
 work=ROOT/'private/integration/native-extractor'/uuid.uuid4().hex;work.mkdir(parents=True)
 binary=work/'genewars-game.exe';shutil.copy2(binary_source,binary);assert sha(binary)==sha(binary_source)
 (work/'input.txt').write_text(INPUT)
 border=ROOT/'private/integration/game-runtime/construction'/('border.bin' if args.border_policy=='mapped-prior' else 'terrain-patterns.bin');border_input_sha=sha(border)
 border_sha='9f1dcbc35c350d6027f98be0f5c8b43b42ca52b7604459c0c42be3aa88913d47' if args.border_policy=='mapped-prior' else '422f124eac806197ba95d2438516284c3bf828a1db8e8aabe330661fff23ef8a'
 expected_input='9f1dcbc35c350d6027f98be0f5c8b43b42ca52b7604459c0c42be3aa88913d47' if args.border_policy=='mapped-prior' else 'd848e423b7a76a2c2dbf517fc595e33a789df7382697b56f382cc7606b18939e'
 assert border_input_sha==expected_input,'border policy input hash mismatch'
 receipt=dict(status='error',utc=datetime.now(timezone.utc).isoformat(),scope='native development seed5428 ordinary-command construction, not authentic mission acceptance',authentic_missions_completable=0,probe=str(probe.relative_to(ROOT)),binary_sha256=sha(binary),input_sha256=sha(work/'input.txt'),work=str(work.relative_to(ROOT)),expected=args.expect,construction_border_sha256=border_sha,border_policy=args.border_policy,border_input_sha256=border_input_sha,border_policy_provenance='explicit operator selection for frozen probe binary; table output hash independently source-validated',runs=[],original_execution=False,limitations=['Development startup, four-player scheduling and explicit replay RNG remain distinct from original bootstrap','C chooser/Tab selection/right-click movement are port controls; original input mapping unresolved','No mission outcome or campaign transition is asserted'])
 try:
  for name in ['first','repeat']:
   folder=work/name
   command=[str(binary),str(ROOT/'private/integration/game-runtime'),'--landing-seed','5428','--native-input-replay',str(work/'input.txt'),'--observations',str(folder),'--seconds','300']
   result=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=320)
   (work/f'{name}.stdout').write_text(result.stdout);(work/f'{name}.stderr').write_text(result.stderr)
   result.check_returncode()
   frames={n:json.loads((folder/f'{n}.json').read_text())for n in NAMES}
   assert frames['menu']['scene']=='Menu'
   assert frames['loaded']['mission']['steps']==0
   assert all(frames[a]['presented_frames']<frames[b]['presented_frames']for a,b in zip(NAMES,NAMES[1:]))
   assert frames['selected']['mission']['selected_actor']==0
   ordered=frames['ordered']['mission']['actor_states'][0]
   assert ordered['pending_destination']==[19839,12928] or ordered['destination']==[19839,12928],ordered
   arrived=frames['arrived']['mission']['actor_states'][0]
   assert arrived['position']==[19839,12928] and arrived['state']==1 and not arrived['pending_move'],arrived
   assert frames['choices']['construction_choices']==[11] and frames['choices']['construction_menu_open']
   assigned=frames['assigned']['mission']['actor_states'][0]
   assert assigned['assignment_pending'] and assigned['building_selector']==11
   assert not frames['assigned']['mission']['buildings'] and not frames['assigned']['construction_menu_open']
   assert frames['assigned']['mission']['steps']==289
   final=frames['final']['mission'];block=final['blocked']
   if args.expect=='blocked':assert block is not None,'expected explicit unsupported/gameplay boundary'
   else:
    assert block is None and final['steps']==1289
    assert len(final['buildings'])==1
    building=final['buildings'][0]
    assert building['selector']==11 and building['flags_word_31']&1 and building['progress_word_2b']==280,building
    assert final['terrain_jobs']['count']==0 and final['actor_states'][0]['state']==1
    assert final['player_funds'][0]['balance']<frames['assigned']['mission']['player_funds'][0]['balance']
    assert any(f['mission']['buildings'] and not f['mission']['buildings'][0]['flags_word_31']&1 for n,f in frames.items()if n.startswith('construction')),'missing visible incomplete-building observation'
   for n,f in frames.items():assert f['error'] is None,(n,f['error'])
   receipt['runs'].append({n:dict(bmp_sha256=sha(folder/f'{n}.bmp'),world_sha256=None if f['mission']is None else f['mission']['world_sha256'],steps=None if f['mission']is None else f['mission']['steps'],blocked=None if f['mission']is None else f['mission']['blocked'])for n,f in frames.items()})
   print(name,'native observations captured',flush=True)
  assert sha(border)==border_input_sha,'construction border changed during native replay'
  assert receipt['runs'][0]==receipt['runs'][1],'native input sequence did not reproduce'
  receipt['status']='blocked' if args.expect=='blocked' else 'passed'
  receipt['startup_resource_provenance']='source884D0-generated-border-development'
  if args.border_policy=='mapped-prior':
   receipt['startup_resource_provenance']='mapped-zero-prior-development'
   receipt['limitations'].append('Mapped zero construction-border prior differs from source884D0 generated422f124e table; affected construction must be rerun after integration')
  receipt['classification']='unsupported_behavior' if args.expect=='blocked' else 'completed_development_extractor'
 except Exception as e:
  receipt['classification']='harness_or_unexpected_behavior';receipt['error']=repr(e);raise
 finally:
  destination=ROOT/f"research/integration/native-extractor-{args.border_policy}-{receipt['status']}.json"
  destination.write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':receipt['status'],'work':str(work)}),flush=True)
if __name__=='__main__':main()
