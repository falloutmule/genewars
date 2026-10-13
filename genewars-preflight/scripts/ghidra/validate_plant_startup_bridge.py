"""Loaded plant state and bounded first scheduled growth/animation updates.

This is not a whole mission tick: other entity updates are explicit exclusions.
"""
from pathlib import Path
import hashlib,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
pins=[]
def pin(name,lines=()):
 b=(G/name).read_bytes()
 for line in lines:assert line in b.decode(),(name,line)
 pins.append({'private_path':'private/ghidra/'+name,'sha256':sha(b)})
pin('entity-order/instructions-0002bbe0.txt',['0002be74 CALL 0x00082bd0','0002be7e CALL 0x000829a0','0002be88 CALL 0x00084310','0002bed3 CALL 0x00086c20'])
pin('module-transition-refs/instructions-0001bbc0.txt',['0001bcd1 CALL dword ptr [EBX]','0001bd36 CALL 0x00070b00','0001bd3b CALL 0x0001c120','0001bd40 CALL 0x0008dfe0'])
pin('terrain-camera-refs/instructions-0007d090.txt',['0007d771 CALL 0x00072ad0'])
pin('camera-player-input/instructions-00072ad0.txt',['00072bcc CALL 0x00075b00','00072bd4 CALL 0x00072960'])
pin('camera-player-input/instructions-00072960.txt',['000729bf CALL 0x0007b240','000729c4 MOV byte ptr [EDX + 0x17],AL'])
pin('mission-tail-init/instructions-0006c690.txt',['0006c695 MOV dword ptr [EAX + 0x27],0x0'])
pin('entity-order-refs/instructions-0006d0b0.txt',['0006d0c8 ADD EDX,0x8','0006d0d7 CMP EAX,0x200','0006d0dc JLE 0x0006d0f4','0006d128 CMP EDI,0x2','0006d130 CMP DX,word ptr [0x0011c2ba]','0006d13f CALL dword ptr [EDX + 0x64]','0006d14f ADD EAX,0x8','0006d195 CALL 0x00083140'])
pin('entity-order/instructions-0006ca90.txt',['0006cb2f MOV byte ptr [EDI + 0x2f],AL','0006cb34 MOV AL,byte ptr [EBX + 0x1b]','0006cbe4 CALL 0x0008b300','0006cbf9 CALL 0x0008b6f0','0006cc11 MOV AL,byte ptr [EBP + 0x1a]','0006cc75 CALL 0x0008b520','0006cc83 MOVSX AX,byte ptr [EAX + EBP*0x1 + 0x10]','0006cc89 ADD word ptr [EDX],AX','0006cc95 MOV word ptr [EDI + 0x2c],0x20','0006ccaa IDIV EBX','0006ccae JNZ 0x0006cfe9','0006cff6 CMP EAX,0x2','0006d007 JLE 0x0006d01d','0006d01a MOV byte ptr [EDI + 0x19],AL'])
pin('plant-startup/instructions-0008b300.txt',['0008b310 CMP byte ptr [EAX + 0x6],0x2'])
pin('plant-startup/instructions-0008b6f0.txt',['0008b708 CMP byte ptr [EDX + 0x6],0x2','0008b713 CALL dword ptr [ECX + 0x68]','0008b726 INC EBX'])
pin('plant-startup/instructions-0001ebd0.txt',['0001ebd0 MOVSX EAX,word ptr [EAX + 0x2c]','0001ebd6 SETLE AL'])
pin('plant-startup/instructions-0008b520.txt',['0008b52f MOV CL,BL','0008b539 INC BH','0008b541 JGE 0x0008b566','0008b56d INC AH','0008b5e1 AND DL,0x7','0008b5ea JNZ 0x0008b6e2','0008b6e0 MOV DL,0x1'])
pin('mission-array-refs/instructions-0006d390.txt',['0006d393 MOVSX EAX,word ptr [EAX]','0006d468 MOV DL,byte ptr [EDX + 0x2]','0006d47c JMP dword ptr CS:[EDX*0x4 + 0x6d370]'])
pin('display-geometry/instructions-0006c780.txt',['0006c79e ADD byte ptr [ESI + 0x19],0x6','0006c7a5 MOV AL,byte ptr [EAX + 0x1d]','0006c7ae OR BL,0x80','0006c7bc CMP EAX,0x4','0006c812 CALL 0x0008acc0','0006c8dd CALL 0x000724c0','0006c8e2 MOV byte ptr [ESI + 0x6],0x0','0006c8e6 DEC word ptr [0x0011c2b2]'])
pin('terrain-map-refs/instructions-0008acc0.txt',['0008ad3e MOV word ptr [EAX],BX','0008ad58 MOV word ptr [EAX + 0x3],DX','0008adf5 MOV word ptr [EBX + 0x6],AX'])
pin('panel-activation-refs/instructions-000724c0.txt',['0007254d MOV byte ptr [EDI + 0x2],0x0','000728aa CALL 0x000409f0','000728ca CALL 0x0002c200'])
pin('plant-startup/instructions-000409f0.txt',['00040a0c CMP EBX,dword ptr [EAX + 0x4]','00040a11 MOV dword ptr [EAX + 0x4],0x0'])
pin('plant-startup/audio-shared-tail.txt',['0004080b POP ECX','0004080c POP EBX','0004080d RET'])
pin('plant-startup/instructions-0002c200.txt',['0002c223 MOV dword ptr [EAX + 0x19f],0x0','0002c249 MOV dword ptr [EAX + 0x1cc],0x0'])
source=(G/'campaign-resources/152.dat').read_bytes();assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
mem=(G/'validated/memory-.object3.bin').read_bytes()
groups=json.loads((P/'research/ghidra/mission-load-evidence.json').read_text())['serialized_groups']
records={}
for group in groups:
 for index in range(group['count']):
  o=group['file_offset']+index*group['packed_stride'];s=source[o:o+group['packed_stride']]
  native=bytearray(group['native_stride']);native[:31]=s[:31];native[39:39+len(s)-31]=s[31:]
  records[(group['family']+1,index)]=native
