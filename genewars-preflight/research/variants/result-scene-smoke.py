"""Frozen native victory renderer smoke; no original execution or gameplay claim."""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(P/'scripts'))
from cargo import environment
from result_inputs import stage

def main():
    stage()
    work=P/'private/review/result-scene'/uuid.uuid4().hex
    work.mkdir(parents=True)
    source=P/'rust/crates/genewars-desktop/src/bin/support/result_scene.rs'
    shutil.copy2(source,work/'result_scene.rs')
    shutil.copy2(Path(__file__).with_name('result-scene-probe.rs'),work/'probe.rs')
    deps=P/'private/build/rust-target/debug/deps'
    cmd=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(work/'probe.rs'),'-L',str(deps),'-o',str(work/'probe.exe')]
    pins={}
    for name in ['genewars_assets','genewars_render']:
        f=max(deps.glob('lib'+name+'-*.rlib'),key=lambda x:x.stat().st_mtime)
        pins[str(f.relative_to(P))]=hashlib.sha256(f.read_bytes()).hexdigest()
        cmd+=['--extern',name+'='+str(f)]
    subprocess.run(cmd,env=environment(),check=True,capture_output=True,timeout=60)
    run=subprocess.run([str(work/'probe.exe'),str(P),str(work/'frame.rgb')],check=True,capture_output=True,text=True,timeout=15)
    rgb=(work/'frame.rgb').read_bytes()
    assert len(rgb)==640*480*3
    result={'status':'passed_render_smoke_not_source_pixel_equality','run':str(work.relative_to(P)),
        'renderer_sha256':hashlib.sha256((work/'result_scene.rs').read_bytes()).hexdigest(),
        'rlib_pins':pins,'rgb_sha256':hashlib.sha256(rgb).hexdigest(),'receipt':run.stdout.strip(),
        'animation':'pending','native_input':'not_run'}
    (work/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result))

if __name__=='__main__':main()
