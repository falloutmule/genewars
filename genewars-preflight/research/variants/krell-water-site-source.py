from pathlib import Path
import hashlib,json,importlib.util
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m);v=m.Source();v.mem={};v.pc=0
listing=P/'private/ghidra/building-loader-refs/instructions-000507d0.txt';text=listing.read_text()
assert v.load(0x507ac,4)==0x50bd2 and v.load(0x507b0,4)==0x50b4b
for line in ['00050b2a CMP AL,0x2','00050b2e JBE 0x00050ad5','00050ad5 MOV DL,0x2','00050b6f PUSH 0xd','00050bc9 MOV byte ptr [EAX + 0x7a],0x5','00050bcd JMP 0x00051583','00050bf3 PUSH 0xc','00050c4a MOV byte ptr [EAX + 0x7a],0x6','00051583 MOV EDX,0x1','0005158b CALL 0x00051e80']:assert line in text,line
files=[listing,P/'private/ghidra/building-loader-refs/instructions-00051e80.txt',P/'private/ghidra/validated/memory-.object1.bin']
r={'status':'passed','scope':'Static instruction and mapped jump-table binding checks only; not fullcallback/native acceptance.','reference':'QA BETA V5 verified CD','classes':[{'class':1,'name':'Blocked','dispatch':'50BD2','message_decimal':12,'actor7A':6},{'class':2,'name':'Water','dispatch':'50B4B','message_decimal':13,'actor7A':5}],'shared_continuation':'51583 EDX1→51E80 cancellation thenexisting57A10tail','audio':'419E0 sameparametersECX1,priority2,volume127,range100; formattedmessagevaries','rng':'No RNG instructions in eitherfeedbackbranch; cancellationandcallbacktail independentlyretained','source_pins':{str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files},'actual_water_callback':'not_run'}
(P/'research/variants/krell-water-site-source.json').write_text(json.dumps(r,indent=2),encoding='utf-8');print(json.dumps({'status':r['status'],'scope':r['scope']}))
