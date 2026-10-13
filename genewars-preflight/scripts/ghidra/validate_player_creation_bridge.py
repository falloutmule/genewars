"""Bounded player blueprint selection/allocation, not an AI tick or live scene."""
from pathlib import Path
import hashlib, itertools, json, struct

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
sha = lambda b: hashlib.sha256(b).hexdigest()
candidate = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G / 'GW-CD-candidate.exe').read_bytes()) == candidate
pins = []

def pin(name, *needles):
    data = (G / name).read_bytes()
    for needle in needles:
        assert needle in data.decode(), (name, needle)
    pins.append({'private_path': 'private/ghidra/' + name, 'sha256': sha(data)})

pin('terrain-camera-refs/instructions-0007d090.txt', '0007d5d5 CALL 0x0007f580', '0007d609 CALL 0x000801c0', '0007d613 CALL 0x00080830', '0007d771 CALL 0x00072ad0')
pin('player-creation/instructions-0007f580.txt', '0007f5a3 ADD EDX,0x24', '0007f5d6 CMP DL,0x4', '0007f699 CALL 0x0007fad0', '0007f6ce CMP EAX,0x5', '0007f6c2 INC byte ptr [EBX]')
pin('player-creation/instructions-0007fad0.txt', '0007fdb0 MOV byte ptr [EAX + 0x15],BL', '0007fdb3 MOV byte ptr [EAX + 0x16],CH', '0007ffd4 CALL 0x0009a630')
pin('font-text/instructions-0001d770.txt', '0001da68 ADD EDX,0x115e90', '0001da71 MOV dword ptr [EDX],EAX')
pin('mission-target-refs/instructions-000801c0.txt', '000801ed CMP EDX,0x1', '0008021d MOVSX EDX,byte ptr [EBX + 0x6e7]', '00080224 CMP EDX,0x1', '00080256 CMP EDX,0x4', '0008029c MOV DL,byte ptr [EDI + 0x16]', '000802be MOV dword ptr [EBX + 0x9],EDX')
pin('player-creation/instructions-00081310.txt', '00081315 MOVSX ECX,DX', '00081342 MOV ECX,0x16360e', '00081350 JMP 0x000802ff')
pin('mission-array-refs/instructions-00081300.txt', '00080304 IMUL EAX,EAX,0x3e8', '00080323 OR byte ptr [EBX + 0x20],0x1', '00080332 CALL 0x0004fd30', '00080343 MOV byte ptr [ESI + 0x3],AL', '00080349 MOV byte ptr [EDX + ESI*0x1 + 0x4],AL')
pin('player-creation/instructions-0004fd30.txt', '0004fd7b MOV byte ptr [ECX + 0x6],0x3', '0004fd88 MOV word ptr [ECX + 0xf],BX', '0004fef4 MOV word ptr [ECX + 0x15],0x280', '0004ff0e MOV word ptr [ECX + 0x70],0x0', '0004ff2c MOV dword ptr [ECX + 0xc0],EBX')
pin('player-creation/instructions-00013eb0.txt', '00013eb0 MOV byte ptr [EAX + 0x2],0x0')
pin('player-creation/instructions-0001f480.txt', '0001f481 MOV word ptr [EAX + 0x2],0x0')
pin('camera-player-input/instructions-00080830.txt', '00080886 MOV EAX,dword ptr [EBX + 0x9]', '00080899 CALL dword ptr [ESI + 0x60]', '000808a8 MOV byte ptr [EDX + 0x19],0x8', '000808dc OR byte ptr [EDX + 0x71],0x4', '00080a25 CALL 0x0007f3b0')
pin('mission-tail-init/instructions-00056e20.txt', '00056e58 MOV dword ptr [ECX + 0xc0],EBX', '00056e63 CALL 0x000982c0', '00056eab MOV dword ptr [ECX + 0x1f],EAX')
pin('player-creation/instructions-00057a10.txt', '00057a1e TEST byte ptr [EAX + 0x70],0x1', '00057a24 JMP 0x0005876f', '0005876f XOR AL,AL', '00058779 RET')
pin('entity-shadow-panel/instructions-0007f3b0.txt', '0007f431 MOV EAX,0x31', '0007f436 CALL 0x00048790', '0007f449 MOV word ptr [EBX],CX', '0007f465 CALL 0x00075b00')
pin('mission-array-refs/instructions-00048790.txt', '000487ae MOVSX EAX,word ptr [0x0011c2b4]', '000487c0 MOV dword ptr [EBP + 0x26],0x1688ae', '000488d8 MOV EDX,0x5', '000489a8 MOV byte ptr [EDX + 0x45],AL', '00048aec MOV word ptr [0x0011c2b4],SI')
pin('player-creation/instructions-0001f070.txt', '0001f082 MOV byte ptr [EAX + 0x6],DL', '0001f085 MOV byte ptr [EAX + 0x18],CL')
pin('player-creation/instructions-00049e90.txt', '00049fb6 MOV dword ptr [EDX + 0x1f],EBX')
pin('player-creation/instructions-0001e810.txt', '0001e831 CALL 0x000417e0')
pin('mission-tail-readers/instructions-0005a850.txt', '0005a85e MOV ECX,0x55', '0005a88d CALL dword ptr [ESI + 0x60]')
pin('mission-tail-init/instructions-0005a1c0.txt', '0005a1d6 MOVSX EDX,byte ptr [EAX + 0x7a]', '0005a1eb MOV dword ptr [EAX + 0x1f],ECX')
pin('mission-array-refs/instructions-0006a540.txt', '0006a586 CMP EDX,0x15d40e', '0006a5a9 CALL dword ptr [ESI + 0x68]', '0006a5e0 CALL dword ptr [ESI + 0x64]', '0006a623 DEC EBX')
pin('player-creation/instructions-0001ec50.txt', '0001ec50 CMP byte ptr [EAX + 0x5e],0x18')
pin('player-creation/instructions-0005ed20.txt', '0005ee1f CALL dword ptr [EDX + 0x94]', '0005ee2d ADD AL,byte ptr [EDX + 0xf]', '0005ee7b CALL 0x00083140')
pin('player-creation/instructions-0006fb60.txt', '0006fb91 CALL dword ptr [EDX + 0x68]', '0006fbf8 CALL 0x0008d7b0')
pin('player-creation/instructions-0005eb30.txt', '0005eb47 CALL dword ptr [EBX + 0xac]')
pin('player-creation/instructions-0005e640.txt', '0005e658 JMP dword ptr CS:[EAX*0x4 + 0x5e5c4]')
pin('mission-array-refs/instructions-00058780.txt', '000587bb MOVSX EAX,byte ptr [ECX + 0x118d77]', '000587b8 CALL dword ptr [EBX + 0x64]')
pin('mission-array-refs/instructions-00047f80.txt', '000480d0 CALL 0x000437a0', '0004834a CALL dword ptr [EDI + 0x64]')
pin('entity-order-refs/instructions-0001c120.txt', '0001c76d CALL 0x00058780', '0001c778 CALL 0x00047f80', '0001c785 CALL 0x0006a540', '0001c790 CALL 0x0006d0b0', '0001c7c0 CALL 0x0004e950')

