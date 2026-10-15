"""Native construction/result/campaign replay; accepted mission count stays zero."""
import argparse,json,shutil,subprocess,uuid
from pathlib import Path
from datetime import datetime,timezone
from validate_native_extractor import ROOT,INPUT,sha
PATTERNS_SHA='d848e423b7a76a2c2dbf517fc595e33a789df7382697b56f382cc7606b18939e'
TABLE_SHA='422f124eac806197ba95d2438516284c3bf828a1db8e8aabe330661fff23ef8a'
class BoundaryBlocked(Exception):pass

def read_stopped_boundary(folder):
    outcome_path=folder/'__outcome.json'
    if not outcome_path.exists():return None
    outcome=json.loads(outcome_path.read_text())
    if outcome.get('status')!='blocked':raise ValueError('unsupported native outcome status')
    frame=json.loads((folder/'__blocked.json').read_text());mission=frame.get('mission')
    if not mission or not mission.get('blocked'):raise ValueError('blocked outcome lacks blocked mission report')
    expected=f"native input replay blocked at step {mission['steps']}: {mission['blocked']}"
    if outcome.get('reason')!=expected:raise ValueError('blocked outcome/report reason mismatch')
    return dict(name='__blocked',steps=mission['steps'],reason=mission['blocked'],world_sha256=mission['world_sha256'])

def assert_expected_boundary(observation,reason,step):
    if observation['reason']!=reason or observation['steps']!=step:
        raise AssertionError(f'boundary mismatch: {observation}')

def chooser_point(choices,selector):
    row=choices.index(selector);top=480-((len(choices)+4)*18+12)
    return (200,top+6+3*18+row*18+9)
def append_build(text,label,selector,choices,point,work_ticks):
    x,y=chooser_point(choices,selector)
    text+=f'key c\nobserve {label}-unlock\npointer {x} {y}\nleft down\nleft up\nobserve {label}-armed\npointer {point[0]} {point[1]}\nright down\nright up\nobserve {label}-assigned\n'
    count=0
    for ticks in work_ticks:
        count+=ticks;text+=f'key space {ticks}\nobserve {label}-work{count}\n'
    return text+f'key c\nobserve {label}-next-choices\nkey escape\n'
def parse_args(argv=None):
    parser=argparse.ArgumentParser();parser.add_argument('--probe',type=Path,required=True);parser.add_argument('--expect',choices=['completed','blocked'],required=True);parser.add_argument('--reference-startup',choices=['fresh']);parser.add_argument('--blocked-reason');parser.add_argument('--blocked-step',type=int);parser.add_argument('--photons',action='store_true');parser.add_argument('--result-view',action='store_true');parser.add_argument('--campaign-view',action='store_true');parser.add_argument('--krell-entry',action='store_true');args=parser.parse_args(argv)
    if args.expect=='blocked' and (args.blocked_reason is None or args.blocked_step is None):parser.error('--expect blocked requires --blocked-reason and --blocked-step')
    if args.krell_entry:args.campaign_view=args.result_view=args.photons=True
    if args.photons and not args.reference_startup:parser.error('--photons requires --reference-startup fresh')
    if args.result_view and not args.photons:parser.error('--result-view requires --photons')
    if args.campaign_view and not args.result_view:parser.error('--campaign-view requires --result-view')
    return args

def build_input(args):
    initial_ticks=200 if args.reference_startup else 189
    text=INPUT.split('pointer 320 234')[0].replace('key space 189',f'key space {initial_ticks}')
    text=append_build(text,'extractor',11,[11],[320,234],(100,200,300,500))
    text=append_build(text,'distributor',3,[11,3],[400,370],(100,200,300,600))
    if args.photons:
        text=append_build(text,'rejected-photon',6,[11,3,6],[430,350],(30,))
        for index,point in enumerate([[468,402],[348,394],[508,346]],1):
            text=append_build(text,f'photon{index}',6,[11,3,6],point,(100,200,400))
    if args.result_view:
        text+='key enter\nobserve victory-entry\n'
    if args.campaign_view:
        text+='key enter\nobserve campaign-entry\n'
    if args.krell_entry:text+='key enter\nobserve krell-entry\n'
    return text

def assert_krell_entry(frame):
    assert frame['scene']=='Mission' and frame['error']is None
    mission=frame['mission'];assert mission['steps']==0 and mission['blocked']is None
    assert mission['campaign_progress']==1
    assert mission['world_mode']=='carried-cd-explicit-opaque-priors'
    assert 'origin: CarriedCdKrell' in mission['runtime_context']
    assert mission['player_funds'][0]['balance']==2000
    actors=mission['actor_states'];assert len(actors)==3 and {a['slot']for a in actors}=={0,1,2}
    assert all(a['owner']==0 and not a['ready']for a in actors),'expected three prepared, not yet landed actors'
    creatures=mission['creature_states'];assert len(creatures)==4
    assert all(c['owner']==4 and c['species']==4 for c in creatures),'expected four neutral Mules'
    markers=mission['family8_states'];assert len(markers)==1
    marker=markers[0]
    assert (marker['slot'],marker['selector'],marker['state'],marker['owner'],marker['flags'],marker['animation'],marker['frame'])==(0,2,10,4,0x4020,0,0)

