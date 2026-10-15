"""Source factory -> literal owned move -> first callback; no native acceptance."""
from pathlib import Path
import json,hashlib,struct,uuid,pickle,importlib.util
P=Path(__file__).resolve().parents[2];f=P/'scripts/ghidra/genepod_mule_factory_contract.py';ns={'__file__':str(f)}
# Reuse exact existing factory preparation, restricting its matrix without changing factory code.
text=f.read_text().split("out={'status'")[0].replace('for seed in [0,5428,10548,65535]:','for seed in [5428]:').replace('for fill in [0,165]:','for fill in [0]:');exec(compile(text,str(f),'exec'),ns)
vm=ns['vm'];m=ns['m'];state=vm.saved_state.copy();put=ns['put'];w=P/'private/review/krell-owned-mule-command'/uuid.uuid4().hex;w.mkdir(parents=True)
files=[P/'private/ghidra'/a for a in ['terrain-panel-refs/instructions-0005df50.txt','mission-array-refs/instructions-0005cc60.txt','activation-rng-refs/instructions-0005c920.txt','mission-array-refs/instructions-00081c60.txt']]
vm.code.update(m.m.Model(files,b'').code);vm.code[0x41910]=('RET',['0x10']);vm.code[0x5e068]=('JMP',['0x5e0a3']);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
def record():return bytes(vm.load(0x1542be+i,1)for i in range(124))
def rng():return[vm.load(0x11878a,2),vm.load(0x11872c,1),vm.load(0x118782,4)]
before=record();seed=rng()[0];put(state,0x440000,2,0);put(state,0x440002,2,1536);put(state,0x440004,2,1536)
r,_,trace=vm.run(entry=0x81c60,state_before=state,seed=seed,registers={'EAX':0x118d74,'EDX':0x440000});after=record();command_rng=rng();(w/'factory.bin').write_bytes(before);(w/'command.bin').write_bytes(after);(w/'command-memory.pickle').write_bytes(pickle.dumps(vm.saved_state));assert after[0x5e]==0 and after[0x78]==7 and after[0x60]&0x18==0x18 and after[0x61]&0xc0==0x80
result={'status':'source_prepared','scope':'Existing synthetic empty-flat nullparent owner0 factory then real opcode11 handler. Caller ejection overwrite not included. No production comparison.','factory_seed':5428,'command_rng':command_rng,'command_instructions':r['steps'],'presentation_port':'41910 audio request bounded RET16 and5E068 selected-panel refresh skipped perexistingcallbackharness; no gameplay/RNG fabricated','changes':[{'offset':hex(i),'before':a,'after':b}for i,(a,b)in enumerate(zip(before,after))if a!=b],'private_run':w.relative_to(P).as_posix(),'source_pins':{s.relative_to(P).as_posix():hashlib.sha256(s.read_bytes()).hexdigest()for s in list(dict.fromkeys(vm.files+ns['files']+files))}}
# The real Mule animation binding is needed by the callback (factory fixture itself used a typed address).
anb=bytearray((P/'private/integration/game-runtime/creatures/MULE.ANB').read_bytes());count,table,components,frames,animations=struct.unpack_from('<H4I',anb)
for i in range(count):
 for offset in [0,4]:struct.pack_into('<I',anb,animations+i*10+offset,0x400000+frames+struct.unpack_from('<I',anb,animations+i*10+offset)[0])
state=vm.saved_state.copy();state.update({0x400000+i:b for i,b in enumerate(anb)});put(state,0x1542be+31,4,0x400000+animations)
try:
 r,_,trace=vm.run(entry=0x5ed20,counter=1,state_before=state,seed=command_rng[0],registers={'EAX':0x1542be});(w/'callback.bin').write_bytes(record());result['callback']={'status':'source_prepared','instructions':r['steps'],'rng_after':rng(),'state':record()[0x5e],'calls':r['calls'],'flags60word':int.from_bytes(record()[0x60:0x62],'little'),'command78':record()[0x78],'changes':[{'offset':hex(i),'before':a,'after':b}for i,(a,b)in enumerate(zip(after,record()))if a!=b and not 31<=i<39]};(w/'callback-trace.json').write_text(json.dumps([hex(pc)for pc in trace]));(w/'callback-memory.pickle').write_bytes(pickle.dumps(vm.saved_state))
except Exception as e:result['callback']={'status':'blocked','reason':repr(e),'pc':hex(vm.pc),'state':record()[0x5e]};(w/'callback-boundary-memory.pickle').write_bytes(pickle.dumps(vm.mem))
result['fixture_pins']={q.name:hashlib.sha256(q.read_bytes()).hexdigest() for q in w.glob('*.bin')};result['validator_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest();result['animation_resource_sha256']=hashlib.sha256((P/'private/integration/game-runtime/creatures/MULE.ANB').read_bytes()).hexdigest()
(P/'research/campaign/krell-owned-mule-command-probe.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
