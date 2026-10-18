"""Independent exact-source integration of native family4 factory transactions."""
from pathlib import Path
from datetime import datetime, timezone
import ast
import hashlib
import json
import shutil
import struct
import subprocess
import sys
from cargo import environment

ROOT=Path(__file__).resolve().parents[1]
WORK=ROOT/'private/integration/landing-world-validation'
MISSION=343958
ACTORS=MISSION
PLAYERS=ACTORS+4000
RNG=PLAYERS+4*1805
EFFECTS=RNG+7
BINDINGS=EFFECTS+200*81
SNAPSHOT=BINDINGS+200
MAP=0x355E
LAYOUTS=[(243038,300,116),(277838,512,41),(298830,20,140),(301630,256,59),(316734,200,73),(331334,32,41),(332646,32,89),(335494,16,94)]
def sha(data):return hashlib.sha256(data).hexdigest()
def pin(path):return dict(path=path.relative_to(ROOT).as_posix(),sha256=sha(path.read_bytes()))
def run(command,**kwargs):
    result=subprocess.run(command,capture_output=True,**kwargs)
    if result.returncode:
        sys.stderr.buffer.write(result.stderr)
        raise subprocess.CalledProcessError(result.returncode,command)
    return result
def address(raw):
    index,kind=struct.unpack('<hB',raw)
    assert 1<=kind<=8 and 0<=index<LAYOUTS[kind-1][1]
    if kind==3:return ACTORS+index*200
    if kind==5:return EFFECTS+index*81
    offset,_,stride=LAYOUTS[kind-1]
    return offset+index*stride
def samples(data,point):
    x,y=point;gx,gy=(x>>8)&127,(y>>8)&127;fx,fy=x&255,y&255
    heights=[];lights=[]
    for dx,dy in [(0,0),(1,0),(0,1),(1,1)]:
        offset=MAP+(((gy+dy)&127)*128+((gx+dx)&127))*14
        light,height=struct.unpack_from('<hh',data,offset)
        heights.append(height);lights.append((light*data[offset+12])>>6)
    def interpolate(v):
        a,b,c,d=v
        numerator=(d-c)*fx+(c-a)*fy if fx<fy else (b-a)*fx+(d-b)*fy
        return 2*a+(numerator>>7)
    return interpolate(heights),max(0,min(64,((interpolate(lights)-0x1E00)>>8)+16))
def main():
    WORK.mkdir(parents=True,exist_ok=True)
    sourcepath=ROOT/'private/integration/game-runtime/terrain/CONQ001.DAT'
    source=sourcepath.read_bytes()
    assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    shutil.copyfile(sourcepath,WORK/'mission.bin')
    # Reuse only the independently authored pure factory oracle, without
    # executing that validator's top-level export/build/report operations.
    oraclepath=ROOT/'scripts/ghidra/validate_landing_bridge.py'
    tree=ast.parse(oraclepath.read_text(encoding='utf-8'))
    functions=[node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='factory']
    assert len(functions)==1
    namespace={'struct':struct}
    exec(compile(ast.Module(body=functions,type_ignores=[]),str(oraclepath),'exec'),namespace)
    factory=namespace['factory']
    built=run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','-p','genewars-assets','--offline','--locked','--message-format=json'],cwd=ROOT)
    artifacts={}
    for line in built.stdout.splitlines():
        entry=json.loads(line)
        if entry.get('reason')=='compiler-artifact' and 'lib' in entry['target']['kind']:
            artifacts[entry['target']['name']]=next(p for p in entry['filenames'] if p.endswith('.rlib'))
    env=environment();exe=WORK/'probe.exe';harness=ROOT/'scripts/landing_world_probe.rs'
    command=[str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={ROOT}/private/build/rust-target/debug/deps','-o',str(exe)]
    for name in ['genewars_assets','genewars_runtime']:command+=['--extern',f'{name}={artifacts[name]}']
    run(command,cwd=ROOT,env=env);run([str(exe),str(WORK)],cwd=ROOT,env=env)
    actual=(WORK/'actual.bin').read_bytes();assert len(actual)==32*5*SNAPSHOT
    nonempty_heads=0
    for case in range(32):
        begin=case*5*SNAPSHOT;state=bytearray(actual[begin:begin+SNAPSHOT])
        assert state[:MISSION]==source
        for owner in range(4):
            player=PLAYERS+owner*1805;point=struct.unpack_from('<hh',state,player+9)
            ground,shade=samples(state,point);cell=((point[1]>>8)&127)*128+((point[0]>>8)&127)
            headpos=MAP+cell*14+6;head=bytes(state[headpos:headpos+3]);old=struct.unpack('<hB',head) if head[2] else None
            slot=owner;start=EFFECTS+slot*81
            state[start:start+81]=factory(bytes(state[start:start+81]),slot,owner,point,ground,shade,old)
            raw=struct.pack('<hB',slot,5)
            if head[2]:
                nonempty_heads+=1;other=address(head);state[other:other+3]=raw
            state[headpos:headpos+3]=raw
            state[player+0x496:player+0x499]=raw
            struct.pack_into('<h',state,0x3554,slot+1)
            state[BINDINGS+slot]=1
            offset=begin+(owner+1)*SNAPSHOT
            assert actual[offset:offset+SNAPSHOT]==state,(case,owner,'complete factory after-image differs')
    sources=[ROOT/'rust/crates/genewars-runtime/src'/name for name in ['actor_world.rs','actor_pool.rs','effect_pool.rs','landing.rs','player_state.rs','world.rs']]
    report=dict(status='passed',verified_at=datetime.now(timezone.utc).isoformat(),sessions=32,factories=128,complete_afterimages=128,
        compared_bytes=128*SNAPSHOT,nonempty_heads=nonempty_heads,repeat_rejection_rollbacks=128,
        output_sha256=sha(actual),source_pins=[pin(sourcepath),pin(oraclepath)],implementation_pins=[pin(p)for p in sources+[harness,Path(__file__)]],
        scope='Exact mission with explicit synthetic player targets/native constructor gaps/RNG. Factory core map/player/count/native-effect transactions and typed bindings. Pending audio/reveal not executed; no descent/contact, first controllable frame or original execution claim.')
    (ROOT/'research/integration/landing-world-validation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({key:report[key]for key in ['status','sessions','factories','compared_bytes','nonempty_heads','repeat_rejection_rollbacks']}))
if __name__=='__main__':main()
