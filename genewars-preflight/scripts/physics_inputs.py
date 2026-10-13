"""Stage exact mapped CD tables; no game execution or broad extraction."""
from pathlib import Path
import hashlib
import json

ROOT=Path(__file__).resolve().parents[1]
TABLES=(
    ('angle.bin','memory-.object1.bin',0xAE134-0x10000,514,'1c26f16f88a3e0cedc9c879837b3849567ff70514c033eb6249bd95a264debce'),
    ('bounce.bin','memory-.object3.bin',0x4530,16,'c62a747206c89218e01102645341f151f5308fe1259b177f9d0418a67575e5e5'),
    ('plant-spread.bin','memory-.object3.bin',0x4564,36,'8aa88c050cdb4229467f43347efcd9a8d88e5651cf2684e92666dcfe886515b6'),
)
def stage():
    directory=ROOT/'private/integration/physics-tables'
    prepared=[]
    for name,source,offset,size,digest in TABLES:
        origin=ROOT/'private/ghidra/validated'/source
        with origin.open('rb') as stream:
            stream.seek(offset);data=stream.read(size)
        if len(data)!=size or hashlib.sha256(data).hexdigest()!=digest:
            raise ValueError('Mapped physics table differs: '+source)
        target=directory/name
        if target.exists() and target.read_bytes()!=data:
            raise ValueError('Refusing changed staged physics table: '+name)
        prepared.append((target,data,origin,offset,digest))
    directory.mkdir(parents=True,exist_ok=True)
    for target,data,*_ in prepared:
        if not target.exists():
            with target.open('xb') as stream:stream.write(data)
    report={'candidate_sha256':'4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
        'tables':[dict(path=p.relative_to(ROOT).as_posix(),source=s.relative_to(ROOT).as_posix(),offset=o,size=len(b),sha256=h) for p,b,s,o,h in prepared],
        'scope':'Pinned AE134 angle,104530 terrain bounce and104564 plant spread direction tables. Runtime gravity/water are explicit state; neither is selected here.'}
    (ROOT/'research/integration/physics-inputs.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    return directory
if __name__=='__main__':print(stage())
