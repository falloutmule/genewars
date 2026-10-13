"""Offline parameter extraction and typed source-state fixture; no original run."""
from pathlib import Path
import hashlib,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra';sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
exe=(G/'GW-CD-candidate.exe').read_bytes();assert sha(exe)==candidate
source=(G/'campaign-resources/152.dat').read_bytes();assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
mapped=(G/'validated/memory-.object3.bin').read_bytes()
params=mapped[0x4b80:0x4b80+12*32]
assert params==exe[0x126980:0x126b00] and sha(params)=='297a1da0e9c30d6d4cb7c88addb8234be260e0e93b7d4b2b1c4c3ad8b7df2b7c'
h=struct.unpack_from('<I',exe,0x3c)[0];u=lambda o:struct.unpack_from('<I',exe,o)[0]
size,base,flags,index,count,_=struct.unpack_from('<6I',exe,h+u(h+0x40)+2*24)
assert base==0x100000
page=index-1+0x4b80//4096;table=h+u(h+0x48);physical=int.from_bytes(exe[table+page*4:table+page*4+3],'big')
assert u(h+0x80)+(physical-1)*4096+0x4b80%4096==0x126980
pins=[]
for name,needles in [
 ('entity-order/instructions-0006ca90.txt',['0006cac8 ADD EBX,0x104b80','0006cc83 MOVSX AX,byte ptr [EAX + EBP*0x1 + 0x10]','0006cca2 MOV BL,byte ptr [EBP + 0x18]']),
 ('mission-tail-init/instructions-0006c690.txt',['0006c69c MOV EDX,dword ptr [EDX*0x4 + 0x1168c4]']),
 ('mission-tail-readers/instructions-0006c950.txt',['0006c98d CALL dword ptr [ESI + 0x60]']),
 ('mission-array-refs/instructions-0006d390.txt',['0006d393 MOVSX EAX,word ptr [EAX]','0006d468 MOV DL,byte ptr [EDX + 0x2]']),
 ('display-geometry/instructions-0006c780.txt',['0006c8bd DEC word ptr [EAX + 0x11936e]','0006c8e6 DEC word ptr [0x0011c2b2]'])]:
 b=(G/name).read_bytes()
 for needle in needles:assert needle in b.decode(),(name,needle)
 pins.append({'private_path':'private/ghidra/'+name,'sha256':sha(b)})
resources=json.loads((P/'research/ghidra/plant-color-evidence.json').read_text())['resources']
parameters=[]
for i in range(12):
 b=params[i*32:(i+1)*32]
 parameters.append({'slot':i,'growth_deltas_i8':list(struct.unpack_from('<8b',b,16)),'spread_period_u8':b[24],'random_walk_steps_u8':b[25],'crowding_limit_u8':b[26],'check_threshold_u8':b[27],'retirement_delay_u8':b[29]})
def ref(b,o):
 index,kind=struct.unpack_from('<hB',b,o)
 return {'index_i16':index,'kind_u8':kind}
plants=[]
for i in range(512):
 s=source[0x43d4e+41*i:0x43d4e+41*(i+1)]
 if not s[6]:continue
 assert s[6]==2
 plants.append({'index':i,'previous':ref(s,0),'next':ref(s,3),'active_type_u8':s[6],'position_i16':list(struct.unpack_from('<hh',s,7)),'flags15_u8':s[21],'source_shade_u8':s[23],'owner_u8':s[24],'animation_u8':s[25],'frame_u8':s[27],'resource_slot_i8':struct.unpack_from('<b',s,35)[0],'growth_i16':struct.unpack_from('<h',s,36)[0],'flags2e_u8':s[38],'update_counter_u8':s[39],'retirement_counter_u8':s[40]})
