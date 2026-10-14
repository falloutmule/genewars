"""Reproduce original victory background/font inputs; no game execution."""
from pathlib import Path
import hashlib,json,subprocess,tempfile
from source_member import SourceReader
ROOT=Path(__file__).resolve().parents[1]
ROWS=[['WINSCR.DAT', 'root/2/disc/145', '5e73b027974f8b1d33c04ee89193aa237b0450b09aea4ed63f2a9996e307d0df', 40515, '2dee4b93857b9de395b89919b66baa7e3086f00c0de072f62f8bca3824c12ea1', 307200], ['WINSCR.PAL', 'root/2/disc/146', '2d084db840f3085df0b956d7f64260432df07b48fdd9e61860cedbb7a9ab7788', 701, '78105d9cf6835259782ee5559a8ec455f8668d4e40d2ae6215f05fc59bc30af9', 768], ['FONT7.DAT', 'root/2/disc/83', 'cd7ccbfb005ee78ef29e29d620a34cb9e275522c4c96b963f84c3abda5757326', 1541, '60dbffae53efe67f5ce634c71c0e97b91a2d4514d8eaf7796d947fc106ad6e0e', 14994], ['FONT7.TAB', 'root/2/disc/84', '62a8476b908544ee6c1d4c6a9c28b1c294c41cdd520f45cc589ed69f90221ff8', 551, '73d5433364795a4d2fabdab1489e130120483f30e3d07e13c5a75aab55b3102b', 1170]]
def stage():
    output=ROOT/'private/integration/result-inputs'
    output.mkdir(parents=True,exist_ok=True)
    with SourceReader() as reader:
        for name,eid,packed_sha,packed_size,digest,size in ROWS:
            target=output/name
            if target.exists():
                data=target.read_bytes()
                if len(data)!=size or hashlib.sha256(data).hexdigest()!=digest: raise ValueError('changed result input '+name)
                continue
            raw=reader.read(eid,max_bytes=1024*1024)
            if len(raw)!=packed_size or hashlib.sha256(raw).hexdigest()!=packed_sha: raise ValueError('changed CD source '+eid)
            with tempfile.TemporaryDirectory(dir=output) as scratch:
                decoded=Path(scratch)/name
                if raw[:3]==b'RNC':
                    subprocess.run([str(ROOT/'private/build/rust-target/debug/genewars-tools.exe'),'rnc-decode','-',str(decoded),str(1024*1024)],input=raw,check=True,capture_output=True,timeout=10)
                    data=decoded.read_bytes()
                else:data=raw
                if len(data)!=size or hashlib.sha256(data).hexdigest()!=digest: raise ValueError('decoded result differs '+name)
                with target.open('xb') as stream:stream.write(data)
    return {'status':'passed','resources':len(ROWS),'original_execution':False}
if __name__=='__main__':print(json.dumps(stage()))
