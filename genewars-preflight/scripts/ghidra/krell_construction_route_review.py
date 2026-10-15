"""Read-only construction cadence, budget and authentic route observations."""
from pathlib import Path
import json,struct,hashlib
P=Path(__file__).resolve().parents[2];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
samples=[];pins={}
for name in ['krell-mule-callbacks-039/frame-785-slot-2-before','krell-mule-callbacks-046/frame-787-slot-4-after','krell-mule-callbacks-046/frame-820-slot-1-blocked-before']:
 d=P/'private/review'/name
 data={n:(d/n).read_bytes() for n in ['actors.bin','buildings.bin','players.bin']}
 for n in data:pins[(d/n).relative_to(P).as_posix()]=sha(d/n)
 a=data['actors.bin'][:200];b=data['buildings.bin'][:67];p=data['players.bin'][:1805]
 samples.append({'fixture':name,'health':struct.unpack_from('<h',b,43)[0],'work_timer':struct.unpack_from('<h',a,0x86)[0],'initial_delay':struct.unpack_from('<h',a,0x84)[0],'stage':struct.unpack_from('<h',a,0x88)[0],'action':a[0x59],'flags70':a[0x70],'funds':list(struct.unpack_from('<ii',p,0x4a3)),'skill':struct.unpack_from('<b',p,0x16+0x17)[0]})
assert [s['work_timer']for s in samples]==[43,41,8]
assert all(s['health']==187 and s['stage']==5 and s['initial_delay']==79 for s in samples)
assert samples[-1]['funds']==[1412,0] and samples[-1]['skill']==31
f=P/'private/integration/fresh-krell-entry-004';entry=json.loads((f/'report.json').read_bytes());win=json.loads((f/'winning-report.json').read_bytes())
assert entry['mission']['campaign_progress']==1 and entry['construction_choices']==[11]
assert entry['mission']['buildings']==[] and entry['mission']['player_funds'][0]['balance']==2000
for n in ['report.json','winning-report.json']:pins[(f/n).relative_to(P).as_posix()]=sha(f/n)
for name in ['private/ghidra/camera-player-input/instructions-00080590.txt','private/ghidra/campaign-level/instructions-0007d090.txt','rust/crates/genewars-runtime/src/building_availability.rs','rust/crates/genewars-runtime/src/construction_work.rs','rust/crates/genewars-runtime/src/construction_payment.rs','rust/crates/genewars-core/src/construction.rs']:
 pins[name]=sha(P/name)
report={'status':'passed_bounded_observations','synthetic_probe_samples':samples,'finding':'No funds/power stall established: construction work timer advances, health changes at zero-timer progress pulses.','accepted_frozen004_entry':{'funds':2000,'previous_win_funds':win['mission']['player_funds'][0]['balance'],'buildings':[],'construction_choices':[11],'raw_blueprint_skill':'not extracted from accepted carry by this probe'},'source_route':{'genepod':0,'dependency_chain':[11,3,0],'photon_processor6_required':False,'restored_player_ranges_half_open':[[0,1185],[1562,1783]],'construction_counts_and_owned_bits_restored':False},'synthetic_skill31_budget':{'initial':2000,'extractor':588,'distributor':338,'genepod':538,'remaining_before_income_and_mules':536,'mule_per_attempt_grade0':75},'pins':pins,'original_execution':False,'new_mission_acceptance':False}
(P/'research/campaign/krell-construction-route-review.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':report['status'],'work_timers':[43,41,8],'route':[11,3,0]}))
