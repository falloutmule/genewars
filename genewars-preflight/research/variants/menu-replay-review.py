"""Independent bounded menu-clock/mode-transition audit; no GUI/original execution."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct
ROOT=Path(__file__).resolve().parents[2];DAY=8640000
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def clock(h,m,s,c):
 # BBA2A literal shift/sub/add path rather than multiplication-only formula.
 a=(h<<4)-h;a<<=2;d=m+a;a=(d<<4)-d;d=(a<<2)+s;a=((d<<2)-d)<<3;return ((d+a)<<2)+c
def relative(raw,baseline):return(raw+(DAY if raw<baseline else 0)-baseline)&0xffffffff
def request(now,start,prior):
 elapsed=(now-start)&0xffffffff;quotient=elapsed//100
 return dict(elapsed_u32=elapsed,quotient=quotient,next_module='0x186F2C'if quotient>30 else prior)
def main():
 evidence=ROOT/'research/ghidra/menu-replay-evidence.json';before=pin(evidence);proof=json.loads(evidence.read_bytes());pins=[]
 for source in proof['instruction_contracts']:
  p=pin(ROOT/source['private_path']);assert p['sha256']==source['sha256'];pins.append(p)
 mem=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes()
 for table in proof['mapped_module_tables']:
  o=int(table['va'],16)-0x100000;b=mem[o:o+table['length']];assert hashlib.sha256(b).hexdigest()==table['sha256'];assert list(struct.unpack('<'+'I'*(len(b)//4),b))==[int(v,16)for v in table['values']]
 for movie in proof['movie_paths']:
  o=int(movie['va'],16)-0x100000;b=mem[o:o+movie['length']];assert hashlib.sha256(b).hexdigest()==movie['sha256'];assert b[:-1].decode()==movie['relative_path']and b[-1]==0
 for vector in proof['timer_vectors']:assert request(vector['now_u32'],vector['start_u32'],vector['requested_module'])==vector['result']
 arithmetic=0
 for h in [0,1,12,23]:
  for m in [0,1,59]:
   for s in [0,1,59]:
    for c in [0,1,99]:assert clock(h,m,s,c)==100*(3600*h+60*m+s)+c;arithmetic+=1
 transitions=[]
 for baseline in [0,1,10000,DAY-1]:
  start=relative(DAY-1,baseline);now=relative(0,baseline);transitions.append(dict(case='midnight',baseline=baseline,start=start,now=now,elapsed=request(now,start,None)['elapsed_u32']))
  priorraw=(baseline-1)%DAY;start=relative(priorraw,baseline);now=relative(baseline,baseline);out=request(now,start,'0x18653C');assert start==DAY-1 and now==0 and out['elapsed_u32']==(1-DAY)&0xffffffff and out['next_module']=='0x186F2C';transitions.append(dict(case='daily-baseline-crossing',baseline=baseline,prior_raw=priorraw,start=start,now=now,elapsed=out['elapsed_u32']))
 assert transitions[0]['elapsed']!=1 and all(t['elapsed']==1 for t in transitions[2::2])
 tail=(ROOT/'private/ghidra/font-menu/instructions-0007c870.txt').read_text()
 assert tail.index('0007ca25 MOV dword ptr [0x00116978],EDX')<tail.index('0007ca30 CALL 0x000a4f96')<tail.index('0007ca3d SUB EAX,EDX')<tail.index('0007ca3f XOR EDX,EDX')<tail.index('0007ca41 DIV EBX')<tail.index('0007ca46 JBE 0x0007ca60')<tail.index('0007ca54 MOV dword ptr [0x00116978],ECX')
 assert '[ESI + 0x14]'in tail and not any('[ESI + 0x14]'in line and line.split()[1]=='MOV'and line.index('[ESI + 0x14]')<line.index(',')for line in tail.splitlines())
 zeroed=[(0x19d320,0x19d320+0x1020),(0x19e350,0x19e350+0x1000)]
 preserved=[0x19f354,0x19f358,0x19f35a,0x19d314,0x19d318,0x19d220]
 assert all(not any(lo<=address<hi for lo,hi in zeroed)for address in preserved)
 callback=(ROOT/'private/ghidra/mouse-callback/instructions-000a3524.txt').read_text();assert callback.index('000a36a2 MOV [0x0019d310],AX')<callback.index('000a36b4 MOV word ptr [0x0019f358],SI');assert callback.index('000a36ae MOV [0x0019d312],AX')<callback.index('000a36bb MOV word ptr [0x0019f35a],DI')
 extra=[pin(ROOT/'private/ghidra/mouse-motion'/name)for name in ['instructions-000a340c.txt','instructions-000a3884.txt']]
 assert pin(evidence)==before
 report=dict(status='passed-independent-menu-replay-review',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,gui_execution=False,evidence=before,instruction_pins=pins,additional_bounds_pins=extra,formal_timer_vectors=len(proof['timer_vectors']),clock_arithmetic_vectors=arithmetic,clock_phase_vectors=transitions,threshold_elapsed_centiseconds=3100,mouse_saved_addresses_outside_zeroed_buffers=[hex(v)for v in preserved],source=pin(Path(__file__)),findings=['No mismatch in3100centisecond unsigned threshold, sharedtail actionoverride or explicitly gated normal mode reinitialization','BBA90 dailyphase discontinuity occurs on raw clock crossing fixedbaseline; atmidnight subtraction is continuous onlywhen baseline isnotzero','Old19F358/35A are copied to temporary19D310/312 before currentSI/DIoverwrite; savedcounter/remainders surviveknownteardownbufferzeroes'],limits=['3100 logicalclock centiseconds is not a guaranteed31.00 hostwallclock delay; handler cadence/DOSclock granularity external','No measured run006 clock/start/action state; replay cannot establish failed or successfulclick','Successful640x480restore required for center320x240 andsensitivity1/1; activeinitreturnsfailureandregisteredcallbackskip assumesdriverpersistence','No new original run, emulator,GUI or mouse-driver simulation'])
 out=Path(__file__).with_suffix('.json');temp=out.with_suffix('.json.tmp');temp.write_text(json.dumps(report,indent=2)+'\n');temp.replace(out);print(json.dumps(dict(status=report['status'],pins=len(pins),formal_timer_vectors=len(proof['timer_vectors']),clock_arithmetic_vectors=arithmetic)))
if __name__=='__main__':main()
