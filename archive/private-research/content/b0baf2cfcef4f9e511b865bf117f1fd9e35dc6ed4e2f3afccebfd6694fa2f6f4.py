from pathlib import Path
import sys,subprocess,os
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
t=P/'private/build/rust-target/debug'
subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,check=True)
args=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-O','private/integration/family8-combined-probe.rs','-L',f'dependency={t}/deps','-o',str(P/'private/integration/family8-combined-probe.exe')]
for n in ['assets','core','runtime']:
    lib=t/f'libgenewars_{n}.rlib'
    if not lib.exists():lib=max((t/'deps').glob(f'libgenewars_{n}-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
    args.extend(['--extern',f'genewars_{n}={lib}'])
subprocess.run(args,cwd=P,env=environment(),check=True)
if len(sys.argv)>1:
    for seed in sys.argv[1:]:subprocess.run([str(P/'private/integration/family8-combined-probe.exe'),str(P),'mule',seed,os.environ.get('GENEWARS_PROBE_FRAMES','450')],check=True)
else:
    for mode in ['actors','mule']:subprocess.run([str(P/'private/integration/family8-combined-probe.exe'),str(P),mode],check=True)
