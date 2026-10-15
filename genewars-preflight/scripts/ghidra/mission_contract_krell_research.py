"""Read-only pins for the already-satisfied ordinary Krell Mule research gate."""
from pathlib import Path
import hashlib, json, struct, importlib.util
P = Path(__file__).resolve().parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
fixture = P/'private/review/krell-mule-callbacks-025/frame-660-slot-0-before/players.bin'
b = fixture.read_bytes()
assert struct.unpack_from('<I', b, 0x62a)[0] == 0x11800
assert b[0x6ae] == 4 and b[0x6e0] == 1
paths = ['private/ghidra/campaign-start/instructions-00080460.txt',
         'private/ghidra/camera-player-input/instructions-00080590.txt',
         'private/ghidra/building-cost-refs/instructions-00021e20.txt',
         'private/ghidra/plant-command/instructions-00021a10.txt',
         'private/ghidra/entity-order/instructions-0002bbe0.txt',
         'private/ghidra/font-menu-bind/instructions-000a8320.txt',
         'private/ghidra/descriptor-loader/instructions-0009a09c.txt',
         'private/ghidra/sprite-functions/instructions-00098a60.txt',
         'private/ghidra/ui-controls/instructions-000a86d0.txt',
         'private/ghidra/sprite-scaled/instructions-00098b60.txt',
         'private/ghidra/validated/memory-.object3.bin']
obj=(P/paths[-1]).read_bytes()
assert obj[0x5698:0x56aa]==b'data/genepanl.anb\0'
assert struct.unpack_from('<I',obj,0x5698+28)[0]==0x185dfc+0xeb
assert struct.unpack_from('<I',obj,0x6dbc+18)[0]==6
asset=P/'private/integration/rnc-decoded/83e7010ccc0631db4937ab96395d058f8931458124eb004a7fa91d1c01c3648e.bin'
anb=asset.read_bytes(); table=struct.unpack_from('<I',anb,2)[0]
off,w,h=struct.unpack_from('<IBB',anb,table+6)
assert (w,h)==(29,29)
mask=bytearray(w*h); pos=18+off
for y in range(h):
 x=0
 while True:
  n=struct.unpack_from('<b',anb,pos)[0];pos+=1
  if n==0:break
  if n<0:x-=n
  else:
   assert x+n<=w
   mask[y*w+x:y*w+x+n]=bytes([1])*n;x+=n;pos+=n
 assert x<=w
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py')
model=importlib.util.module_from_spec(spec);spec.loader.exec_module(model)
vm=model.Source(); vm.code.update(model.m.Model([P/paths[-2]],b'').code)
order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
state={0x300000+i:v for i,v in enumerate(anb)}
state.update({0x400000+i:v for i,v in enumerate(struct.pack('<IBB',0x300000+18+off,w,h))})
hit_cases=[]
for x,y in [(-1,14),(0,0),(14,14),(28,28),(29,14),(14,29),(14,0),(0,14)]:
 vm.run(state_before=state,entry=0x98b60,registers={'EAX':0x400000,'EDX':x&0xffffffff,'EBX':y&0xffffffff})
 expected=bool(mask[y*w+x]) if 0<=x<w and 0<=y<h else False
 assert bool(vm.r['EAX'])==expected
 hit_cases.append({'local':[x,y],'hit':expected})
result = dict(status='passed', reference_sha256='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
              source_pins={s: sha(P/s) for s in paths},
              observation={'fixture': fixture.relative_to(P).as_posix(), 'sha256':sha(fixture),
                           'player':0, 'research_entry':4, 'research_value':71680,
                           'species_list_count':1, 'species_list':[4]},
              constructor={'function':'80460', 'human_research_write':'80546', 'value':71680,
                           'list_append':'804C1..80505'},
              carry={'function':'80590', 'range_start':1562, 'byte_count':221},
              conclusion='No new Geneticist research action is required to initiate Mule production on this reference route.',
              production={'species_list_index':0, 'species':4, 'minimum_funds':75,
                          'opcode':12, 'words':['species=4','selected building slot','start=1'],
                          'research_gate':'nonzero'},
              ui_correction={'control':'106DBC', 'descriptor_byte_offset':6, 'descriptor_index':1,
                             'resource_binding':'2BD08..2BDA4, receiver185DFC+EB',
                             'width_height_loader':'A8320 bit20; A8376..A838A'},
              authenticity_scope='Source constructor/carry/gate plus current native fixture observation; not a new native gameplay acceptance.',
              ui_resource={'entry_id':'root/2/disc/85','name':'GENEPANL.ANB',
                           'input_sha256':'83e7010ccc0631db4937ab96395d058f8931458124eb004a7fa91d1c01c3648e',
                           'decoded_sha256':sha(asset),'descriptor_index':1,'width':w,'height':h,
                           'bounds_half_open':[26,344,55,373],'mask_sha256':hashlib.sha256(mask).hexdigest(),
                           'opaque_pixels':sum(mask),'literal_hit_cases':hit_cases,
                           'owned_panel_enable':'21E96..21F0A: flags1 set, flags40 clear, owner equals selected'},
              remaining=['Native production input acceptance; preserve original input/capture dispatch'],
              original_execution=False)
(P/'research/campaign/krell-mule-research-command.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps({'status':'passed','research_value':71680,'species_list':[4]}))

