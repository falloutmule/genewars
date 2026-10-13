"""Verify and provision isolated analysis tooling; never execute game bytes."""
from pathlib import Path
import hashlib, json, zipfile

P = Path(__file__).resolve().parents[2]
def digest(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()
def unzip_checked(src, dest):
    dest = dest.resolve()
    with zipfile.ZipFile(src) as z:
        for item in z.infolist():
            out = (dest / item.filename).resolve()
            if not out.is_relative_to(dest):
                raise ValueError('unsafe zip path')
        z.extractall(dest)
metadata = json.loads((P/'research/ghidra/jdk-release.json').read_text())[0]
specs = [
    ('tools/ghidra/ghidra.zip', '85bd2990945f3a78df4d1e09f1bb1f40ab77be3bac62c6e7678900788c7f0f41', 512517017),
    ('tools/ghidra/lx-loader.zip', '8b7e7461bd48621faa5e32c27a2c7ff8b17b82c7b765c8c6afe2cee8db693cec', 313312),
    ('tools/jdk/jdk.zip', metadata['binary']['package']['checksum'], metadata['binary']['package']['size']),
]
checks=[]
for rel, expected, size in specs:
    src=P/'private'/rel
    actual=digest(src)
    assert actual==expected and src.stat().st_size==size, rel
    checks.append({'path':rel,'sha256':actual,'size':size,'published_digest_match':True})
unzip_checked(P/'private/tools/ghidra/ghidra.zip', P/'private/tools/ghidra')
unzip_checked(P/'private/tools/jdk/jdk.zip', P/'private/tools/jdk')
ghidra=next((P/'private/tools/ghidra').glob('ghidra_*_PUBLIC'))
unzip_checked(P/'private/tools/ghidra/lx-loader.zip', ghidra/'Ghidra/Extensions')
dest=P/'private/ghidra/GW-CD-candidate.exe'
with (P/'private/disc/member-004.img').open('rb') as f, dest.open('wb') as out:
    remain=1265994
    for lba in range(501, 501+(remain+2047)//2048):
        f.seek(lba*2352+16)
        payload=f.read(min(remain,2048))
        assert len(payload)==min(remain,2048)
        out.write(payload)
        remain-=len(payload)
    assert remain==0
assert digest(dest)=='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
checks.append({'path':'ghidra/GW-CD-candidate.exe','sha256':digest(dest),'size':dest.stat().st_size,'provenance':'GAME/GW.EXE;1 raw disc LBA501, MODE1/2352 payload +16'})
(P/'research/ghidra/tool-verification.json').write_text(json.dumps({'checks':checks,'jdk':metadata['release_name'],'ghidra':'12.0.1','loader':'v12.0.1','candidate_is_canonical':False,'network_failure':'Initial non-escalated curl: Could not resolve host api.adoptium.net; approved network retry succeeded.'},indent=2)+'\n')
print(json.dumps(checks,indent=2))
