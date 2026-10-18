"""Compare negative-height building movement queries with literal74270/74160."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, importlib.util, json, struct, subprocess, uuid

P = Path(__file__).resolve().parents[1]
OFFSETS = [(0,0),(256,0),(256,256),(0,256),(-256,256),(-256,0),(-256,-256),(256,-256),(7680,0)]
CLASSES = [3,1,2,3,2,2,1,2,1,2,3,3,2,1]
def cell(point): return ((point[1] >> 8) & 127) * 128 + ((point[0] >> 8) & 127)
def word(v): return (v + 32768) % 65536 - 32768
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()

def main():
    work = P/'private/review/building-obstacle'/uuid.uuid4().hex
    work.mkdir(parents=True)
    module = P/'rust/crates/genewars-runtime/src/building_obstacle.rs'
    pin = sha(module)
    spec = importlib.util.spec_from_file_location('source', P/'research/variants/initial-creature-source.py')
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    class Source(m.Source):
        def push(self, value):
            if self.pc == 0x74222: self.queries.append((self.r['EAX']-0x11c2be)//14)
            return super().push(value)
    source = Source()
    listing = P/'private/ghidra/landing-contact/instructions-0008b460.txt'
    source.code.update(m.m.Model([listing], b'').code)
    order = sorted(source.code); source.next = dict(zip(order, order[1:]))
    source.mem = {}
    order = [source.load(0x1060b8+i,1) for i in range(9)]
    assert order == [8,2,3,4,5,6,7,1,255]
    assert [tuple(m.signed(source.load(0x104564+i*4+j*2,2),16) for j in range(2)) for i in order] == OFFSETS
    assert [source.load(0x104de0+i,1) for i in range(14)] == CLASSES
    cases = []
    for selector in range(14):
        for index in range(9):
            for boundary in [-1,0,1]:
                extent = {1:85,2:128,3:256}[CLASSES[selector]]
                cases.append((selector,index,extent+boundary,(32767,-32768),1,False))
    cases += [(1,0,d,(4096,3584),flag,multiple) for d in [0,85,86,-85,-86,-32768]
              for flag in [0,1] for multiple in [False,True]]
    cases += [(0,-1,0,(32767,-32768),1,False)]
    data = bytearray(); expected = bytearray(); steps = 0
    for selector,index,delta,point,flag,multiple in cases:
        records = [bytearray(67) for _ in range(9)]
        if index >= 0:
            r = records[index]; r[6]=4; r[45]=selector
            struct.pack_into('<hh',r,7,word(point[0]+delta),point[1])
        if multiple:
            r = records[1]; r[6]=4; r[45]=1
            struct.pack_into('<hh',r,7,*point)
        state = {0x200000+j:b for j,b in enumerate(struct.pack('<hh',*point))}
        for i,(dx,dy) in enumerate(OFFSETS):
            at = cell((word(point[0]+dx),word(point[1]+dy)))
            raw = bytearray(14)
            if records[i][6]: struct.pack_into('<hB',raw,6,i,4)
            if i==0: raw[11]=flag
            state.update({0x11c2be+at*14+j:b for j,b in enumerate(raw)})
            state.update({0x1645ae+i*67+j:b for j,b in enumerate(records[i])})
        source.queries=[]
        report,_,_=source.run(entry=0x74270,state_before=state,registers={'EAX':0x200000,'EDX':0,'EBX':0xffffffff})
        result = source.r['EAX'] != 0
        indices = [cell((word(point[0]+dx),word(point[1]+dy))) for dx,dy in OFFSETS]
        visited = [indices.index(q) for q in source.queries]
        expected.extend(bytes([int(result),len(visited)]+visited+[255]*(9-len(visited))))
        data.extend(struct.pack('<hhB',*point,flag)+b''.join(records)); steps += report['steps']
    harness=work/'probe.rs'
    harness.write_text(f'#[path="{module.as_posix()}"]mod obstacle;'+r'''
use std::io::{Read,Write};
fn main(){let mut data=Vec::new();std::io::stdin().read_to_end(&mut data).unwrap();let mut out=Vec::new();
for c in data.chunks_exact(608){let point=[i16::from_le_bytes(c[0..2].try_into().unwrap()),i16::from_le_bytes(c[2..4].try_into().unwrap())];
let cell=|p:[i16;2]|((((i32::from(p[1])>>8)&127)*128)+((i32::from(p[0])>>8)&127))as usize;
let offsets:[[i16;2];9]=[[0,0],[256,0],[256,256],[0,256],[-256,256],[-256,0],[-256,-256],[256,-256],[7680,0]];
let indices:Vec<_>=offsets.iter().map(|v|cell([point[0].wrapping_add(v[0]),point[1].wrapping_add(v[1])])).collect();
let mut map=vec![0;229376];map[cell(point)*14+11]=c[4];let mut queries=Vec::new();
let result=obstacle::move_obstacle(&map,point,|at|{let i=indices.iter().position(|v|*v==at).unwrap();queries.push(i as u8);let record:[u8;67]=c[5+i*67..5+(i+1)*67].try_into().unwrap();Ok::<_,()>(if record[6]==0{None}else{Some(record)})}).unwrap();
out.push(result as u8);out.push(queries.len()as u8);queries.resize(9,255);out.extend(queries);}
std::io::stdout().write_all(&out).unwrap();}
''',encoding='utf-8')
    exe=work/'probe.exe'
    subprocess.run([str(P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'),'--edition=2021',str(harness),'-o',str(exe)],check=True)
    actual=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout
    assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
    assert pin==sha(module),'module changed during proof'
    (work/'input.bin').write_bytes(data);(work/'expected.bin').write_bytes(expected);(work/'actual.bin').write_bytes(actual)
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=len(cases),compared_bytes=len(actual),literal_instructions=steps,
        implementation_pins={module.relative_to(P).as_posix():pin,Path(__file__).relative_to(P).as_posix():sha(Path(__file__))},
        listing_pins={p.relative_to(P).as_posix():sha(p)for p in source.files+[listing]},private_run=work.relative_to(P).as_posix(),original_execution=False,
        scope='Mask0/negative-height74270 including literal74160/8B460. Exact returned-presence and ordered cell-query sequence: every selector, every ordered offset, extent boundaries, signed-word wrap, reserved empty cell, clear flag and first-candidate rejection. Native map-list traversal is the world adapter; positive-height animation bounds remain unsupported.')
    (P/'research/integration/building-obstacle-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:receipt[k]for k in ['status','cases','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
