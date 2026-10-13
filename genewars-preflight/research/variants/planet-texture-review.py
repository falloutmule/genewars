"""Compare all four exact PL assets with independent texture-address gathering."""
from datetime import datetime,timezone
import hashlib,json
from pathlib import Path
import subprocess,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
sha=lambda data:hashlib.sha256(data).hexdigest()
def record(path):
    data=path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(),size=len(data),sha256=sha(data))

def main():
    evidence_path=ROOT/'research/ghidra/planet-texture-evidence.json'
    evidence=json.loads(evidence_path.read_bytes())
    listings=[]
    for item in evidence['private_listings']:
        actual=record(ROOT/item['private_path']);assert actual['sha256']==item['sha256'];listings.append(actual)
    assert record(ROOT/'private/ghidra/GW-CD-candidate.exe')['sha256']==evidence['candidate_sha256']
    corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results']
    scratch=ROOT/'private/review/planet-textures';scratch.mkdir(parents=True,exist_ok=True)
    exe=scratch/'planet-texture-review.exe';source=Path(__file__).with_suffix('.rs')
    command=[str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow','-O',str(source),'-o',str(exe)]
    built=subprocess.run(command,capture_output=True,env=environment(),timeout=60)
    assert built.returncode==0,built.stderr.decode(errors='replace')
    # Explicit atlas coordinates derived from the consumer's two placement branches.
    positions=[(0,0,128),(128,0,128),(0,128,128),(128,128,128),
        (0,256,128),(128,256,128),(0,384,128),(128,384,128),
        (0,512,64),(64,512,64),(128,512,64),(192,512,64),(0,576,64),(64,576,64)]
    assets=[];pixels=0;opaque=0
    for asset in evidence['assets']:
        name=asset['source_path'].rsplit('/',1)[-1].split(';')[0]
        raw=ROOT/'private/reference-run/run-001/drive-c/GAME/DATA'/name
        assert sha(raw.read_bytes())==asset['source_sha256']
        cached=next(v for v in corpus if v['input_sha256']==asset['source_sha256'])
        data=(ROOT/cached['private_output']).read_bytes()
        assert len(data)==163840 and sha(data)==asset['decoded_sha256']==cached['output_sha256']
        output=scratch/name;output.mkdir(exist_ok=True)
        native=subprocess.run([str(exe),str(raw),str(output)],capture_output=True,timeout=15)
        assert native.returncode==0,native.stderr.decode(errors='replace')
        checks=json.loads(native.stdout);assert checks['indices']==155648
        assert (output/'decoded.bin').read_bytes()==data
        tiles=[];visited=set()
        for kind,(column,row,side) in enumerate(positions):
            addresses=[(row+y)*256+column+x for y in range(side) for x in range(side)]
            assert not visited.intersection(addresses);visited.update(addresses)
            indices=bytes(data[address] for address in addresses);mask=bytes(int(v!=0) for v in indices)
            assert (output/f'tile-{kind:02}.indices').read_bytes()==indices
            assert (output/f'tile-{kind:02}.mask').read_bytes()==mask
            proof=asset['tiles'][kind]
            assert (proof['type'],proof['width'],proof['height'],proof['source_offset'],proof['end_exclusive'])==(kind,side,side,addresses[0],addresses[-1]+1)
            assert sha(indices)==proof['indices_sha256'] and sha(mask)==proof['mask_sha256']
            assert sum(mask)==proof['opaque']
            pixels+=len(indices);opaque+=sum(mask)
            tiles.append(dict(type=kind,indices=len(indices),opaque=sum(mask),first_address=addresses[0],last_address=addresses[-1],indices_sha256=sha(indices),mask_sha256=sha(mask)))
        assert len(visited)==155648 and max(visited)==163711
        assets.append(dict(entry_id=asset['entry_id'],source=record(raw),decoded=record(output/'decoded.bin'),tiles=tiles,native_checks=checks,uninterpreted_atlas_bytes=len(data)-len(visited)))
    assert pixels==622592
    result=dict(status='passed',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
        candidate_sha256=evidence['candidate_sha256'],evidence=record(evidence_path),harness=record(source),native_harness=record(exe),
        reviewed_source=record(ROOT/'rust/crates/genewars-assets/src/lib.rs'),listing_hash_checks=listings,
        asset_count=4,tile_count=56,indices_compared=pixels,opacity_bytes_compared=pixels,opaque_pixels=opaque,assets=assets,
        limits=['Exact PL1..4 inputs and types0..13 only; unused8192bytes peratlas uninterpreted',
            'No palette association, interpolation, projection, clipping, draw order or live rendering established',
            'PLANETS.DAT consumer remains unproven; numbered textures are a separate resource path'])
    Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(status='passed',assets=4,tiles=56,indices=pixels,opacity_bytes=pixels,opaque=opaque)))

if __name__=='__main__':main()
