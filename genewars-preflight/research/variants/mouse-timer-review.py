"""Independent 011 timer/control-flow and callback-relocation byte audit."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,sys,types
ROOT=Path(__file__).resolve().parents[2]
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def main():
 p=ROOT/'private/reference-run/mouse-probe-011';binary=p/'drive-c/MOUSE.COM';code=binary.read_bytes();assert len(code)==1234 and hashlib.sha256(code).hexdigest()=='7e0f4eca31e358b25a5aad959ce9eeec3ecd7c7e876ac78bf28999bdc1ce5b3b'
 cp=ROOT/'private/review/input-free-yield/capstone';shim=types.ModuleType('pkg_resources');shim.resource_filename=lambda a,b:str(cp/'capstone'/b);sys.modules['pkg_resources']=shim;sys.path.insert(0,str(cp));import capstone
 engine=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_16);engine.detail=True
 old=(ROOT/'private/reference-run/mouse-probe-008/drive-c/MOUSE.COM').read_bytes();previous=list(engine.disasm(old[:245],0x100));current=list(engine.disasm(code[:306],0x100));ops={i.address:i for i in current}
 old_callback=[i for i in previous if 0x16a<=i.address<0x1f5];new_callback=[i for i in current if 0x1a7<=i.address<0x232]
 def normalize(instruction,new=False):
  operands=[]
  for o in instruction.operands:
   if o.type==capstone.x86.X86_OP_REG:operands.append(('reg',engine.reg_name(o.reg),o.size))
   elif o.type==capstone.x86.X86_OP_MEM:
    m=o.mem;disp=m.disp
    if new and not m.base and not m.index and disp in [0x249,0x24b]:disp-=70
    operands.append(('mem',m.segment,m.base,m.index,m.scale,disp,o.size))
   elif o.type==capstone.x86.X86_OP_IMM:
    value=o.imm
    if new and instruction.mnemonic.startswith('j'):value-=61
    elif new and instruction.address==0x1d0:value-=70
    operands.append(('imm',value,o.size))
   else:raise AssertionError(o.type)
  return instruction.mnemonic,operands
 assert [normalize(i) for i in old_callback]==[normalize(i,True) for i in new_callback]
 expected={0x115:('int','0x1a'),0x117:('mov','word ptr [0x5cd], dx'),0x11f:('je','0x12c'),0x121:('mov','byte ptr [0x5cf], 1'),0x12a:('jmp','0x13f'),0x12e:('int','0x1a'),0x130:('sub','dx, word ptr [0x5cd]'),0x134:('cmp','dx, 0x444'),0x138:('jb','0x11b'),0x13a:('mov','byte ptr [0x5cf], 2'),0x13f:('mov','ax, 0xc'),0x142:('xor','cx, cx'),0x149:('int','0x33'),0x185:('int','0x21'),0x18b:('mov','dx, 0x23c'),0x194:('mov','cx, 1'),0x197:('mov','dx, 0x5cf'),0x19c:('int','0x21')}
 for address,v in expected.items():assert(ops[address].mnemonic,ops[address].op_str)==v,address
 assert code[0x232-0x100:0x23c-0x100]==b'MOUSE.LOG\0' and code[0x23c-0x100:0x245-0x100]==b'STOP.BIN\0'
 assert 0x24d+64*14==0x5cd and code[0x5cd-0x100:0x5d0-0x100]==bytes(3)
 int21=[i.address for i in current if i.mnemonic=='int' and i.op_str=='0x21'];assert min(int21)>0x149 and len(int21)==8
 cases=0
 for start in range(65536):
  for elapsed in [0,1,1090,1091,1092,1093,32767,32768,65534,65535]:
   now=(start+elapsed)%65536;observed=(now-start)%65536;assert(observed<0x444)==(elapsed<1092);cases+=1
 preparer=ROOT/'scripts/reference/mouse_probe.py';launcher=ROOT/'scripts/reference/run_mouse_probe.py';launchtext=launcher.read_text()
 for term in ['--timed','prepare_timed()','startup.wShowWindow = 1','process.wait(timeout=max(0, 180 - (time.monotonic() - started)))','finally:','process.kill()']:assert term in launchtext
 config=p/'dosbox-x.conf';assert pin(config)['sha256']=='506802e42d08c46d77f176efcd0efab03cbf57e2b8fbe15f8242d0b9284a285b'
 report=dict(status='passed-independent-timed-empty-probe-011-review',utc=datetime.now(timezone.utc).isoformat(),emulator_execution=False,original_execution=False,com=pin(binary),config=pin(config),emitter=pin(preparer),launcher=pin(launcher),timer_ticks=1092,ordinary_modulo_tick_cases=cases,callback_instructions_isomorphic=len(new_callback),callback_code_delta=61,callback_counter_data_delta=70,records_extent=[0x24d,0x5cd],timer_start_word=0x5cd,stop_reason_byte=0x5cf,max_log_bytes=904,host_timeout_seconds=180,safe_for_supervised_empty_run=True,checker_grants_execution_authorization=False,limits=['Tickinterval~59.98s only under normal18.2065Hz clock advancement','BIOSmidnightreset or clockjump can end early; frozen/slow guestclock remainsbounded byhost180s','MOUSE.LOG/STOP.BIN errors remain unavailable; no runtime/input result asserted','011callback matches008up to relocation; 008independent register/guard review supplies statecoverage'])
 report['review_source']=pin(Path(__file__));out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:report[k] for k in ['status','ordinary_modulo_tick_cases','callback_instructions_isomorphic']}))
if __name__=='__main__':main()
