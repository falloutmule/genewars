"""Expected unsupported completed-Genepod callback, using a frozen runtime rlib."""
from pathlib import Path
import sys,subprocess,json,hashlib,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
def main():
 work=P/'private/review/genepod-completed-boundary'/uuid.uuid4().hex;work.mkdir(parents=True)
 target=P/'private/build/rust-target/debug';harness=Path(__file__).with_name('genepod_completed_boundary.rs')
 command=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={target}/deps','-o',str(work/'probe.exe')]
 for name in ['assets','runtime']:command+=['--extern',f'genewars_{name}={target}/libgenewars_{name}.rlib']
 built=subprocess.run(command,env=environment(),capture_output=True,text=True,timeout=60)
 (work/'build.log').write_text(built.stdout+built.stderr);assert built.returncode==0,built.stderr
 run=subprocess.run([str(work/'probe.exe'),str(P)],capture_output=True,text=True,timeout=30)
 (work/'run.log').write_text(run.stdout+run.stderr);assert run.returncode==0,run.stderr
 result=dict(status='passed-expected-boundary',private_run=work.relative_to(P).as_posix(),scope=run.stdout.strip(),
  harness_sha256=hashlib.sha256(harness.read_bytes()).hexdigest(),runtime_library_sha256=hashlib.sha256((target/'libgenewars_runtime.rlib').read_bytes()).hexdigest(),original_gameplay_acceptance=False)
 (P/'research/integration/genepod-completed-boundary.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
if __name__=='__main__':main()
