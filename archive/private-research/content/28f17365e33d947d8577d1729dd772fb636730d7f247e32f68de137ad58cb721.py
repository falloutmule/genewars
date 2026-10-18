from pathlib import Path
import struct,json,hashlib,importlib.util,re
P=Path(__file__).resolve().parents[2]
D=P/'private/review/bootstrap-context/fbc3cdc5f63a4ebfa01d0e78c2cf5354/seed65535-players1-timer1000/frame14111/boundary'
W=P/'private/review/actor-idle-relocation';W.mkdir(exist_ok=True)
spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
backing=(D/'backing.bin').read_bytes()
state={0x118d60+i:b for i,b in enumerate(backing[:0x355e])}
state.update({0x11c2be+i:b for i,b in enumerate(backing[0x355e:0x3b55e])})
for name,count,width,address,vtable in [('actors',20,200,0x16360e,0x112194),('creatures',300,124,0x1542be,0x111fec),('effects',200,81,0x1688ae,0x112254),('family8',16,102,0x16da36,0x11242c)]:
 raw=(D/(name+'.bin')).read_bytes()
 for slot in range(count):
  native=bytearray(raw[slot*width:(slot+1)*width]);struct.pack_into('<I',native,35,vtable)
  if name=='actors':struct.pack_into('<I',native,192,0x118d74+native[24]*1805+0x16+native[0x74]*36)
  state.update({address+slot*width+i:b for i,b in enumerate(native)})
for slot in range(512):
 raw=backing[277838+slot*41:277838+(slot+1)*41];native=bytearray(raw[:31]+bytes(8)+raw[31:]);struct.pack_into('<I',native,35,0x1120ac);state.update({0x15d40e+slot*49+i:b for i,b in enumerate(native)})
state.update({0x118d74+i:b for i,b in enumerate((D/'players.bin').read_bytes())})
state.update({0x1645ae+i:0 for i in range(256*67)})
raw=(D/'rng.bin').read_bytes();seed,ring,draws=struct.unpack('<HBI',raw)
state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))})
state[0x118968]=0;state[0x118969]=0;state[0x180d80]=0
base=0x16360e+200
bp=struct.unpack('<I',bytes(state[base+192+i]for i in range(4)))[0]
style,category=state[bp+0x15],state[bp+0x16]
resources=re.findall(r'actor_spec\((\d+),(\d+),\d+,"[^"]+","[^"]+","([0-9a-f]{64})"', (P/'rust/crates/genewars-runtime/src/actor_resources.rs').read_text())
anb_hash=next(h for s,c,h in resources if int(s)==style and int(c)==category)
anb=bytearray((P/f'private/integration/sprites/{anb_hash}.anb').read_bytes());count,table,components,frames,animations=struct.unpack_from('<H4I',anb)
for i in range(count):
 for offset in [0,4]:struct.pack_into('<I',anb,animations+i*10+offset,0x420000+frames+struct.unpack_from('<I',anb,animations+i*10+offset)[0])
state.update({0x420000+i:b for i,b in enumerate(anb)});state.update({base+31+i:b for i,b in enumerate(struct.pack('<I',0x420000+animations))})
files=[P/'private/ghidra'/f for f in ['terrain-map-refs/instructions-000305f0.txt','initial-creature/instructions-0006d230.txt','construction-command/instructions-00052280.txt','plant-command/instructions-00074a50.txt','actor-motion/instructions-00057a10.txt','actor-motion/instructions-000526c0.txt','actor-motion/instructions-0006f760.txt','construction-command/instructions-0001ee40.txt','actor-motion/instructions-0001ee30.txt','entity-order/instructions-0001f440.txt']]
for kind,entry in [('motion',0x6fb60),('callback',0x57a10)]:
 model=m.Source();model.code.update(m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));
 # Raw66A3 is a WORD seed store despite unsized disassembler text.
 model.code[0x8b542]=model.code.get(0x8b542,('NOP',[]))
 report,_,trace=model.run(counter=14111,seed=seed,state_before=state,entry=entry,registers={'EAX':base})
 after=bytes(model.load(base+i,1)for i in range(200));(W/f'source-{kind}.bin').write_bytes(after)
 result={'status':'passed-source-execution','entry':hex(entry),'fixture_sha256':{f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in D.glob('*.bin')},'report':report,'calls':[hex(x)for x in model.calls],'after_sha256':hashlib.sha256(after).hexdigest(),'source_pins':{f.relative_to(P).as_posix():hashlib.sha256(f.read_bytes()).hexdigest()for f in model.files+files}}
 (W/f'source-{kind}.json').write_text(json.dumps(result,indent=2)+'\n');print(kind,report,flush=True)
