"""Complete padded framebuffer oracle for exact first-mission building common images."""
import hashlib,json,struct,subprocess,sys,uuid
from pathlib import Path
import validate_bturr_common_pixels as reference
from validate_bturr_status_overlay_pixels import sprite_from_table
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment

def decode_frame(data,index,animation=0):
    _,*offsets=struct.unpack_from('<H4I',data)
    first,last,count=struct.unpack_from('<IIH',data,offsets[3]+animation*10)
    assert 0<=index<count and last==first+(count-1)*14
    f=offsets[2]+first+index*14;component=struct.unpack_from('<I',data,f)[0]
    parts=[]
    for c in range(data[f+11]):
        p=offsets[1]+component+c*10;ref,x,y,flags,extra=struct.unpack_from('<IhhBB',data,p)
        assert flags==0
        width,height,pixels,opaque,_=sprite_from_table(data,offsets,ref//6)
        parts.append({'sprite_width':width,'sprite_height':height,'local':[x>>1,y>>1],'indices':pixels,'opacity':opaque})
    return {'components':parts,'stats':[],'display_dimensions_raw':list(struct.unpack_from('<hh',data,f+4))}

def main():
    work=ROOT/'private/integration/source-building-pixels'/uuid.uuid4().hex;work.mkdir(parents=True)
    for rel,digest in reference.LISTINGS.items():reference.check(ROOT/rel,digest)
    fade=reference.check(reference.FADE,*reference.INPUTS[reference.FADE]);gla=reference.check(reference.GLA,*reference.INPUTS[reference.GLA])
    reference.decode_frame=decode_frame
    subprocess.run([sys.executable,'scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=ROOT,check=True,capture_output=True)
    deps=ROOT/'private/build/rust-target/debug/deps';rlib=max(deps.glob('libgenewars_runtime-*.rlib'),key=lambda p:p.stat().st_mtime)
    exe=work/'harness.exe'
    subprocess.run([str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(ROOT/'scripts/desktop/source_building_pixel_harness.rs'),'--extern',f'genewars_runtime={rlib}','-L',f'dependency={deps}','-o',str(exe)],env=environment(),check=True)
    total=0;results=[]
    for name,completed in [('BSOLA',10),('BMINE',20),('BPOWE',9)]:
        path=ROOT/f'private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/{name}.ANB';data=path.read_bytes();cases=[]
        for animation,count in [(0,3),(1,completed)]:
            for frame in range(count):
                for variant,(origin,shade,actor_flags,global_flags,phase) in enumerate([
                    ([320,240],17,0,0,0),([1,1],30,0,4,3),([637,475],0,0,0,2),([-32768,32767],11,2,4,3)]):
                    cases.append(dict(id=len(cases),animation=animation,frame=frame,owner=variant,shade=shade,actor_flags=actor_flags,global_flags=global_flags,origin=origin,phase=phase,building_flags=0x40|variant%2))
        casefile=work/f'{name}.txt';casefile.write_text('\n'.join(' '.join(map(str,[c['id'],c['animation'],c['frame'],c['owner'],c['shade'],c['actor_flags'],c['global_flags'],*c['origin'],c['phase'],c['building_flags']])) for c in cases))
        run=subprocess.run([str(exe),str(path),str(reference.FADE),str(reference.GLA),str(casefile)],capture_output=True,text=True,check=True)
        (work/f'{name}.stdout').write_text(run.stdout)
        reports={};rejections=[]
        for line in run.stdout.splitlines():
            if line.startswith('reject_'):rejections.append(line);continue
            words=line.split();reports[int(words[0])]=dict(w.split('=') for w in words[1:])
        assert len(reports)==len(cases) and len(rejections)==6
        for case in cases:
            expected=reference.oracle(data,fade,gla,case);actual=reports[case['id']]
            assert actual['sha']==expected['sha256'],(name,case)
            for field in ['written','clipped']:assert int(actual[field])==expected[field]
            assert int(actual['opaque'])==expected['opaque']
        total+=len(cases);results.append({'resource':name,'cases':len(cases),'atomic_rejections':len(rejections)})
    receipt={'status':'passed','scope':'source43A40 common main image only; no callbacks, projection, overlays or completed building behavior','cases':total,'resources':results,'framebuffer_bytes_per_case':(480-1)*648+640,'work':str(work.relative_to(ROOT)),
             'implementation_pins':{p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in ['rust/crates/genewars-runtime/src/building_resources.rs','rust/crates/genewars-runtime/src/source_building_resources.rs']}}
    (ROOT/'research/integration/source-building-pixels.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
if __name__=='__main__':main()
