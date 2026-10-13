"""Independent source6A540 numeric scheduler replay; UI animation calls pending."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,random,struct,subprocess,sys
from cargo import environment
P=Path(__file__).resolve().parents[1];W=P/'private/review/family-scheduler'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    W.mkdir(parents=True,exist_ok=True)
    spec=importlib.util.spec_from_file_location('scheduler_source',P/'research/variants/initial-creature-source.py')
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);model=m.Source()
    listing=P/'private/ghidra/mission-array-refs/instructions-0006a540.txt'
    assert sha(listing)=='d947fde62e1f4a044d4c74953d8c95a1b25b0e4144fd43f64a1d148f4b5cca1b'
    extension=m.m.Model([listing],b'');model.code.update(extension.code)
    order=sorted(model.code);model.next=dict(zip(order,order[1:]))
    # Only these eight explicit presentation calls are deferred. The same
    # helper remains fully interpreted when reached by the Mule callback.
    ui_calls=[0x6a633,0x6a647,0x6a65b,0x6a66f,0x6a683,0x6a697,0x6a6ab,0x6a6bf]
    for address in ui_calls:
        assert model.code[address]==('CALL',['0x0007ac00'])
        model.code[address]=('NOP',[])
    randomizer=random.Random(0x6a540);inputs=bytearray();expected=bytearray();steps=0
    cases=[]
    for seed in [0,5428,10548,65535]:
      for count in [0,1,4]:
        players=[]
        for player in range(4):
            record=bytearray(randomizer.randbytes(1805));record[1]=player;record[0x4a1]=player;players.append(record)
        inputs+=struct.pack('<HB',seed,count)+b''.join(players)
        state={0x118d74+p*1805+i:b for p,r in enumerate(players)for i,b in enumerate(r)}
        state.update({0x11878c:count,0x11c2b0:1,0x11c2b1:0,0x11872c:0})
        state.update({0x118782+i:0 for i in range(4)})
        report,native,trace=model.run(seed=seed,state_before=state,entry=0x6a540)
        assert all(address in trace for address in ui_calls)
        expected+=bytes(model.load(0x118d74+p*1805+i,1)for p in range(4)for i in range(1805))
        native=bytearray(native);native[31:39]=bytes(8);expected+=native
        expected+=bytes(model.load(0x11c2be+i,1)for i in range(229376))
        expected+=struct.pack('<HBI',report['rng_seed_after'],report['rng_ring_after'],report['rng_draw_count_after'])
        for i,offset in enumerate([0,0x15e,0x8c,0x172,0xd2,0xdc,0xe6,0xf0]):
            expected+=struct.pack('<IHI',0x116818 if i==1 else 0x11680c,offset,0x184cbc+8*i)
        steps+=report['steps'];cases.append(dict(seed=seed,player_count=count,draws=report['draws']))
    (W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected)
    subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
    target=P/'private/build/rust-target/debug';exe=W/'probe.exe'
    cmd=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/family_scheduler_probe.rs','-L',f'dependency={target}/deps','-o',str(exe)]
    for name in ['genewars_runtime','genewars_assets']:cmd+=['--extern',f'{name}={target}/lib{name}.rlib']
    subprocess.run(cmd,cwd=P,env=environment(),capture_output=True,check=True)
    subprocess.run([str(exe),str(P)],cwd=P,env=environment(),check=True)
    actual=(W/'actual.bin').read_bytes()
    assert actual==expected,dict(actual=len(actual),expected=len(expected),first_difference=next((i for i,(a,e)in enumerate(zip(actual,expected))if a!=e),None))
    files=['scripts/validate_family_scheduler.py','scripts/family_scheduler_probe.rs','rust/crates/genewars-runtime/src/actor_world/family_scheduler.rs','research/variants/initial-creature-source.py']
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=len(actual),literal_instruction_steps=steps,
        listing_sha256=sha(listing),source_pins={p:sha(P/p)for p in files},output_sha256=hashlib.sha256(actual).hexdigest(),
        scope='Source6A540 first neutral-Mule pass, player-count reset range, entire player/native/map afterimages, RNG and eight ordered UI request descriptors. Native resource/vtable pointers normalized to owned typed bindings. UI animation bodies deferred only at eight scheduler callsites; not a complete original startup.',original_execution=False)
    (P/'research/integration/family-scheduler-validation.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
if __name__=='__main__':main()