plants={i:r for (kind,i),r in records.items() if kind==2 and r[6]}
assert len(plants)==40 and list(plants)==list(range(40))
assert all(r[0x15]==4 and r[0x19]==r[0x1b]==r[0x2e]==r[0x2f]==0 for r in plants.values())
assert struct.unpack_from('<h',source,0x3552)[0]==40 and struct.unpack_from('<h',source,0x355a)[0]==0

def reference(data,offset):
 index,kind=struct.unpack_from('<hB',data,offset)
 if index<0 or not 1<=kind<=8:return None
 assert (kind,index) in records
 return kind,index

def cell(x,y):return (((y&127)*128+(x&127))*14)
region=bytearray(source[0x355e:0x3b55e])
plant_links=[];cell_plants={}
for y in range(128):
 for x in range(128):
  ref=reference(region,cell(x,y)+6);seen=set();found=[]
  while ref is not None:
   assert ref not in seen;seen.add(ref);r=records[ref]
   if r[6]==2:
    assert ref[0]==2 and r[6]
    found.append(ref[1]);plant_links.append(ref[1])
   ref=reference(r,3)
  cell_plants[x,y]=found
assert sorted(plant_links)==list(range(40))
for i,r in plants.items():
 x,y=struct.unpack_from('<hh',r,7);assert i in cell_plants[(x>>8)&127,(y>>8)&127]

def terrain_class(position):
 x,y=position;gx,gy=(x>>8)&127,(y>>8)&127
 t=region[cell(gx+(int(x&255>=128)),gy+(int(y&255>=128)))+5]&7
 if t:return t
 return int(any(region[cell(gx+dx,gy+dy)+5]&7 for dy in [-1,0,1] for dx in [-1,0,1]))

def cursor_next(cursor):
 assert 0<=cursor<=512
 n=cursor+8
 return ((n+1)&7) if n>512 else n