code = (G / 'validated/memory-.object1.bin').read_bytes()
memory = (G / 'validated/memory-.object3.bin').read_bytes()
tables = []
for va, values in [(0x801a4,[0x801dc,0x80207,0x80273,0x80241]),(0x48674+0x31*4,[0x48c77])]:
    data = code[va-0x10000:va-0x10000+4*len(values)]
    assert list(struct.unpack('<'+'I'*len(values),data)) == values
    tables.append({'va':hex(va),'length':len(data),'sha256':sha(data),'targets':[hex(v) for v in values]})
for va, value in [(0x112194+0x64,0x57a10),(0x111fec+0x64,0x5ed20),(0x111fec+0x68,0x1ec50),(0x111fec+0x94,0x6fb60),(0x112254+0x78,0x1f070),(0x112254+0x44,0x1e810)]:
    data=memory[va-0x100000:va-0x100000+4];assert struct.unpack('<I',data)[0]==value
    tables.append({'va':hex(va),'length':4,'sha256':sha(data),'target':hex(value)})

def eligible(category, mask, progress, campaign):
    assert 0<=category<=255 and 0<=mask<=255 and -128<=progress<=127
    return category<4 and bool(mask & (1<<category)) and (not campaign or category in (0,2) or progress >= (1 if category==1 else 4))

def plan(records, mask, progress, campaign, player, active_slots, old_count):
    assert len(records)==32 and len(active_slots)==5 and 0<=player<4 and 0<=old_count<=5
    slots=list(active_slots);used=set();attempts=[];created=[]
    for index,(active,category) in enumerate(records):
        assert 0<=active<=255 and 0<=category<=255
        if active and category not in used and eligible(category,mask,progress,campaign):
            used.add(category);attempts.append(index)
            if 0 in slots:
                local=slots.index(0);slots[local]=3
                assert old_count+len(created)<5, 'safe count/storage admission'
                created.append({'blueprint_index':index,'local_slot':local,'global_slot_id':player*5+local,'player_list_offset':4+old_count+len(created),'blueprint_flags20_or':1})
    return {'attempts':attempts,'created':created,'player_count_after':old_count+len(created),'slot_types_after':slots}

