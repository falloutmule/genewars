"""Independent ordered numeric reveal model against owned Rust world afterimages."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, math, random, struct, sys
from validate_actor_world import ROOT, run, pin, LAYOUTS
from cargo import environment

WORK=ROOT/'private/integration/reveal-validation'
MAP=0x355e

def height(data,x,y):
    return struct.unpack_from('<h',data,MAP+((y&127)*128+(x&127))*14+2)[0]

def sample(data,x,y,light=False):
    cx,cy=(x>>8)&127,(y>>8)&127;fx,fy=x&255,y&255
    def vertex(dx,dy):
        if not light:return height(data,cx+dx,cy+dy)
        p=MAP+(((cy+dy)&127)*128+((cx+dx)&127))*14
        return (struct.unpack_from('<h',data,p)[0]*data[p+12])>>6
    a,b,c,d=vertex(0,0),vertex(1,0),vertex(0,1),vertex(1,1)
    value=2*a+(((d-c)*fx+(c-a)*fy if fx<fy else (b-a)*fx+(d-b)*fy)>>7)
    return max(0,min(64,((value-0x1e00)>>8)+16)) if light else value

def model(source,target):
    data=bytearray(source);tx,ty=target;center=sample(data,tx,ty);changed=visits=writes=0
    for row in range(33):
        y=(((ty-4096)&65535)>>8)+row;y&=255
        for col in range(33):
            x=((((tx-4096)&65535)>>8)+col)&255;cx,cy=x&127,y&127
            p=MAP+(cy*128+cx)*14;old=data[p+12]
            if old>=63:continue
            dx=((tx-(x<<8)+16384)&32767)-16384;dy=((ty-(y<<8)+16384)&32767)-16384
            dz=(2*height(data,cx,cy)-center)>>3;distance=dx*dx+dy*dy+dz*dz
            if distance>=4096**2:continue
            value=63 if distance<3840**2 else (4095-math.isqrt(distance))//4
            if value<=old:continue
            data[p+12]=value;changed+=1
            for xx,yy in [(cx,cy),(cx-1,cy),(cx-1,cy-1),(cx,cy-1)]:
                cell=MAP+((yy&127)*128+(xx&127))*14;data[cell+11]|=12;visits+=1
                index,kind=struct.unpack_from('<hB',data,cell+6)
                while kind:
                    base,count,stride=LAYOUTS[kind-1];assert 0<=index<count
                    record=base+index*stride;assert data[record+6]==kind
                    if kind==2 and not data[record+0x15]&2:
                        at=struct.unpack_from('<2h',data,record+7);data[record+0x17]=sample(data,*at,light=True);writes+=1
                    else:assert kind in (1,2,3,5)
                    index,kind=struct.unpack_from('<hB',data,record+3)
    return data+struct.pack('<3I',changed,visits,writes),[changed,visits,writes]

def main():
    WORK.mkdir(parents=True,exist_ok=True)
    source_path=ROOT/'private/integration/game-runtime/terrain/CONQ001.DAT';source=source_path.read_bytes()
    assert hashlib.sha256(source).hexdigest()=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    pins=[]
    expected_pins=['338546946ce863304620159905fd171845a8d655c4b09f0f6657b6a58fedc966',
        '9424f5091c9fdb3baa1f4e27c53310c85cdbf87a51ce4fa0aa11b54dadb363a5',
        '6ae63cd69b0f93d01d7a7850c7a2c9c7c56d11b5ff22db129edd1cc7e2819832',
        '2f96fb7289506e9382a8eed3576dfd894b5fba8eaad980b71cdaca39db0451cb',
        'b91628625841cfa04088d9babedbb03644231d8e019edd92ac86e8a806241e69',
        'dba12fb47f1658bff3d7fa8bfcee4cc648c065194e697f4a85f5958592291223']
    for folder,name in [('camera-player-input','00075b00'),('camera-player-input','00084b40'),('camera-player-input','0008b800'),('camera-player-input','0007b240'),('entity-plant','0001f240'),('entity-plant','0001f280')]:
        path=ROOT/f'private/ghidra/{folder}/instructions-{name}.txt';assert path.read_text().startswith(name+' ');pins.append(pin(path))
    assert [p['sha256']for p in pins]==expected_pins
    assert hashlib.sha256((ROOT/'private/ghidra/GW-CD-candidate.exe').read_bytes()).hexdigest()=='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
    object3=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes()
    for table,expected in [(0x111fec,0x1f240),(0x1120ac,0x1f280),(0x112194,0x1f240),(0x112254,0x1f240)]:
        assert struct.unpack_from('<I',object3,table-0x100000+0x6c)[0]==expected
    rng=random.Random(0x75b00);targets=[(20736,13568),(16384,15872),(0,0),(32767,32767),(-32768,-32768),(-1,-1)]
    targets += [struct.unpack_from('<2h',source,277838+i*41+7)for i in range(512)if source[277838+i*41+6]==2][:10]
    targets += [(rng.randrange(-32768,32768),rng.randrange(-32768,32768))for _ in range(16)]
    (WORK/'mission.bin').write_bytes(source);(WORK/'targets.bin').write_bytes(b''.join(struct.pack('<2h',*t)for t in targets))
    expected=bytearray();counts=[]
    for target in targets:
        result,count=model(source,target);expected.extend(result);counts.append(count)
    build=run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked','--message-format=json'],cwd=ROOT)
    artifacts={}
    for line in build.stdout.splitlines():
        item=json.loads(line)
        if item.get('reason')=='compiler-artifact' and 'lib' in item['target']['kind']:artifacts[item['target']['name']]=next(p for p in item['filenames']if p.endswith('.rlib'))
    harness=ROOT/'scripts/reveal_probe.rs';exe=WORK/'probe.exe';env=environment()
    command=[str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={ROOT}/private/build/rust-target/debug/deps','-o',str(exe)]
    for name in ['genewars_assets','genewars_runtime']:command+=['--extern',f'{name}={artifacts[name]}']
    run(command,cwd=ROOT,env=env);run([str(exe),str(WORK)],cwd=ROOT,env=env)
    actual=(WORK/'native.bin').read_bytes();assert actual==expected,next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None)
    paths=[ROOT/'rust/crates/genewars-runtime/src/actor_world.rs',ROOT/'rust/crates/genewars-runtime/src/actor_world/reveal.rs',ROOT/'rust/crates/genewars-core/src/terrain.rs',harness,Path(__file__)]
    receipt=dict(status='passed',verified_at=datetime.now(timezone.utc).isoformat(),source_pin=pin(source_path),instruction_pins=pins,implementation_pins=[pin(p)for p in paths],cases=len(targets),afterimage_bytes=len(actual),afterimage_sha256=hashlib.sha256(actual).hexdigest(),cases_detail=[dict(target=t,changed_vertices=c[0],cell_visits=c[1],plant_shade_writes=c[2])for t,c in zip(targets,counts)],scope='Independent ordered radius4096 factor, four-cell dirty flag and eligible plant-shade callback model. Exact mission bytes, explicit native priors; actors/creatures/players/RNG unchanged; repeated reveal idempotent. Presentation invalidation/minimap callbacks remain pending; no original program execution.')
    (ROOT/'research/integration/reveal-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:receipt[k]for k in ['status','cases','afterimage_bytes','afterimage_sha256']}))
if __name__=='__main__':main()
