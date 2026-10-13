"""Independent numeric-port review; runs only our Rust harness, never originals."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment

def record(path):
    data=path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(),size=len(data),sha256=hashlib.sha256(data).hexdigest())

def main():
    receipt_path=ROOT/'research/ghidra/construction-evidence.json'
    proof=json.loads(receipt_path.read_bytes())
    listings=[]
    for entry in proof['private_listings']:
        actual=record(ROOT/entry['private_path']);assert actual['sha256']==entry['sha256'];listings.append(actual)
    obj3=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes()
    costs=struct.unpack_from('<14i',obj3,0x4e1c)
    assert list(costs)==proof['costs']
    assert hashlib.sha256(obj3[0x4e1c:0x4e54]).hexdigest()==proof['cost_table_sha256']
    scratch=ROOT/'private/review/construction';scratch.mkdir(parents=True,exist_ok=True)
    source=Path(__file__).with_suffix('.rs');exe=scratch/'construction-review.exe'
    built=subprocess.run([str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow','-O',str(source),'-o',str(exe)],capture_output=True,env=environment(),timeout=60)
    assert built.returncode==0,built.stderr.decode(errors='replace')
    run=subprocess.run([str(exe)],capture_output=True,timeout=30);assert run.returncode==0,run.stderr.decode(errors='replace')
    counts=json.loads(run.stdout)
    data=''.join(','.join(str(int(v[key])) for key in ('selector','skill','primary','secondary','refill','ready'))+'\n' for v in proof['vectors'])
    run=subprocess.run([str(exe),'--vectors'],input=data.encode(),capture_output=True,timeout=10)
    assert run.returncode==0
    lines=run.stdout.decode().splitlines();assert len(lines)==len(proof['vectors'])==18
    for vector,line in zip(proof['vectors'],lines):
        status,cost,p,s,stage=line.split(',')
        actual=dict(status=status,primary=int(p),secondary=int(s),stage=int(stage))
        if cost!='null':actual['cost']=int(cost)
        assert actual==vector['expected'],(vector,actual)
    counts['receipt_vectors']=len(lines)
    result=dict(status='passed',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
        checks=counts,harness=record(source),native_harness=record(exe),reviewed_source=record(ROOT/'rust/crates/genewars-core/src/construction.rs'),
        evidence_receipt=record(receipt_path),private_listing_hash_checks=listings,cost_table_sha256=proof['cost_table_sha256'],
        scope='Pure debit final values after mode0/stage3 and callback prerequisite; signed selector safely restricted0..13; world/helper effects excluded.',
        finding='No mismatch in signed branches, low-byte readiness contract, wrapping final values, negative costs, refill assignment/threshold or stage result.')
    Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(counts))

if __name__=='__main__':main()