def bounded_first_growth(growth,animation,counter,flags15,flags2e,overlay,neighbors,threshold,crowding,period,delta,retirement_counter=3,owner=4):
 assert -32768<=growth<=32767 and 0<=animation<=255 and -128<=delta<=127
 assert counter==0 and not flags15&1 and not flags2e&0x9a
 assert 0<=owner<=4 and 0<=retirement_counter<=255 and 0<=neighbors<=255
 assert growth>0 and overlay>0 and threshold>0 and period>1
 updated=0 if neighbors>crowding else min(((growth+delta+32768)&65535)-32768,32)
 if updated<=0:
  common={'growth_i16':0,'counter_u8':1,'animation_u8':(animation+6)&255,'flags15_u8':flags15|4,'flags2e_u8':flags2e|128,'created_count_u8':0}
  if retirement_counter>=4:return dict(common,phase='delayed-retirement',retirement_counter_u8=retirement_counter-4,active_type_u8=2)
  return dict(common,phase='retire',retirement_counter_u8=retirement_counter,active_type_u8=0,cell13_after=overlay+2 if overlay<64 else overlay,plant_count_delta=-1,owner_count_decrement=owner if owner<4 else None,pending_bookkeeping=['unlink-cell-list','check-old-frame-dirty-in-current-view','724C0-reference/UI/audio-notifications'])
 stage=min(updated>>3,2)
 new_animation=max(animation,stage)
 return {'phase':'grow','growth_i16':updated,'counter_u8':1,'animation_u8':new_animation,'flags15_u8':flags15|(4 if new_animation!=animation else 0),'active_type_u8':2,'created_count_u8':0}

def unlink_source_record(index,gx,gy):
 r=plants[index];prev=reference(r,0);nxt=reference(r,3)
 if nxt is not None:records[nxt][:3]=r[:3]
 if prev is not None:records[prev][3:6]=r[3:6]
 else:region[cell(gx,gy)+6:cell(gx,gy)+9]=r[3:6]
 cell_plants[gx,gy].remove(index)

cursor=cursor_next(0);scheduled=[]
for index,r in plants.items():
 if len(scheduled)<2 and index==cursor:
  scheduled.append(index);cursor=cursor_next(cursor)
assert scheduled==[8,16] and cursor==24
fixtures=[]
for index in scheduled:
 r=plants[index];position=struct.unpack_from('<hh',r,7);gx,gy=[(v>>8)&127 for v in position]
 neighbors=sum(1 for dy in [-1,0,1] for dx in [-1,0,1] for i in cell_plants[(gx+dx)&127,(gy+dy)&127] if struct.unpack_from('<h',plants[i],0x2c)[0]>0)
 selector=struct.unpack_from('<b',r,0x2b)[0];params=mem[0x4b80+selector*32:0x4b80+(selector+1)*32]
 t=terrain_class(position);delta=struct.unpack_from('<b',params,16+t)[0]
 growth=struct.unpack_from('<h',r,0x2c)[0]
 result=bounded_first_growth(growth,r[0x19],r[0x2f],r[0x15],r[0x2e],region[cell(gx,gy)+13],neighbors,params[27],params[26],params[24],delta,params[29],r[24])
 fixtures.append({'record':index,'resource_slot':selector,'growth_before':growth,'terrain_class':t,'growth_delta':delta,'neighbor_positive_growth_count':neighbors,'crowding_threshold':params[26],'period':params[24],'result':result})
 if result['phase']=='retire':
  unlink_source_record(index,gx,gy);region[cell(gx,gy)+13]=result['cell13_after'];r[6]=0;r[0x2e]=result['flags2e_u8'];r[0x30]=result['retirement_counter_u8']
 struct.pack_into('<h',r,0x2c,result['growth_i16']);r[0x2f]=1;r[0x19]=result['animation_u8'];r[0x15]=result['flags15_u8']
private=G/'plant-startup/first-scheduled-fixture.json';payload=(json.dumps(fixtures,separators=(',',':'))+'\n').encode();private.write_bytes(payload)
assert sum(bool(r[6]) for r in plants.values())==38
assert [r['result']['phase'] for r in fixtures]==['retire','retire']
# Check live-linked membership after both original-style predecessor/next rewrites.
post_links=[]
for y in range(128):
 for x in range(128):
  ref=reference(region,cell(x,y)+6);seen=set()
  while ref is not None:
   assert ref not in seen;seen.add(ref);r=records[ref]
   if r[6]==2:post_links.append(ref[1])
   ref=reference(r,3)
