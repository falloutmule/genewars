"""Build an isolated optimized numeric trajectory and retain reproducible captures."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,sys,uuid
from cargo import environment
from runtime_inputs import stage
P=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames',type=int,default=6143)
    parser.add_argument('--seed',type=int,default=5428)
    parser.add_argument('--capture-stride',type=int,default=1)
    parser.add_argument('--output',type=Path,help='optional distinct public receipt; failed runs retain private results only')
    args=parser.parse_args()
    assert 1<=args.frames<=20000 and 0<=args.seed<=65535 and 1<=args.capture_stride<=20000
    runtime,input_receipt=stage()
    work=P/'private/review/family8-trajectory'/uuid.uuid4().hex;work.mkdir(parents=True)
    source_files=[p for crate in ['assets','core','runtime'] for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')]
    source_files += [P/'scripts/family8_trajectory_probe.rs',Path(__file__)]
    pins={p.relative_to(P).as_posix():sha(p)for p in source_files}
    input_pins={r['path']:r['sha256']for r in input_receipt['resources']}
    assert all(sha(runtime/rel)==digest for rel,digest in input_pins.items())
    rustc=P/'private/tools/rust/cargo/bin/rustc.exe';env=environment()
    def command(args):
        r=subprocess.run(list(map(str,args)),cwd=P,env=env,capture_output=True,timeout=180)
        assert r.returncode==0,r.stderr.decode(errors='replace')
    for crate in ['assets','core','runtime']:
        cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes',
            '--crate-name',f'genewars_{crate}','--crate-type','rlib',P/f'rust/crates/genewars-{crate}/src/lib.rs',
            '-L',f'dependency={work}','-o',work/f'libgenewars_{crate}.rlib']
        if crate=='runtime':
            for dep in ['assets','core']:cmd+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
        command(cmd)
    binary=work/'probe.exe'
    cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes',P/'scripts/family8_trajectory_probe.rs','-L',f'dependency={work}','-o',binary]
    for dep in ['assets','core','runtime']:cmd+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
    command(cmd)
    assert pins=={p.relative_to(P).as_posix():sha(p)for p in source_files},'sources changed during compilation; retry'
    for rel,digest in pins.items():
        contents=(P/rel).read_bytes()
        assert hashlib.sha256(contents).hexdigest()==digest,'source changed before archival: '+rel
        saved=work/'compilation_sources'/rel;saved.parent.mkdir(parents=True,exist_ok=True);saved.write_bytes(contents)
    (work/'runtime-inputs.json').write_text(json.dumps(input_receipt,indent=2)+'\n',encoding='utf-8')
    metadata={'utc':datetime.now(timezone.utc).isoformat(),'source_pins':pins,'binary_sha256':sha(binary),
        'seed':args.seed,'requested_frames':args.frames,'capture_stride':args.capture_stride,'native_priors':'explicit zero host policy','original_execution':False,
        'resource_catalog_sha256':input_receipt['catalog_sha256'],'runtime_input_pins':input_pins}
    (work/'compilation.json').write_text(json.dumps(metadata,indent=2)+'\n',encoding='utf-8')
    r=subprocess.run([str(binary),str(P),'mule',str(args.seed),str(args.frames),str(work),str(args.capture_stride)],env=env,capture_output=True,timeout=300)
    (work/'stdout.txt').write_bytes(r.stdout);(work/'stderr.txt').write_bytes(r.stderr)
    assert r.returncode==0,r.stderr.decode(errors='replace')
    assert all(sha(runtime/rel)==digest for rel,digest in input_pins.items()),'runtime inputs changed during probe'
    output=r.stdout.decode();stopped=re.search(r'STOP frame(\d+): (.+)',output)
    metadata.update(status='boundary' if stopped else 'passed',completed_frames=int(stopped[1])-1 if stopped else args.frames,
        boundary=stopped[2] if stopped else None,private_output=work.relative_to(P).as_posix())
    assert stopped or f'PASS{args.frames} mule=true seed={args.seed}'in output
    captures={p.relative_to(work).as_posix():sha(p)for p in work.glob('frame*/*/*.bin')}
    metadata['captures']=captures
    (work/'result.json').write_text(json.dumps(metadata,indent=2)+'\n',encoding='utf-8')
    if not stopped:
        target=(args.output or P/'research/integration/family8-trajectory-validation.json').resolve()
        assert target.is_relative_to((P/'research/integration').resolve()),'public receipts belong under research/integration'
        tmp=target.with_suffix('.json.tmp');tmp.write_text(json.dumps(metadata,indent=2)+'\n',encoding='utf-8');tmp.replace(target)
    print(json.dumps({k:metadata[k]for k in ['status','completed_frames','boundary','private_output']}),flush=True)
    return 1 if stopped else 0
if __name__=='__main__':raise SystemExit(main())
