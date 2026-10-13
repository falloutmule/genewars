"""Compile and run only an independent harness of our Rust core ports."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment


def record(path):
    raw = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(),size=len(raw),sha256=hashlib.sha256(raw).hexdigest())


def main():
    scratch = ROOT/'private/review/core-controls'
    scratch.mkdir(parents=True,exist_ok=True)
    source = Path(__file__).with_suffix('.rs')
    exe = scratch/'core-control-review.exe'
    command = [str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(source),'-o',str(exe)]
    compiled = subprocess.run(command,capture_output=True,env=environment(),timeout=60)
    assert compiled.returncode == 0,compiled.stderr.decode(errors='replace')
    run = subprocess.run([str(exe)],capture_output=True,timeout=10)
    assert run.returncode == 0,run.stderr.decode(errors='replace')
    checks = json.loads(run.stdout)
    listings = []
    receipts = []
    for name in ('input-evidence.json','list-layout-evidence.json'):
        path = ROOT/'research/ghidra'/name
        receipts.append(record(path))
        evidence = json.loads(path.read_bytes())
        for item in evidence['private_listings']:
            actual = record(ROOT/item['private_path'])
            assert actual['sha256'] == item['sha256']
            listings.append(actual)
    output = dict(status='passed',checked_utc=datetime.now(timezone.utc).isoformat(),
        original_or_gui_execution=False,checks=checks,harness=record(source),native_harness=record(exe),
        reviewed_sources=[record(ROOT/'rust/crates/genewars-core/src'/name) for name in ('mouse_packet.rs','custom_list.rs')],
        static_evidence=receipts,listing_hash_checks=listings,
        bounds_scope='Ten 225-byte records; 34-byte NUL window is a conservative safe-port rule, not a guarantee of original string termination.',
        packet_scope='Mode2 one-button transition only; cadence, callback races, mickey scaling and other packet fields excluded.')
    Path(__file__).with_suffix('.json').write_text(json.dumps(output,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(checks))


if __name__=='__main__': main()
