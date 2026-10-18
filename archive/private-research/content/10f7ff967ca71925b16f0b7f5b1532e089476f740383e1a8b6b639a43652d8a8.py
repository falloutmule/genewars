"""Bound a human-visible input check of our Rust runtime; never run original code."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import shutil
import subprocess
import time
import uuid

ROOT = Path(__file__).resolve().parents[2]


def record(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(), size=len(data), sha256=hashlib.sha256(data).hexdigest())


def main():
    directory = ROOT / 'private/integration/runtime-ui' / uuid.uuid4().hex
    directory.mkdir(parents=True)
    original = ROOT / 'private/build/rust-target/debug/genewars-game.exe'
    tested = directory / 'genewars-game.exe'
    shutil.copyfile(original, tested)
    inputs = [record(p) for p in sorted((ROOT / 'private/integration/game-runtime').rglob('*')) if p.is_file()]
    paths = [*sorted((ROOT / 'rust/crates/genewars-runtime/src').glob('*.rs')),
             ROOT / 'rust/crates/genewars-desktop/src/bin/genewars-game.rs',
             *[ROOT / 'rust/crates/genewars-desktop/src/bin/support' / name for name in
               ['runtime_resources.rs','runtime_scene.rs','game_window.rs','menu_art.rs','status_text.rs']],
             ROOT / 'research/integration/runtime-resource-catalog.tsv']
    command = [str(tested), str(ROOT / 'private/integration/game-runtime'), '--seconds', '300', '--report', str(directory / 'report.json')]
    started = datetime.now(timezone.utc).isoformat()
    begin = time.monotonic()
    with (directory / 'stdout.txt').open('w', encoding='utf-8') as out, (directory / 'stderr.txt').open('w', encoding='utf-8') as err:
        process = subprocess.Popen(command, stdout=out, stderr=err, cwd=ROOT)
        launch = dict(pid=process.pid, command=command, started_utc=started,
                      binary=record(tested), inputs=inputs, sources=[record(p) for p in paths],
                      original_execution=False, timeout_seconds=330)
        (directory / 'launch.json').write_text(json.dumps(launch, indent=2)+'\n', encoding='utf-8')
        print(json.dumps(dict(directory=str(directory),pid=process.pid)), flush=True)
        timeout = False
        try:
            code = process.wait(timeout=330)
        except subprocess.TimeoutExpired:
            timeout = True
            process.kill()
            code = process.wait(timeout=10)
    result = dict(launch, ended_utc=datetime.now(timezone.utc).isoformat(), exit_code=code,
                  elapsed_seconds=round(time.monotonic()-begin,3), host_timeout=timeout,
                  inputs_unchanged=all(record(ROOT / r['path']) == r for r in inputs),
                  sources_unchanged=all(record(ROOT / r['path']) == r for r in launch['sources']))
    (directory / 'process.json').write_text(json.dumps(result, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({k:result[k] for k in ['exit_code','elapsed_seconds','host_timeout','inputs_unchanged','sources_unchanged']}),flush=True)


if __name__ == '__main__':
    main()