eligibility_cases=0
for category,mask,progress,campaign in itertools.product(range(4),range(256),range(-128,128),(False,True)):
    branch=False
    if category==0:branch=bool(mask&1)
    elif category==1:branch=bool(mask&2) and (progress>=1 or not campaign)
    elif category==2:branch=bool(mask&4)
    elif category==3:branch=bool(mask&8) and (progress>=4 or not campaign)
    assert eligible(category,mask,progress,campaign)==branch;eligibility_cases+=1
allocation_cases=0
for bits,player,index in itertools.product(range(32),range(4),range(32)):
    slots=[3 if bits>>i&1 else 0 for i in range(5)];records=[(0,0)]*32;records[index]=(1,0)
    result=plan(records,1,0,True,player,slots,sum(bool(v) for v in slots))
    expected=next((i for i in range(5) if not(bits>>i&1)),None)
    assert result['attempts']==[index]
    assert [r['global_slot_id'] for r in result['created']]==([] if expected is None else [player*5+expected])
    allocation_cases+=1
vectors=[]
for categories,mask,progress,campaign,slots,count in [([0,0,1,2,3],15,0,True,[0]*5,0),([3,2,1,0,2],15,4,True,[0]*5,0),([0,0,2,2],5,0,True,[3]*5,5),([0,1,2,3],15,-128,False,[0,3,0,3,0],2),([255,3,1,2,0],255,-1,True,[0]*5,0)]:
    records=[(1,c) for c in categories]+[(0,0)]*(32-len(categories))
    vectors.append({'categories':categories,'mask_u8':mask,'progress_i8':progress,'campaign':campaign,'player':0,'slots':slots,'old_count':count,'result':plan(records,mask,progress,campaign,0,slots,count)})
