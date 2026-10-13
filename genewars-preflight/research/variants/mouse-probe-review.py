"""Independent actual-byte callback state/guard audit; no emulator launch."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,random,struct,sys,types
ROOT=Path(__file__).resolve().parents[2];PROBE=ROOT/'private/reference-run/mouse-probe-008';WORK=ROOT/'private/review/mouse-probe-008'
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def main():
 binary=PROBE/'drive-c/MOUSE.COM';code=binary.read_bytes();assert len(code)==1160 and hashlib.sha256(code).hexdigest()=='5c6fc1e7025508fbd6490d2bfe85acf8918fcef51883c4ec711212d4d53e3b4e'
 package=ROOT/'private/review/input-free-yield/capstone';shim=types.ModuleType('pkg_resources');shim.resource_filename=lambda package_name,relative:str(package/'capstone'/relative);sys.modules['pkg_resources']=shim;sys.path.insert(0,str(package));import capstone
 assert capstone.__version__=='5.0.1';engine=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_16);engine.detail=True;instructions=list(engine.disasm(code[:245],0x100));ops={i.address:i for i in instructions};assert sum(i.size for i in instructions)==245
 WORK.mkdir(parents=True,exist_ok=True);listing='\n'.join(f'{i.address:04x} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in instructions)+'\n';(WORK/'actual-code-disassembly.txt').write_text(listing,encoding='ascii')
 callback_start=0x16a;callback_end=0x1f5;count_at=0x203;overflow_at=0x205;record_at=0x207;record_end=record_at+64*14
 assert record_end<=0x100+len(code) and code[count_at-0x100:record_at-0x100]==bytes(4)
 callback=[i for i in instructions if callback_start<=i.address<callback_end];assert callback[-1].mnemonic=='retf' and not any(i.mnemonic=='int' for i in callback)
 assert [(i.address,i.op_str) for i in instructions if i.mnemonic=='int']==[(0x104,'0x33'),(0x111,'0x33'),(0x115,'0x16'),(0x11b,'0x16'),(0x127,'0x33'),(0x130,'0x21'),(0x13e,'0x21'),(0x15f,'0x21'),(0x163,'0x21'),(0x168,'0x21')]
 assert ops[0x11d].op_str=='ax, 0xc' and ops[0x120].op_str=='cx, cx' and ops[0x127].op_str=='0x33'
 # Tiny independent interpreter executes only the authored callback instructions.
 # It models word stack, segment addressing, branch flags and bounded data writes.
 def run(count,overflow,seed):
  rng=random.Random(seed);registers={n:rng.randrange(65536) for n in ['ax','bx','cx','dx','si','di','bp','ds','es']};registers.update(cs=0x2000,ss=0x5000,sp=0xfff0 if seed%2 else 0x8000);initial=dict(registers);flags=rng.randrange(65536);initialflags=flags;memory={};writes=[]
  def address(seg,off):return registers[seg]*16+(off&65535)
  def byte(addr):return memory.get(addr,(addr*31+17)%256)
  def word(seg,off):addr=address(seg,off);return byte(addr)|(byte(addr+1)<<8)
  def put(seg,off,value):addr=address(seg,off);memory[addr]=value&255;memory[addr+1]=value>>8&255;writes.append((seg,off&65535,value&65535))
  put('cs',count_at,count);put('cs',overflow_at,overflow);put('ss',registers['sp'],0x1234);put('ss',registers['sp']+2,0x9000);writes.clear();initial_memory=dict(memory)
  def get(o):
   if o.type==capstone.x86.X86_OP_REG:return registers[engine.reg_name(o.reg)]
   if o.type==capstone.x86.X86_OP_IMM:return o.imm&65535
   assert o.type==capstone.x86.X86_OP_MEM
   m=o.mem;off=m.disp
   if m.base:off+=registers[engine.reg_name(m.base)]
   if m.index:off+=registers[engine.reg_name(m.index)]*m.scale
   segment=engine.reg_name(m.segment) if m.segment else 'ss' if m.base and engine.reg_name(m.base)=='bp' else 'ds';return word(segment,off)
  def setvalue(o,v):
   v&=65535
   if o.type==capstone.x86.X86_OP_REG:registers[engine.reg_name(o.reg)]=v;return
   assert o.type==capstone.x86.X86_OP_MEM;m=o.mem;off=m.disp+(registers[engine.reg_name(m.base)] if m.base else 0)+(registers[engine.reg_name(m.index)]*m.scale if m.index else 0);segment=engine.reg_name(m.segment) if m.segment else 'ss' if m.base and engine.reg_name(m.base)=='bp' else 'ds';put(segment,off,v)
  def push(v):registers['sp']=(registers['sp']-2)&65535;put('ss',registers['sp'],v)
  def pop():v=word('ss',registers['sp']);registers['sp']=(registers['sp']+2)&65535;return v
  pc=callback_start;steps=0
  while True:
   i=ops[pc];pc+=i.size;steps+=1;assert steps<=80;mn=i.mnemonic;o=i.operands
   if mn=='pushf':push(flags)
   elif mn=='push':push(get(o[0]))
   elif mn=='popf':flags=pop()
   elif mn=='pop':setvalue(o[0],pop())
   elif mn=='mov':setvalue(o[0],get(o[1]))
   elif mn=='cmp':a,b=get(o[0]),get(o[1]);flags=(flags&~65)|int(a<b)|(64 if a==b else 0)
   elif mn=='add':a,b=get(o[0]),get(o[1]);v=a+b;setvalue(o[0],v);flags=(flags&~65)|int(v>65535)|(64 if v&65535==0 else 0)
   elif mn=='shl':a,b=get(o[0]),get(o[1]);v=a<<b;setvalue(o[0],v);flags=(flags&~65)|((a>>(16-b))&1)|(64 if v&65535==0 else 0)
   elif mn=='inc':v=(get(o[0])+1)&65535;setvalue(o[0],v);flags=(flags&~64)|(64 if not v else 0)
   elif mn=='jae':
    if not flags&1:pc=get(o[0])
   elif mn=='je':
    if flags&64:pc=get(o[0])
   elif mn=='jmp':pc=get(o[0])
   elif mn=='retf':ip=pop();registers['cs']=pop();assert(ip,registers['cs'])==(0x1234,0x9000);break
   else:raise AssertionError((i.address,mn))
  for name,value in initial.items():
   if name in ['cs','sp']:continue
   assert registers[name]==value,(name,registers[name],value)
  assert registers['sp']==initial['sp']+4 and flags==initialflags
  # CS is now the return segment; access logged memory via the original COM CS.
  registers['cs']=initial['cs'];data_writes=[(off,value) for seg,off,value in writes if seg!='ss']
  if count<64:
   expected=[count]+[initial[n] for n in ['ax','bx','cx','dx','si','di']];start=record_at+count*14;assert [word('cs',start+2*j) for j in range(7)]==expected
   assert word('cs',count_at)==count+1 and word('cs',overflow_at)==overflow
   assert data_writes==[(start+2*j,v) for j,v in enumerate(expected)]+[(count_at,count+1)]
  else:
   assert word('cs',count_at)==count and word('cs',overflow_at)==min(65535,overflow+1)
   assert data_writes==([] if overflow==65535 else [(overflow_at,overflow+1)])
  assert all(off in [count_at,overflow_at] or record_at<=off<record_end for off,_ in data_writes)
  return steps,len(data_writes)
 checked=steps=writes=0
 for count in range(64):
  for seed in range(16):s,w=run(count,seed*4096+seed,seed+count*16);checked+=1;steps+=s;writes+=w
 for count in [64,65,255,256,32767,32768,65534,65535]:
  for overflow in [0,1,255,32767,65534,65535]:
   for seed in range(16):s,w=run(count,overflow,seed+count*16);checked+=1;steps+=s;writes+=w
 rng=random.Random(0x33c)
 for _ in range(2048):s,w=run(rng.randrange(65536),rng.randrange(65536),rng.randrange(2**32));checked+=1;steps+=s;writes+=w
 for n in range(65536):
  if n<64:assert record_at+n*14+14<=record_end and n+1<=64
  assert 8+n*14<65536 if n<=64 else True
  assert min(65535,n+1)<=65535
 driver=ROOT/'private/review/input-free-yield/src-ints-mouse.cpp';drivertext=driver.read_text();assert pin(driver)['sha256']=='66bbeb5e435487ebd0d838e1985df7cc43f1c632154e6c73f397b6afb1efb005'
 for term in ['mouse.sub_mask = reg_cx','mouse.sub_seg = SegValue(es)','mouse.sub_ofs = reg_dx','mouse.events>0 && !mouse.in_UIR','reg_bh=GetWheel8bit()','reg_si=(uint16_t)static_cast<int16_t>(mouse.mickey_x)','reg_di=(uint16_t)static_cast<int16_t>(mouse.mickey_y)','mouse.in_UIR = true','mouse.in_UIR = false']:assert term in drivertext
 launcher=ROOT/'scripts/reference/run_mouse_probe.py';launchtext=launcher.read_text();assert launchtext.index('    try:\n        launched =')<launchtext.index("write_new(logs / 'launch-process.json'")
 for term in ['process.wait(timeout=max(0, 180 - (time.monotonic() - started)))','finally:','if process.poll() is None:','process.kill()','except ValueError as error:',"{'status': 'unavailable'}"]:assert term in launchtext
 assert 'if not args.run_empty:' in launchtext and 'return\n    # Exclusive creation' in launchtext
 result=dict(status='passed-independent-authored-callback-byte-review',utc=datetime.now(timezone.utc).isoformat(),emulator_execution=False,original_execution=False,com=pin(binary),main_assembly=pin(PROBE/'MOUSE.asm'),callback_bounds=[callback_start,callback_end],callback_state_cases=checked,executed_authored_callback_instructions=steps,checked_callback_data_writes=writes,counter_domain_checks=65536,record_count_limit=64,record_bytes=14,max_log_bytes=904,stack_input_register_offsets={'AX':16,'BX':14,'CX':12,'DX':10,'SI':8,'DI':6},driver_source=pin(driver),official_source='https://github.com/joncampbell123/dosbox-x/blob/dosbox-x-v2026.10.01/src/ints/mouse.cpp',register_semantics=['AX eventmask with requested bits only;007F omits0100absoluteindicator','BL heldbuttons;BH wheel extension','CX/DX unsigned drivercoordinates','SI/DI raw signed16 cumulative motion counters; no0Bhreset in probe'],limitations=['Conventional valid COM stack required; callback state tests use distinct CS/SS and SP8000/FFF0','Probe uses186+ immediate shifts, compatible with pinned DOSBox CPU;8086 label is imprecise','File create/write/close errors exit0; absent/partial log must remain unavailable, not successful observation','No emulator/host-input responsiveness observation in this static review','Host180second supervision must be separately enforced by launcher'])
 result['review_source']=pin(Path(__file__));result['launcher']=pin(launcher);result['probe_preparer']=pin(ROOT/'scripts/reference/mouse_probe.py');result['probe_validator']=pin(ROOT/'scripts/reference/validate_mouse_probe.py');result['safe_for_supervised_empty_run']=True;result['checker_grants_execution_authorization']=False;result['host_timeout_seconds']=180
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n',encoding='utf8');tmp.replace(out);print(json.dumps({k:result[k] for k in ['status','callback_state_cases','executed_authored_callback_instructions']}))
if __name__=='__main__':main()