def main():
    args=parse_args()
    initial_ticks=200 if args.reference_startup else 189
    startup_args=['--reference-startup','fresh']if args.reference_startup else['--landing-seed','5428']
    probe=args.probe if args.probe.is_absolute()else ROOT/args.probe
    work=ROOT/'private/integration/native-building-chain'/uuid.uuid4().hex;work.mkdir(parents=True)
    binary=work/'genewars-game.exe';shutil.copy2(probe/'genewars-game.exe',binary)
    patterns=ROOT/'private/integration/game-runtime/construction/terrain-patterns.bin';assert sha(patterns)==PATTERNS_SHA
    menu=ROOT/'rust/crates/genewars-desktop/src/bin/support/building_menu.rs';menu_text=menu.read_text()
    for snippet in ['(choices.len()+4)*18+12','top+6+3*18','(row/18)']:assert snippet in menu_text,'review changed chooser geometry'
    assert chooser_point([11,3],3)==(200,447)
    text=build_input(args)
    (work/'input.txt').write_text(text);names=[line.split()[1]for line in text.splitlines()if line.startswith('observe ')]
    receipt=dict(status='error',utc=datetime.now(timezone.utc).isoformat(),scope='development native Extractor/Distributor construction',authentic_missions_completable=0,border_policy='generated',construction_border_sha256=TABLE_SHA,patterns_sha256=PATTERNS_SHA,binary_sha256=sha(binary),input_sha256=sha(work/'input.txt'),chooser_source_sha256=sha(menu),probe=str(probe.relative_to(ROOT)),work=str(work.relative_to(ROOT)),runs=[],limitations=['Development initialization and seed5428, not original campaign startup','Port-owned input mapping and chooser layout','No mission outcome or campaign transition asserted'])
    receipt['startup_arguments']=startup_args;receipt['original_execution']=False;receipt['photon_chain']=args.photons;receipt['krell_entry']=args.krell_entry
    if args.reference_startup:
        receipt['scope']='native Extractor/Distributor construction from source-backed fresh startup'
        receipt['limitations'][0]='Fresh reference startup uses serialized RNG; full mission acceptance remains separate'
    if args.photons:
        receipt['scope']='native fresh Exceland construction chain and source-backed victory predicate'
        receipt['limitations'][2]='Campaign continuation not accepted; no complete mission flow claimed'
    if args.result_view:
        receipt['scope']+=' with visible result-view entry'
        receipt['limitations'].append('Result movie decoder and audio mixer unavailable; entering result screen alone is not campaign continuation')
    if args.campaign_view:
        receipt['scope']+=' and source-backed campaign carry/view'
        receipt['limitations'][2]='Campaign carry/view is checked; complete mission acceptance remains blocked by presentation fidelity gaps'
    if args.krell_entry:
        receipt['scope']+=' followed by carried Krell startup at step zero'
        receipt['limitations'].append('Krell startup only: no simulated tick, marker arrival, Genepod production, zone credit or Krell completion asserted')
        receipt['limitations'][0]='Reference startup and carried Krell context retain explicit opaque-prior qualifications; accepted original missions remain zero'
    try:
        for run_name in ['first','repeat']:
            folder=work/run_name
            result=subprocess.run([str(binary),str(ROOT/'private/integration/game-runtime'),*startup_args,'--native-input-replay',str(work/'input.txt'),'--observations',str(folder),'--seconds','300'],cwd=ROOT,capture_output=True,text=True,timeout=600)
            (work/f'{run_name}.stdout').write_text(result.stdout);(work/f'{run_name}.stderr').write_text(result.stderr)
            stopped=read_stopped_boundary(folder)
            if stopped is not None:
                receipt['blocked_observation']=stopped;receipt['process_exit_code']=result.returncode
                raise BoundaryBlocked(stopped['reason'])
            result.check_returncode()
            frames={name:json.loads((folder/f'{name}.json').read_text())for name in names}
            blocked=next(((n,f['mission'])for n,f in frames.items()if f['mission']is not None and f['mission']['blocked']is not None),None)
            if blocked:
                receipt['blocked_observation']=dict(name=blocked[0],steps=blocked[1]['steps'],reason=blocked[1]['blocked'],world_sha256=blocked[1]['world_sha256'])
                raise BoundaryBlocked(blocked[1]['blocked'])
            assert all(frames[a]['presented_frames']<frames[b]['presented_frames']for a,b in zip(names,names[1:]))
            extractor=frames['extractor-work1100']['mission'];assert extractor['steps']==initial_ticks+1100 and extractor['blocked']is None
            assert any(b['selector']==11 and b['flags_word_31']&1 and b['progress_word_2b']==280 for b in extractor['buildings'])
            assert frames['distributor-unlock']['construction_choices']==[11,3]
            armed=frames['distributor-armed']['mission'];assert not armed['actor_states'][0]['assignment_pending']
            assigned=frames['distributor-assigned']['mission'];assert assigned['steps']==initial_ticks+1100
            actor=assigned['actor_states'][0];assert actor['assignment_pending']and actor['building_selector']==3
            assert len(assigned['buildings'])==1,'chooser must not directly create building'
            final=frames['distributor-work1200']['mission']
            if args.expect=='blocked':assert final['blocked']is not None
            else:
                assert final['blocked']is None and final['steps']==initial_ticks+2300
                buildings={b['selector']:b for b in final['buildings']};assert set(buildings)=={11,3}
                assert buildings[3]['progress_word_2b']==240 and buildings[3]['flags_word_31']&1
                assert final['terrain_jobs']['count']==0 and final['actor_states'][0]['state']==1
                assert final['player_funds'][0]['balance']<assigned['player_funds'][0]['balance']
                assert frames['distributor-next-choices']['construction_choices']==[11,3,6]
                assert any(any(b['selector']==3 and not b['flags_word_31']&1 for b in frames[f'distributor-work{n}']['mission']['buildings'])for n in [100,300,600]),'no visible incomplete Distributor observation'
            if args.photons:
                rejected=frames['rejected-photon-work30']['mission']
                assert rejected['steps']==2530 and rejected['blocked']is None
                assert rejected['player_funds'][0]['balance']==1074
                assert not rejected['actor_states'][0]['assignment_pending']
                for index in range(1,4):
                    result=frames[f'photon{index}-work700']['mission']
                    completed=[b for b in result['buildings'] if b['selector']==6 and b['flags_word_31']&1 and b['progress_word_2b']==80]
                    assert result['blocked']is None and len(completed)==index
                result=frames['photon3-work700']['mission']
                assert result['steps']==4630 and result['outcome']=={'state':'won','player':0}
                assert result['player_funds'][0]['balance']==510
            if args.result_view:
                result=frames['victory-entry'];assert result['scene']=='Victory'
                assert result['mission']['campaign_progress']==1
                assert result['result_view']['stage']==5 and result['result_view']['feedback_text_index']==478
                assert result['result_view']['movie']=='unavailable_decoder'
                assert not result['result_view']['campaign_transition_accepted']
            if args.campaign_view:
                result=frames['campaign-entry'];assert result['scene']=='Campaign' and result['mission']is None
                carry=result['campaign'];assert carry['progress']==1 and carry['next_mission']=='Krell'
                assert carry['carry_projected'] and carry['retired_actors']==[0,1]
                assert carry['mode_after_exit']==0 and carry['frame_after_view_resume']==4630
                assert not carry['next_mission_initialized']
            if args.krell_entry:assert_krell_entry(frames['krell-entry'])
            assert all(f['error']is None for f in frames.values())
            receipt['runs'].append({n:dict(bmp_sha256=sha(folder/f'{n}.bmp'),world_sha256=None if f['mission']is None else f['mission']['world_sha256'],steps=None if f['mission']is None else f['mission']['steps'],blocked=None if f['mission']is None else f['mission']['blocked'])for n,f in frames.items()})
            print(run_name,'native building-chain captured',flush=True)
        assert sha(patterns)==PATTERNS_SHA
        assert receipt['runs'][0]==receipt['runs'][1],'replay mismatch'
        receipt['status']='blocked'if args.expect=='blocked'else'passed';receipt['classification']='unsupported_behavior'if args.expect=='blocked'else('completed_carried_krell_entry'if args.krell_entry else'completed_campaign_view'if args.campaign_view else'completed_fresh_photon_commands'if args.photons else'completed_fresh_distributor'if args.reference_startup else'completed_development_distributor')
    except BoundaryBlocked as e:
        receipt['status']='blocked';receipt['classification']='unsupported_behavior';receipt['error']=str(e)
        receipt['expected_boundary_check']='not_requested'
        if args.expect=='blocked':
            receipt['expected_boundary_check']='failed'
            assert_expected_boundary(receipt['blocked_observation'],args.blocked_reason,args.blocked_step)
            receipt['expected_boundary_check']='passed'
            return
        raise
    except Exception as e:receipt['error']=repr(e);receipt['classification']='harness_or_unexpected_behavior';raise
    finally:
        encoded=json.dumps(receipt,indent=2)+'\n'
        (ROOT/f"research/integration/native-building-chain-{probe.name}-{receipt['status']}.json").write_text(encoded)
        (ROOT/f"research/integration/native-building-chain-{receipt['status']}.json").write_text(encoded);print(json.dumps({'status':receipt['status'],'work':str(work)}),flush=True)
if __name__=='__main__':main()