assert sorted(post_links)==[i for i in range(40) if i not in scheduled]
vectors=[]
for growth,animation,delta in [(1,0,0),(7,0,0),(8,0,0),(15,0,0),(16,0,0),(32,1,3),(22,0,2),(11,0,3),(5,2,-4)]:
 result=bounded_first_growth(growth,animation,0,4,0,1,1,7,3,4,delta)
 vectors.append({'growth_before':growth,'animation_before':animation,'signed_delta':delta,'result':result})
retirement_vectors=[]
for neighbors,crowding,growth,delta,delay,owner,overlay in [(3,3,1,0,3,4,1),(4,3,1,0,3,4,1),(4,3,22,3,4,4,63),(4,3,22,3,255,4,64),(0,3,1,-1,0,0,63),(0,3,32767,1,3,3,255)]:
 retirement_vectors.append({'neighbors_u8':neighbors,'crowding_u8':crowding,'growth_i16':growth,'delta_i8':delta,'delay_u8':delay,'owner_u8':owner,'cell13_u8':overlay,'result':bounded_first_growth(growth,0,0,4,0,overlay,neighbors,7,crowding,4,delta,delay,owner)})
growth_cases=0
for growth in range(1,32768):
 for delta in [-4,-3,-2,-1,0,1,2,3]:
  result=bounded_first_growth(growth,0,0,4,0,1,0,7,3,4,delta)
  ax=(growth+(delta&65535))&65535;ax=ax-65536 if ax&32768 else ax
  if ax>32:ax=32
  if ax<=0:assert result['phase']=='retire' and result['growth_i16']==0
  else:assert result['phase']=='grow' and result['growth_i16']==ax and result['animation_u8']==min(ax>>3,2)
  growth_cases+=1
