"""Read-only audit of existing native evidence and shared snapshot wiring. No GUI."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json
ROOT=Path(__file__).resolve().parents[2]
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def main():
 receipt=ROOT/'research/integration/plant-snapshot-window.json';summary=json.loads(receipt.read_bytes());private=ROOT/summary['private_receipt']['path'];assert pin(private)['sha256']==summary['private_receipt']['sha256'];saved=json.loads(private.read_bytes())
 assert saved['status']=='passed'and saved['native_windows_launched']is True and len(saved['cases'])==6
 comparisons=[]
 for case in saved['cases']:
  for key,digest in [('snapshot_path','snapshot_sha256'),('fixture_bmp_path','fixture_bmp_sha256'),('native_report','native_report_sha256')]:assert pin(ROOT/case[key])['sha256']==case[digest]
  native=json.loads((ROOT/case['native_report']).read_bytes());scene=native['plant_scene'];assert native['presented_frames']==2 and native['exit_reason']=='frame_limit';assert native['rgb_crc16']==case['expected_rgb_crc16'];assert scene['queue_order']==case['expected_queue_order'];assert scene['initial_state_inferred']is False and scene['admission_callbacks_applied']is False and scene['dirty_prepass_applied']is False
  comparisons.append(dict(case=case['case'],native_report_sha256=case['native_report_sha256'],crc16=case['expected_rgb_crc16'],queue_order=scene['queue_order']))
 assert all(item['exit_code']!=0 and item['report_absent']for item in saved['rejected'])and len(saved['rejected'])==3
 current=[]
 for prior in summary['code_sources']:
  p=pin(ROOT/prior['path']);current.append(dict(**p,matches_saved=p['sha256']==prior['sha256']))
 for p in current:
  if p['path'].startswith('rust/'):assert p['matches_saved'],p
 desktop=(ROOT/'rust/crates/genewars-desktop/src/bin/genewars-terrain.rs').read_text();tools=(ROOT/'rust/crates/genewars-tools/src/plant_scene.rs').read_text()
 for snippet in ['PlantSceneSnapshot::parse(&snapshot_bytes)','queue_family1(snapshot.active, snapshot.initial_count, snapshot.margin, &candidates)','family1_draw_packet(snapshot.gate, snapshot.mode, queued.flags)','draw_explicit_family1_scene(&records, &mut rendered.indices[128..], [512, 480], 640','origin: [i32::from(plant.queued.x), i32::from(plant.queued.draw_y)]','native_flags_at_2e: plant.record.jitter','shade_at_17: plant.queued.shade']:assert snippet in desktop,snippet
 assert 'PlantSceneSnapshot::parse(bytes).map(Self)'in tools
 review=json.loads((ROOT/'research/variants/plant-scene-review.json').read_bytes());assert review['status']=='passed-independent-native-plant-scene-review'
 report=dict(status='passed-existing-native-snapshot-wiring-review',utc=datetime.now(timezone.utc).isoformat(),checker_gui_execution=False,checker_original_execution=False,public_evidence=pin(receipt),private_evidence=pin(private),source_pins=current,private_receipt_records_code_pins='code_sources'in saved,recorded_native_cases=len(comparisons),recorded_rejections=3,comparisons=comparisons,independent_scene_pixel_review=pin(ROOT/'research/variants/plant-scene-review.json'),limits=['Native evidence was inspected, not newly executed by this reviewer','Public source pins match current production; old private receipt lacks code pins, so public summary alone does not establish historical source capture','Native outputs are CRC16 and receipts; no native rawframebuffer or cryptographic pixel equality claimed','Independent fullpixel comparison covers shared headless scene and tools adapter; current native source uses same APIs','Scene snapshot queue is fixed at load and drawn on host camera/map diagnostic refresh; no live projection/callback or original cadence claim'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n');tmp.replace(out);print(json.dumps(dict(status=report['status'],recorded_native_cases=6,recorded_rejections=3)))
if __name__=='__main__':main()
