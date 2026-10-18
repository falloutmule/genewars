"""Independent exact ANB metadata and sprite comparison for mission-one building resources."""
import hashlib,json,struct,subprocess,sys,uuid
from pathlib import Path
from validate_bturr_status_overlay_pixels import sprite_from_table
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
def main():
    work=ROOT/'private/integration/source-building-resources'/uuid.uuid4().hex;work.mkdir(parents=True)
    source=ROOT/'private/reference-run/run-006/drive-c/GAME/DATA/BUILDING'
    output=bytearray();resources=[]
    for name,selector in [('BSOLA',6),('BMINE',11),('BPOWE',3)]:
        data=(source/f'{name}.ANB').read_bytes();header,*offsets=struct.unpack_from('<H4I',data)
        counts=[]
        for a in range(2):
            first,last,count=struct.unpack_from('<IIH',data,offsets[3]+10*a);counts.append(count)
            for index,at in enumerate(range(first,last+1,14)):
                raw=data[offsets[2]+at:offsets[2]+at+14]
                component_start=struct.unpack_from('<I',raw)[0];width,height=struct.unpack_from('<hh',raw,4);hold,n=raw[10:12]
                output.extend(struct.pack('<BBBBBhh',selector,a,index,hold,n,width,height))
                for ci in range(n):
                    c=data[offsets[1]+component_start+ci*10:offsets[1]+component_start+(ci+1)*10]
                    output.extend(c);sprite=sprite_from_table(data,offsets,struct.unpack_from('<I',c)[0]//6)
                    output.extend(bytes(sprite[:2]));output.extend(sprite[2]);output.extend(sprite[3])
        resources.append({'name':name,'selector':selector,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'section_offsets':offsets,'animation_frame_counts':counts})
    build=subprocess.run([sys.executable,'scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=ROOT,capture_output=True,text=True);(work/'build.log').write_text(build.stdout+build.stderr);build.check_returncode()
    deps=ROOT/'private/build/rust-target/debug/deps';rlib=max(deps.glob('libgenewars_runtime-*.rlib'),key=lambda p:p.stat().st_mtime)
    exe=work/'harness.exe';harness=ROOT/'scripts/desktop/source_building_resource_harness.rs'
    subprocess.run([str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'--extern',f'genewars_runtime={rlib}','-L',f'dependency={deps}','-o',str(exe)],cwd=ROOT,env=environment(),check=True)
    subprocess.run([str(exe),str(source),str(work/'actual.bin')],check=True)
    assert (work/'actual.bin').read_bytes()==output
    receipt={'status':'passed','scope':'exact animation0/1 metadata and all component sprite pixels/opacity; no callback/draw-policy claim','resources':resources,'compared_bytes':len(output),'work':str(work.relative_to(ROOT)),'implementation_sha256':hashlib.sha256((ROOT/'rust/crates/genewars-runtime/src/source_building_resources.rs').read_bytes()).hexdigest()}
    (ROOT/'research/integration/source-building-resources.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
if __name__=='__main__':main()