assert len(plants)==40 and all(p['owner_u8']==4 and p['update_counter_u8']==p['retirement_counter_u8']==0 for p in plants)
heads=[];region=source[0x355e:0x3b55e]
for cell in range(128*128):
 r=ref(region,14*cell+6)
 if r['kind_u8']:
  heads.append({'cell_xy':[cell%128,cell//128],'head':r,'cell13_u8':region[14*cell+13]})
owner_counts=[struct.unpack_from('<H',source,0x14+i*0x70d+0x5fa)[0] for i in range(4)]
assert owner_counts==[0]*4 and struct.unpack_from('<h',source,0x3552)[0]==40
folder=G/'plant-state';folder.mkdir(exist_ok=True)
(folder/'parameter-block.bin').write_bytes(params)
payload=(json.dumps({'parameters':parameters,'plants':plants,'nonempty_cell_heads':heads,'raw_owner_counts_u16':owner_counts,'plant_count_i16':40,'schedule_cursor_i16':0},indent=2)+'\n').encode()
(folder/'initial-conq001-plants.json').write_bytes(payload)
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,
 'parameters':{'source':'candidate executable LE object3 initialized data; independent of ANB image data','mapped_va':'0x104B80','mapped_offset':'0x4B80','file_offset':'0x126980','file_end_exclusive':'0x126B00','count':12,'stride':32,'size':len(params),'sha256':sha(params),'private_path':'private/ghidra/plant-state/parameter-block.bin','runtime':'Offline extract once from pinned candidate, then runtime reads the standalone hash-bound table; executable input is not required at runtime. Preserve unproved bytes opaque.','fields':{'growth_deltas_i8':{'offset':16,'count':8},'spread_period_u8':24,'random_walk_steps_u8':25,'crowding_limit_u8':26,'check_threshold_u8':27,'retirement_delay_u8':29},'bindings':[{'slot':r['slot'],'image_entry_id':r['entry_id'],'image_input_sha256':r['input_sha256'],'parameter_record_sha256':sha(params[r['slot']*32:(r['slot']+1)*32])} for r in resources]},
 'plant_fields':{'file_base':'0x43D4E','count':512,'packed_stride':41,'native_stride':49,'fields_source_native':{'previous_reference':[0,0],'next_reference':[3,3],'active_type_u8':[6,6],'position_x_i16':[7,7],'position_y_i16':[9,9],'flags15_u8':[21,21],'source_shade_u8':[23,23],'owner_u8':[24,24],'animation_u8':[25,25],'frame_u8':[27,27],'selector_i8':[35,43],'growth_i16':[36,44],'flags2e_u8':[38,46],'update_counter_u8':[39,47],'retirement_counter_u8':[40,48]},'reference':'three bytes: signed little-endian i16 index plus u8 kind; kind2 identifies numeric family1 plants. Negative index or kind0 gives null. Validate kind/index bounds before following. Native resource/vtable/cache pointers are runtime-only.'},
 'map_links':{'file_base':'0x355E','size':229376,'dimensions':[128,128],'cell_stride':14,'cell_index':'(y&127)*128+(x&127)','head_reference_offset':6,'cell_counter_u8_offset':13,'position_cell':'((i16_position>>8)&127) per axis','neighbor_count':'Nine wrapped cells; follow all linked kinds safely, count only kind2 with active type2 and signed growth>0, accumulate with u8 wrap. All40plant memberships independently verified in plant-startup proof.'},
 'owner_counts':{'source_file_base':'0x14','player_stride':'0x70D','count':4,'player_relative_word_offset':'0x5FA','initial_raw_words':owner_counts,'retirement':'Owner0..3 requests wrapping DECword; owner4 skips. Live retained/player-updated counts must be caller state, not replaced by these raw zeros.'},
 'plant_count':{'source_word_offset':'0x3552','native_va':'0x11C2B2','type':'signed i16','initial_raw':40},
 'scheduler_cursor':{'source_word_offset':'0x355A','native_va':'0x11C2BA','initial_raw':0},
 'fixture':{'source_entry_id':'root/2/disc/152','source_sha256':sha(source),'private_path':'private/ghidra/plant-state/initial-conq001-plants.json','fixture_sha256':sha(payload),'plants':len(plants),'nonempty_cell_heads':len(heads),'scope':'Typed raw source snapshot before player/other-family/visibility updates; source shade is not final runtime shade.'},
 'limits':['Parameter names and unproved bytes remain opaque; no full public original data dump.','This source fixture cannot establish live prior-family effects, selected-player gates or current shade.','No original execution, implicit user command or reproduction RNG in this extraction.']}
(P/'research/ghidra/plant-state-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS {len(pins)} instruction pins; raw/mapped384byteparameteridentity;12bindings;{len(plants)}typedplants;{len(heads)}nonemptycellheads;owner0..3rawcounts0.')