generated=[(1,i//5) for i in range(20)]+[(0,0)]*12
initial=plan(generated,5,0,True,0,[0]*5,0)
assert initial['attempts']==[0,10] and initial['player_count_after']==2
source=(G/'campaign-resources/152.dat').read_bytes()
assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
assert source[0x1ebd]==5 and source[0x1ec2:0x1ec6]==bytes(4) and source[0x34b8]==0
source_groups=json.loads((P/'research/ghidra/mission-load-evidence.json').read_text())['serialized_groups']
assert [g['active_raw_byte_nonzero'] for g in source_groups]==[1,40,0,0,0,0,0,0]
raw=source[0x3b55e:0x3b55e+116];native=raw[:31]+bytes(8)+raw[31:]
assert native[6]==1 and native[0x15]==0x80 and native[0x18]==4 and native[0x5e]==8 and native[0x7a]==4
text=(P/'private/integration/menu/text.dat').read_bytes()
assert sha(text)=='23e60305a9fa028a0f6541456ab453b7e7cc439f787a2b75e4b8d088e0b01786'
names=text.split(b'\0')[:128]
assert all(names) and len(set(names))==128 and max(map(len,names))<20
for flag in range(256):
    assert (flag&1==0)==(flag in range(0,256,2))

receipt={'candidate_sha256':candidate,'instruction_contracts':pins,'mapped_tables':tables,
 'scope':'Pure blueprint category selection, first-free actor-slot requests and suppressed actor-update gate. No AI, audio, pointers, live first-frame or complete world initialization claim.',
 'source':{'entry_id':'root/2/disc/152','sha256':sha(source),'mask_source_offset':'0x1EBD','mask_u8':5,'special_branch_dword_source_offset':'0x1EC2','special_branch_dword':0,'world_auxiliary_source_count_at34B8':0,'loaded_active_family_counts':[1,40,0,0,0,0,0,0]},
 'names_prerequisite':{'entry_id':'root/2/disc/592','sha256':sha(text),'first128_nonempty_unique':True,'maximum_name_bytes':max(map(len,names)),'scope':'Metadata generation needs initialized TEXT pointers; names remain private. RNG/style/skill/name choices are not emulated.'},
 'selection_contract':{'record_count':32,'record_stride':36,'player_relative_base':'0x16','active_u8_offset':0,'category_u8_offset':'0x16','record_flags_u32_offset':'0x20','categories':{'0':{'mask':1},'1':{'mask':2,'campaign_min_progress':1},'2':{'mask':4},'3':{'mask':8,'campaign_min_progress':4}},'progress':'signed i8 at player+0x6E7','quota':'at most one attempt/category; quota consumed before allocation even on failure','target_reset':'After selection, player+9 receives separate target table11A9DE[unsigned player+1].'},
 'allocation_contract':{'player':'safe0..3','family':2,'base_va':'0x16360E','player_partition_bytes':1000,'slots_per_player':5,'native_stride':200,'global_slot_id':'player*5+local_slot','selection':'first active byte+6 equal0 in ascending local slot order; no slot returns AL0 without changes','success':'Blueprint byte+0x20 OR1; 4FD30 sets active3/owner/player-ID/resource metadata pointer. Append lowbyte global ID to player+4+signed oldcount and byte-increment player+3. Safe plan admits count0..5 and checks list capacity on each successful free-slot allocation. It does not require count to equal occupied slots; inconsistent counts are accepted when no append overflows.'},
 'new_actor_fields':{'active_type_u8':3,'owner_u8':'player','id_u16':'player*5+local_slot','blueprint_index_u8_at74':'selected index0..31','flags15_u16_after_4FD30':'0x0280','flags70_u16_after_4FD30':'0x0000','after_80830':{'position_at7':'player+9 explicit target','flags16_or':8,'flags71_or':4,'animation19':8,'frame1b':0,'counter1d':0,'state1c':1,'flags15_u16':'0x0A80','flags70_u16':'0x0400'},'runtime_initializer':'56E20 binds resource using blueprint style+15/category+16, clears native94..C0 and populates runtime pointers; source bytes do not encode these pointers.'},
 'actor_update_gate':{'entry':'0x57A10','flag':'native byte+0x70 mask01','clear':'Direct jump5876F, return AL0, no mutation/callback/queue request','set':'Full update unresolved; do not approximate','ordinary_dispatch':'58780 uses signed player+3 count and signed byte IDs player+4 onward, calling virtual+64 in stored order. Safe IDs must belong to valid20-slot array.'},
 'generated_metadata_contract':'7F580 clears32 active-name bytes and count, then four category loops with five records each.7FAD0 writes category+16 from outer index, style+15 from RNG modulo4, and selects an unused name among32 per style. With initialized hash-bound nonempty/unique names it creates20 records in category order0,1,2,3. This count/category statement does not specify random fields or copy unsafe strings.',
 'initial_creation_gates':['Successful hash-bound CONQ001 load, single-player campaign, selectedplayer0, retained player+2=0 and signed progress0, player+3=0, initialized TEXT/name pointers, inactive five family2slots. Selected-player restoration preserves prior state, so these runtime gates are not inferred from raw player bytes alone.','Under those gates metadata category selection attempts0,10 and creates family2 IDs0,1.80830 with EDX=-1 and special dword11AC22=0 calls7F3B0, requesting factorycode31 at separate target81,53.'],
 'initial_creation_plan':initial,
 'family4_factory_request':{'entry':'0x7F3B0','factory':'0x48790','object_code_u8':'0x31','numeric_family':4,'native_type_byte':5,'native_base_va':'0x1688AE','stride':81,'capacity':200,'position':'player+9 when11AC22=0','arguments':'height helper+0x6400, anglewords0/0x400/0x440, ownerplayer, remainingargszero','effects':'Factory increments family4 count; binds resource116818+10*0x31; successful reference saved toplayer+0x496/+0x498. Selectedowner requests reveal75B00(4096). Virtual44 is audio request1E810→417E0, retained external effect. Full pool replacement behavior is excluded.'},
 'family0_loaded_contract':{'file_base':'0x3B55E','packed_stride':116,'native_base_va':'0x1542BE','native_stride':124,'capacity':300,'active_count':1,'active_index':0,'type_u8':1,'flags15_u8':128,'owner_u8':4,'state5e_u8':8,'resource_selector7a_i8':4,'initialization':'5A850 copies header31/payload85 around runtime pointers, then calls5A1C0.5A1C0 binds180E84[signed selector], with fallback table104A64, and does not create player actors or change the loaded count.'},
 'preceding_update_boundary':'Ordinary1C120 requires118770mask04 set and118968mask02 clear; dispatches58780 family2,47F80 family3/playerbookkeeping,6A540 family0,6D0B0 plants, then other families includingfamily4.6A540 bounds its primary scan to300slots, skips inactive/flags15mask01, consumes one signedremainingcount per virtual64 call. Its creation pass can invoke marked newborn records without an array-end guard.5ED20 firstcalls6FB60 virtual94 before parity gating;6FB60 and5EB30/5E640 contain transitive AI/physics/target callbacks. Full preceding effects remain explicit unknown inputs; do not assume the loaded plant neighborhood survives them.',
 'capacity_boundary':'Created family2 actors gate-suppress themselves whilebyte70mask01 clear. Family4 is updated after plants on ordinary route. The sole loaded family0 object can request queue admission beforeplants, but family3/playerbookkeeping and fullfamily0 callbacks remain unresolved creation/mutation boundaries. Hence no universal priorqueuecount1/3/4 or livepopulation44 claim. An explicit prioracceptedqueuecount<=216 stillguarantees40plant slots under existing256entry queue contract.',
 'eligibility_cases':eligibility_cases,'allocation_cases':allocation_cases,'actor_gate_cases':256,'vectors':vectors}
(P/'research/ghidra/player-creation-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS {len(pins)} instruction pins; {len(tables)} mapped targets; {eligibility_cases} category cases; {allocation_cases} allocations; 256 actor gates; initial attempts0/10 -> IDs0/1.')