cursor_vectors=[{'cursor':i,'next':cursor_next(i)} for i in [0,1,7,8,496,504,505,511,512]]
for cursor_input in range(513):assert 0<=cursor_next(cursor_input)<=512
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,
 'field_layout':{'source_family1_base':'0x43d4e','source_stride':41,'native_stride':49,'fields':{'active_type_u8':[6,6],'flags15_u8':[21,21],'shade_u8':[23,23],'owner_u8':[24,24],'animation_u8':[25,25],'frame_u8':[27,27],'resource_i8':[35,43],'growth_i16':[36,44],'flags2e_u8':[38,46],'update_counter_u8':[39,47],'retirement_counter_u8':[40,48]},'field_pairs':'sourceoffset,nativeoffset; cachedframeptrnative27hex isruntimeonly'},
 'pure_plan_admission':{'growth_i16':'1..32767','animation_u8':'0..255','flags15':'u8,mask01clear','flags2e':'u8,mask9Aclear','counter2f':'exactly0','cell13':'u8,1..255','neighbor_byte_sum':'0..255; originalnine-cellcountaccumulateswithbytewrap','crowding_limit':'u8,0..255','check_threshold1b':'u8,1..255','spread_period18':'u8,2..255','delta':'i8,-128..127','retirement_delay1d':'u8,0..255','owner':'u8,0..4; largerownerssaferejected','frame':'preserved; notadvancedbythisslice'},
 'timeline':'7D090 calls72AD0 (75B00 then72960shade refresh) beforemodule transition.2BBE0 invokes82BD0/829A0/84310 thenlocked86C20 beforethe next1C120 entityupdate.1BBC0latercalls70B00,1C120,8DFE0. Withfresh emptyqueue the moduleinitializationdraw hasno newlyqueuedplants; do notlabelitfirstliveentityframe.',
 'loaded_plant_contract':'40activeCONQ001family1records0..39, native type2, animation0/frame0, flags15=04, flags2E=0,counter2F=0.6C690 setsresource pointer andcachedframe27=NULL, preservingserializedanimation/frame.72960replacesnative shade17frommap light/factor beforemoduleinit.',
 'schedule':'Ordinary6D0B0 startscursor16bitword11C2BA with+8; ifsignedresult>512 then(cursor+1)&7, elsekeepsresult. Atmost2eligible matchingindex slots invokevirtual64, advancingcursoragainperinvoke; stillqueuesallremainingeligibleactive records. Safeinputcursor0..512. Sourcecount40andcursor0yieldrequests8,16; cursorafter=24.',
 'growth_slice':'Closed first-counter update: flags15mask01clear; flags2Emask9Aclear; growthsigned16>0; oldcounter0; celloverlay13>0; parameterthreshold1B>0; periodparameter18>1. Incrementcounterto1. Ifpositive-growth neighborcount>crowdingparameter1A: growth0, enterretirement below. Else addsignedparameter[10+terrainclass] withsigned16wrap/capupper32; nonpositiveoutputalsoretires. Counter1modperiod!=0 bypassesspawn/RNG; positivegrowthanimation=max(oldanimation,min(growth>>3,2)). ORflag04ifanimationchanged, frameunchanged, returncreatedcount0.',
 'retirement_slice':'Withadmittedfreshflags2E mask88clear:6C780 adds6tou8animation, setsgrowth0, flags2E|80 andcounter30=resourcecounter1D. Ifcounter>=4: subtract4andreturnactive. Ifcounter<4: immediatelyunlink; cell13 increasesby2 onlyifold<64 (63becomes65); owner0..3requestsDECwordatplayer+5FA, owner4skips; request724C0bookkeeping, cleartype6anddecrementplantcount. Immediatebranchdoesnotrenderanimation6 becauseactivebytecleared. Requestoldframe-dirtycheckinexplicitcurrentview; originalcallsvirtual70onlywhenprojectedpointinside[0,640-margin)x[0,480), andcachedframe27NULL suppressesitsbody.',
 'bookkeeping':'724C0 clears matching3-byte target-reference typebytes in activeotherfamilies (notplantgrowthfields), mayreactivatepanel ifremovedentityselected, andcalls409F0audio/objectnotification plus2C200UIselectioncleanup.409F0scans9attachmentrecordsstride16hex andclearsmatchingpointer+4;2C200onlyclearsmatchingpanelpointersR+19F/R+1CC. These areexplicitpendingrequests, notsilentlyexecutedbythepurestateplan. Actualworld/selection/audioinputsremaincallerresponsibility.',
 'terrain_class':'8B520 samples cell+5low3 atpositionroundedupindependentlywhenfraction>=128. Ifthatclass0, return1ifany of3x3neighbors aroundunroundedfloorcellhasnonzerolow3, else0.128wrap. This is separatefromtextureUVorientation.',
 'neighbor_contract':'8B300(mask0) establishesatleastone type2linkedentitypercell;8B6F0(mode0) countstype2whosevirtual68 returnsfalse.1EBD0 returnsgrowth<=0, socountpositivegrowth.6D390resolves3-byte signed16index/u8familyreferences. All40sourceplantmaplinksvalidatedexactlyonce andpositioncellconsistent.',
 'source_fixture':{'entry_id':'root/2/disc/152','sha256':sha(source),'active_family_counts':[g['active_raw_byte_nonzero'] for g in groups],'plant_links_before':len(plant_links),'plant_links_after':len(post_links),'scheduled_indices':scheduled,'cursor_after':cursor,'private_path':'private/ghidra/plant-startup/first-scheduled-fixture.json','sha256':sha(payload),'results_metadata':fixtures},
 'cursor_vectors':cursor_vectors,'growth_vectors':vectors,'retirement_vectors':retirement_vectors,'register_growth_cases':growth_cases,
 'capacity':'Loadedsource has1activefamily0 and40activefamily1, others0. Thisboundsisolatedloadedstate41entries, notliveafterplayerentitycreation/otherupdates. Explicitprioracceptedcount<=216 guaranteesroomforall40plants; largerpriorstate mustuseexistingcapacity256queuecontract.',
 'limits':['This validates firstscheduledplant updates againstanexplicitloadedstate, notwhole1C120tick. Earlierfamily0/player updates maymutatestate; keepthat externalstate explicit.','75B00callbacks/actorcreation notexecuted; shade prooftakesfinalmap snapshot asinput.','No timing/frame-rate oruniversalinitialqueue emptinessinferred.','Fullgrowthspawning/longretirement paths excluded; immediateowner4retirementproducesexplicitpendingbookkeeping.']}
(P/'research/ghidra/plant-startup-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS19pins;40->38source-linked plants;513cursor domain;{growth_cases}registergrowthcases;15vectors;scheduled8/16 immediate-retirement requests')
print(json.dumps(fixtures))
