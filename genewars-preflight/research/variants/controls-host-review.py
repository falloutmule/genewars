"""Record source review and run only unit tests; never launch a GUI/original."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[2]


def record(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(),size=len(data),sha256=hashlib.sha256(data).hexdigest())


def main():
    scratch = ROOT/'private/review/controls-host'
    scratch.mkdir(parents=True,exist_ok=True)
    command = [sys.executable,'-B',str(ROOT/'scripts/cargo.py'),'test','--offline','--locked',
               '-p','genewars-desktop','--bin','genewars-controls']
    run = subprocess.run(command,capture_output=True,timeout=120)
    (scratch/'unit-stdout.txt').write_bytes(run.stdout)
    (scratch/'unit-stderr.txt').write_bytes(run.stderr)
    assert run.returncode == 0,run.stderr.decode(errors='replace')
    text = run.stdout.decode(errors='replace')
    counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed',text)
    assert counts and sum(map(int,counts)) >= 11
    receipt = dict(status='static-and-unit-review-passed',checked_utc=datetime.now(timezone.utc).isoformat(),
        gui_or_original_execution=False,source=[record(ROOT/path) for path in
            ('rust/crates/genewars-desktop/src/bin/genewars-controls.rs','rust/crates/genewars-render/src/lib.rs',
             'rust/crates/genewars-core/src/mouse_packet.rs','rust/crates/genewars-core/src/custom_list.rs')],
        command=command,exit_code=run.returncode,unit_tests_passed=sum(map(int,counts)),
        private_outputs=[record(scratch/name) for name in ('unit-stdout.txt','unit-stderr.txt')],
        reviewed_contracts=['Ordered press notification sets an empty latch even while already physically held',
            'First press position retained separately; hit test uses current position at packet production',
            'Quick press/release produces press packet then release packet',
            'Captured release activates outside control; original list order resolves overlapping hits',
            'Physical-to-virtual floor mapping agrees with nearest-neighbor display sampling',
            'Selected component y offset and all32 sprite rows used; opaque palette zero remains hit-capable',
            'Public hit coordinate subtraction widened to i64 after independent overflow finding',
            'Input/packet/activation traces capped with explicit omitted counts',
            '60Hz is a host diagnostic schedule, not original timing'],
        limits=['Focus changes are deliberately ignored; focus-loss recovery not claimed',
            'No native pointer/quick-click observation by this review receipt',
            'No gameplay, right-button, audio or original scheduler parity'],
        earlier_build_failure='Bundled GNU DLLTool could not start assembler; root supplied private LLVM tooling, after which these unit tests passed.')
    Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status=receipt['status'],unit_tests_passed=receipt['unit_tests_passed'],gui_or_original_execution=False)))


if __name__=='__main__': main()
