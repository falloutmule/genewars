"""Audit the recorded supervised Rust input check; does not replay UI input."""
from pathlib import Path
import hashlib
import json
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'private/integration/runtime-ui/7cf7ef61a8af4fe488a91e6e3069bc22'


def pin(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(), size=len(data), sha256=hashlib.sha256(data).hexdigest())


def main():
    launch = json.loads((DIRECTORY/'launch.json').read_bytes())
    process = json.loads((DIRECTORY/'process.json').read_bytes())
    observed = json.loads((DIRECTORY/'report.json').read_bytes())
    for key in launch:
        assert launch[key] == process[key], ('launch/process mismatch', key)
    assert launch['command'] == [str(DIRECTORY/'genewars-game.exe'),str(ROOT/'private/integration/game-runtime'),
                                  '--seconds','300','--report',str(DIRECTORY/'report.json')]
    assert pin(ROOT/launch['binary']['path']) == launch['binary']
    assert process['exit_code'] == 0 and not process['host_timeout']
    assert process['inputs_unchanged'] and process['sources_unchanged']
    assert 0 < process['elapsed_seconds'] < launch['timeout_seconds'] == 330
    assert datetime.fromisoformat(process['ended_utc']) > datetime.fromisoformat(launch['started_utc'])
    assert not process['original_execution']
    for source in launch['inputs']:
        assert pin(ROOT/source['path']) == source
    assert observed['scene_history'] == ['Menu','Loading','Mission','Menu','Options','Menu','Quit']
    assert observed['scene'] == 'Quit' and observed['error'] is None and observed['mission'] is None
    assert len(observed['ticks']) == 52 and observed['presented_frames'] >= 52
    assert not observed['gameplay'] and not observed['full_world_tick']
    trajectory = json.loads((ROOT/'research/variants/runtime-world-review.json').read_bytes())['trajectory']
    for actual, expected in zip(observed['ticks'],trajectory):
        assert actual['step'] == expected['tick'] and actual['world_sha256'] == expected['world_sha256']
    assert observed['ticks'][0]['rgb_sha256'] == 'adbc34fa1b7d0313f75abcf5d3afa6f26de3e63293920003697b5adde8c7bb83'
    assert observed['ticks'][1]['rgb_sha256'] == '71cd1f332aed8dd5c4d1969c096f6be5fb8edefdc3b1a2630da4493eec1725f8'
    names = ['menu.jpg','world-step-0.jpg','world-step-2.jpg','view-right.jpg','world-paused-52.jpg','options-on.jpg']
    for name in names:
        assert (DIRECTORY/name).read_bytes().startswith(b'\xff\xd8\xff')
    result = dict(status='passed-recorded-runtime-ui',checked_utc=datetime.now(timezone.utc).isoformat(),
                  original_execution=False,input_replayed_by_checker=False,
                  launch=pin(DIRECTORY/'launch.json'),process=pin(DIRECTORY/'process.json'),report=pin(DIRECTORY/'report.json'),
                  tested_binary=launch['binary'],tested_source_pins=launch['sources'],
                  screenshots=[pin(DIRECTORY/name) for name in names],
                  observations=['Single Player loaded the world in the same window',
                                'Two Space presses gave 40 to 38 to 37 plants; second step visibly removed a plant',
                                'Right/Left reprojected terrain and plants then restored the view',
                                'Enter started automatic steps, second Enter paused at step52',
                                'Escape returned to menu; Options opened; Space toggled OFF to ON; Escape returned',
                                'Quit exited normally; assets and tested sources remained unchanged'],
                  limits=['Historical supervised binary/source pins; subsequent builds are covered separately by runtime-validation',
                          'Window captures include host cursor/scaling; they are not raw original-game framebuffers',
                          'No player gameplay command, full world tick, save/load or original-game parity established'])
    (ROOT/'research/integration/runtime-ui.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status=result['status'],steps=52,screenshots=len(names),exit_code=0)))


if __name__ == '__main__':
    main()
