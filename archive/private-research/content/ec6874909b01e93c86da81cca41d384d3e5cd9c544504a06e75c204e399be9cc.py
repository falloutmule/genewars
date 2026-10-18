"""Independent complete byte comparison for composed airborne callbacks."""
from pathlib import Path
from datetime import datetime,timezone
import ast,json,shutil,struct,sys
from validate_landing_world import ROOT,MAP,MISSION,PLAYERS,RNG,EFFECTS,BINDINGS,SNAPSHOT,sha,pin,run,address,samples
from cargo import environment
from physics_inputs import stage

WORK=ROOT/'private/integration/landing-step-validation'
def i16(v):return (v+32768)%65536-32768
def main():
    WORK.mkdir(parents=True,exist_ok=True)
    for path in stage().glob('*.bin'):shutil.copyfile(path,WORK/path.name)
    sourcepath=ROOT/'private/integration/game-runtime/terrain/CONQ001.DAT'
    assert sha(sourcepath.read_bytes())=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    shutil.copyfile(sourcepath,WORK/'mission.bin')
    oracle=ROOT/'scripts/ghidra/validate_landing_bridge.py'
    tree=ast.parse(oracle.read_text(encoding='utf-8'))
    nodes=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='factory'];assert len(nodes)==1
    ns={'struct':struct};exec(compile(ast.Module(body=nodes,type_ignores=[]),str(oracle),'exec'),ns);factory=ns['factory']
    mem=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes()
    sb=mem[0x7EE8:0x7EE8+8192];cb=mem[0x86E8:0x86E8+8192]
    assert sha(sb)=='84dfe6902e4c8947b75f4ff0a54d82fced478dd9682255eaa973a88cbf76cf3c'
    assert sha(cb)=='688e629369dc1c5379bae04b5e11100a4fd0aabd89745f2c0b5b6309ce0261c9'
    sine=struct.unpack('<2048i',sb);cosine=struct.unpack('<2048i',cb)
    build=run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked','--message-format=json'],cwd=ROOT)
    artifacts={}
    for line in build.stdout.splitlines():
        item=json.loads(line)
        if item.get('reason')=='compiler-artifact' and 'lib' in item['target']['kind']:
            artifacts[item['target']['name']]=next(p for p in item['filenames'] if p.endswith('.rlib'))
    harness=ROOT/'scripts/landing_step_probe.rs';exe=WORK/'probe.exe';env=environment()
    cmd=[str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={ROOT}/private/build/rust-target/debug/deps','-o',str(exe)]
    for name in ['genewars_assets','genewars_runtime']:cmd+=['--extern',f'{name}={artifacts[name]}']
    run(cmd,cwd=ROOT,env=env);run([str(exe),str(WORK)],cwd=ROOT,env=env)
    actual=(WORK/'actual.bin').read_bytes();assert len(actual)==16*9*SNAPSHOT
    for case in range(16):
        begin=case*9*SNAPSHOT;state=bytearray(actual[begin:begin+SNAPSHOT])
        for step in range(8):
            parent=EFFECTS;point=struct.unpack_from('<hh',state,parent+7);ground,shade=samples(state,point)
            height=struct.unpack_from('<i',state,parent+39)[0]-136
            struct.pack_into('<i',state,parent+39,height);struct.pack_into('<i',state,parent+17,ground);state[parent+23]=shade
            seed,ring,count=struct.unpack_from('<HBI',state,RNG);high=[]
            for _ in range(3):seed=seed*0xE62D&65535;high.append(seed>>8)
            yaw,pitch,speed=high[2]*8,1024-high[1]%128,2560+64*(high[0]%10)
            slot=step+1;start=EFFECTS+slot*81;prior=bytes(state[start:start+81]);cell=((point[1]>>8)&127)*128+((point[0]>>8)&127)
            headpos=MAP+cell*14+6;head=bytes(state[headpos:headpos+3]);old=struct.unpack('<hB',head) if head[2] else None
            effect=bytearray(factory(prior,slot,0,point,ground,shade,old));effect[69]=6;effect[73]=prior[73];effect[76]=prior[76]
            struct.pack_into('<i',effect,39,height);struct.pack_into('<hhh',effect,57,yaw,pitch,speed)
            horizontal=cosine[(pitch-512)%2048]*speed>>16
            struct.pack_into('<hhh',effect,45,i16(cosine[(yaw-512)%2048]*horizontal>>16),i16(sine[(yaw-512)%2048]*horizontal>>16),i16(-sine[(pitch-512)%2048]*speed>>16))
            struct.pack_into('<h',effect,55,-16);seed=seed*0xE62D&65535;effect[29]=1+(seed>>8)%3;effect[21]|=1
            state[start:start+81]=effect
            identity=struct.pack('<hB',slot,5)
            if head[2]:other=address(head);state[other:other+3]=identity
            state[headpos:headpos+3]=identity;struct.pack_into('<h',state,0x3554,slot+1);state[BINDINGS+slot]=6
            struct.pack_into('<HBI',state,RNG,seed,(ring+4)&31,(count+4)&0xFFFFFFFF)
            offset=begin+(step+1)*SNAPSHOT
            assert state==actual[offset:offset+SNAPSHOT],(case,step,'composed callback differs')
    pins=[ROOT/'rust/crates/genewars-runtime/src'/p for p in ['actor_world.rs','actor_world/effect_updates.rs','effect_pool.rs','landing.rs','physics_tables.rs']]
    report=dict(status='passed',verified_at=datetime.now(timezone.utc).isoformat(),sessions=16,callbacks=128,draws=512,compared_bytes=128*SNAPSHOT,output_sha256=sha(actual),source_pins=[pin(sourcepath),pin(oracle)],implementation_pins=[pin(p) for p in pins+[harness,Path(__file__)]],scope='Exact map and load-derived physics with explicit player/native-prior/RNG fixtures. Complete shared-state airborne phase0+smoke-factory comparisons. Other family callbacks and smoke updates intentionally absent; not a full scheduler or original startup trace.')
    (ROOT/'research/integration/landing-step-validation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ['status','sessions','callbacks','draws','compared_bytes']}))
if __name__=='__main__':main()
