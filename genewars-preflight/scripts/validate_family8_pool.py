"""Check native family8 storage invariants, separately from factory replay."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess, uuid
from cargo import environment
P = Path(__file__).resolve().parents[1]
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    work = P / 'private/review/family8-pool' / uuid.uuid4().hex
    work.mkdir(parents=True)
    exe = work / 'probe.exe'
    source = P / 'scripts/family8_pool_probe.rs'
    modules = [P / 'rust/crates/genewars-runtime/src' / name for name in ['family8_pool.rs', 'family8_creation.rs']]
    paths = [Path(__file__), source] + modules
    pins = {p.relative_to(P).as_posix(): sha(p) for p in paths}
    subprocess.run([str(P / 'private/tools/rust/cargo/bin/rustc.exe'), '--edition=2021', str(source), '-o', str(exe)],
                   cwd=P, env=environment(), check=True, capture_output=True)
    assert pins == {p.relative_to(P).as_posix(): sha(p) for p in paths}
    result = subprocess.run([str(exe)], cwd=P, check=True, capture_output=True, text=True)
    listing = P / 'private/ghidra/mission-tail-readers/instructions-00010090.txt'
    receipt = dict(status='passed-storage-invariants', utc=datetime.now(timezone.utc).isoformat(),
        slots=16, loader_compared_bytes=1632, source_loader_pin={listing.relative_to(P).as_posix(): sha(listing)},
        implementation_pins=pins, private_run=work.relative_to(P).as_posix(), result=result.stdout.strip(),
        scope='Synthetic complete16-slot inactive loader/gap comparison and checked typed creation, stale before-image, pointer and identity mutation rejection. Loader mapping is source10090/6D490 structural evidence; this harness does not interpret those instructions. The separate family8-creation validator replays the complete numeric source factory. No owned-world adapter or scheduler integration.', original_execution=False)
    (P / 'research/integration/family8-pool-validation.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(result.stdout.strip())
if __name__ == '__main__': main()
