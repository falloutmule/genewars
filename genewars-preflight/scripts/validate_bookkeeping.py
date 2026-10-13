"""Listing-based numeric review of empty building bookkeeping and countdown.

Only arithmetic instructions are interpreted. Audio/format output and shared
UI animation callbacks are explicit side-effect boundaries, not executed code.
"""
from pathlib import Path
import importlib.util,hashlib,json,random,struct,subprocess,sys
from datetime import datetime,timezone
from cargo import environment
P=Path(__file__).resolve().parents[1];W=P/'private/review/bookkeeping';W.mkdir(parents=True,exist_ok=True)
def pin(p):return {'path':p.relative_to(P).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def load_module(path):
    spec=importlib.util.spec_from_file_location('model',path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def main():
    m=load_module(P/'research/variants/initial-creature-source.py');model=m.Source()
    listings=[P/'private/ghidra/mission-array-refs'/f'instructions-{a}.txt'for a in ['00047f80','000437a0','00013780']]
    assert [pin(p)['sha256']for p in listings]==[
        'bae7d9800c80bce7b1419e5b7a2193b1f934b5080dd30285c7a2a15b50a2a4e4',
        '7aa65558885556472096f9f32bc7e469dcc4f227385c258451ba4c105343156d',
        '634efec82d190cdf793a444a423709cd2e962031a6b2e20ec05c8606bc3a36b4']
    # Add exact listing data to the existing bounded arithmetic interpreter.
    extension=m.m.Model(listings,b'');model.code.update(extension.code)
    order=sorted(model.code);model.next=dict(zip(order,order[1:]))
    # Output boundaries preserve the source calling convention. These return
    # only; no numeric game-state effect from those helpers is asserted.
    for address,cleanup in [(0x981d3,0),(0x41ce0,20),(0x41910,16),(0x984c0,0)]:
        model.code[address]=('RET',[hex(cleanup)]if cleanup else[])
    rand=random.Random(4700437);inputs=bytearray();expected=bytearray();steps=0;boundaries=set()
    periods=[0,1,3,4,6000,65535]
    for case in range(48):
        period=periods[case%len(periods)];players=[]
        for player in range(4):
            r=bytearray(rand.randbytes(1805));r[1]=player;r[0x4a1]=player;r[0x4a2]=0
            r[0x5e5]=[0,49,255][(case+player)%3]
            struct.pack_into('<i',r,0x4a7,[-1,0,1,999][(case+player)%4])
            for i in range(8):struct.pack_into('<H',r,0x5ea+2*i,[0,1,999,1000,65535][(case+player+i)%5])
            struct.pack_into('<I',r,0x499,[0,1,700,0xffffffff][(case+player)%4]);players.append(r)
        state={0x118d74+player*1805+j:b for player,r in enumerate(players)for j,b in enumerate(r)}
        state.update({0x11878c:4,0x11a9da:period&255,0x11a9db:period>>8})
        # Source.run owns selected-player global=0; all four player records
        # still participate, including three nonselected notification paths.
        selected=0
        report,_,trace=model.run(state_before=state,entry=0x47f80)
        inputs+=struct.pack('<BH',selected,period)+b''.join(players)
        expected+=bytes(model.saved_state[0x118d74+p*1805+i]for p in range(4)for i in range(1805))
        steps+=report['steps'];boundaries.update(model.calls)
    (W/'players-input.bin').write_bytes(inputs);(W/'players-expected.bin').write_bytes(expected)
    countdown=bytearray();count_expected=bytearray()
    # Countdown model follows only source13780 numeric branches; expiry is
    # rejected because its source factories are outside this admitted slice.
    for count in [0,1,2,6000,65534,65535]:
      for period in periods:
       for progress in [0,14,20,-1]:
        for enabled in [0,1,255]:
         for flags in [0,64,128,192]:
          countdown+=struct.pack('<HHbBB',count,period,progress,enabled,flags)
          special=progress in [14,20]
          expiry=count==1 and((special and enabled>0)or not special)
          if expiry:count_expected+=bytes([1,0,0,0]);continue
          state={0x118d74+i:0 for i in range(4*1805)}
          state.update({0x16da36+i:0 for i in range(16*102)})
          state.update({0x11878c:4,0x11a9da:period&255,0x11a9db:period>>8,0x11c2bc:count&255,0x11c2bd:count>>8,
            0x11945b:progress&255,0x11937b:enabled,0x118770:flags})
          report,_,_=model.run(state_before=state,entry=0x13780)
          actual=model.load(0x11c2bc,2);count_expected+=struct.pack('<BHB',0,actual,count==0);steps+=report['steps']
    (W/'countdown-input.bin').write_bytes(countdown);(W/'countdown-expected.bin').write_bytes(count_expected)
    cmd=[sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked']
    subprocess.run(cmd,cwd=P,check=True,capture_output=True)
    target=P/'private/build/rust-target/debug';rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
    cmd=[str(rustc),'--edition=2021','scripts/bookkeeping_probe.rs','-L',f'dependency={target}/deps','--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-o',str(W/'probe.exe')]
    subprocess.run(cmd,cwd=P,env=environment(),check=True,capture_output=True);subprocess.run([str(W/'probe.exe'),str(W)],check=True)
    for name in ['players','countdown']:
        a=(W/f'{name}-actual.bin').read_bytes();e=(W/f'{name}-expected.bin').read_bytes()
        assert a==e,(name,next((i for i,(x,y)in enumerate(zip(a,e))if x!=y),None),len(a),len(e))
    receipt=dict(status='passed-bounded-empty-bookkeeping',utc=datetime.now(timezone.utc).isoformat(),player_cases=192,countdown_cases=len(countdown)//7,
        compared_bytes=len(expected)+len(count_expected),literal_instruction_steps=steps,instruction_pins=[pin(p)for p in listings],
        source_pins=[pin(P/p)for p in ['scripts/validate_bookkeeping.py','scripts/bookkeeping_probe.rs','rust/crates/genewars-runtime/src/bookkeeping.rs','research/variants/initial-creature-source.py']],
        boundaries=['No buildings or family8 entities','Audio/format and shared UI animation bodies excluded at explicit calling-convention boundaries','Countdown expiry requires factories and rejects','No complete original scheduler or startup claim'],original_execution=False)
    (P/'research/integration/bookkeeping-validation.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
if __name__=='__main__':main()
