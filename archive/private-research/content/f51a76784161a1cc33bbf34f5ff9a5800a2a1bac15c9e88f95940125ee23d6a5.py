"""Literal13780 timer branches with factory/callback/presentation request ports."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,struct,subprocess,uuid
from cargo import environment
P=Path(__file__).resolve().parents[1]
def sha(f):return hashlib.sha256(f.read_bytes()).hexdigest()
def main():
    spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    class Source(m.Source):
        def get(self,operand):
            if operand=='0x00041910':self.effects+=1
            if operand=='0x000981d3':self.narrations+=1
            if operand=='0x000984c0':self.animations.append((self.r['EAX']-0x500000,self.r['EDX']))
            return super().get(operand)
    model=Source();listing=P/'private/ghidra/mission-array-refs/instructions-00013780.txt'
    assert sha(listing)=='634efec82d190cdf793a444a423709cd2e962031a6b2e20ec05c8606bc3a36b4'
    model.code.update(m.m.Model([listing],b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]))
    for address,cleanup in [(0x101d0,0),(0x118e0,0),(0x981d3,0),(0x41ce0,20),(0x41910,16),(0x984c0,0)]:model.code[address]=('RET',[hex(cleanup)]if cleanup else[])
    work=P/'private/review/countdown-transition'/uuid.uuid4().hex;work.mkdir(parents=True);inputs=bytearray();expected=bytearray();instructions=0;cases=0
    for count in [0,1,2,65534,65535]:
      for progress in [-128,0,14,20]:
       for kind in [0,1,2]:
        for flags in [0,64,128,192]:
            counter_byte=[0,1,255][(cases//3)%3];period=[0,1,6000,65535][cases%4]
            pool=bytearray(16*102)
            for i in range(16):struct.pack_into('<I',pool,i*102+0x23,0x11242c)
            if kind:pool[6]=8;pool[0x55]=kind-1
            state={0x16da36+i:b for i,b in enumerate(pool)}
            state.update({0x11c2bc:count&255,0x11c2bd:count>>8,0x11945b:progress&255,0x11937b:counter_byte,0x118770:flags,0x11878c:0,0x185ec1:99,0x11a9da:period&255,0x11a9db:period>>8})
            state.update({0x11680c+i:b for i,b in enumerate(struct.pack('<I',0x500000))})
            model.effects=model.narrations=0;model.animations=[]
            report,_,_=model.run(state_before=state,entry=0x13780,stack_fill=[0,85,255][cases%3])
            started=model.load(0x185ec1,1)==0;expired=model.effects==2
            assert model.animations==[(0x15e,0x184d04)]+[(0x118+i*10,0x184d0c+i*5)for i in range(7)]
            assert model.narrations==int(started or expired)
            inputs+=struct.pack('<HHBBBB',count,period,progress&255,counter_byte,flags,kind)
            expected+=struct.pack('<HBBB',model.load(0x11c2bc,2),started,expired,model.effects+model.narrations+started)
            cases+=1;instructions+=report['steps']
    (work/'input.bin').write_bytes(inputs);exe=work/'probe.exe'
    subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/countdown_transition_probe.rs','-o',str(exe)],cwd=P,env=environment(),capture_output=True,check=True)
    subprocess.run([str(exe),str(work)],check=True);actual=(work/'actual.bin').read_bytes();assert actual==expected,next((i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b)
    receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=len(actual),literal_instructions=instructions,private_run=work.relative_to(P).as_posix(),
        implementation_pins={f:sha(P/f)for f in ['scripts/validate_countdown_transition.py','scripts/countdown_transition_probe.rs','rust/crates/genewars-runtime/src/countdown_expiry.rs']},listing_sha256=sha(listing),original_execution=False,
        scope='All timer branches, active selector0 restart gate, ordered animation descriptor/state pairs and presentation request counts. Factories and callbacks remain request ports; no complete world pass claimed.')
    (P/'research/integration/countdown-transition-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:receipt[k]for k in ['status','cases','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
