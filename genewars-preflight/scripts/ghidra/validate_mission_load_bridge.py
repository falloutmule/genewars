"""Bounded source-only mission tail and initial camera-target receipt."""
from pathlib import Path
import hashlib
import json
import re
import struct

P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
mapped=(G/'validated/memory-.object3.bin').read_bytes()
word=lambda a:struct.unpack_from('<I',mapped,a-0x100000)[0]
# capacity, native stride, packed stride, ctor descriptor, ctor, vtable, reader, init
families=[
 (300,0x7c,0x74,0x111fc4,0x1e390,0x111fec,0x5a850,0x5a1c0),
 (512,0x31,0x29,0x111fb0,0x1e3e0,0x1120ac,0x6c950,0x6c690),
 (20,0xc8,0x8c,0x111f9c,0x1e400,0x112194,0x57540,0x56e20),
 (256,0x43,0x3b,0x111f88,0x1e3f0,0x112120,0x45010,0x44ff0),
 (200,0x51,0x49,0x111f74,0x1e450,0x112254,0x49fd0,0x49e90),
 (32,0x31,0x29,0x111f60,0x1e4a0,0x1122fc,0x4ee80,0x4ee70),
 (32,0x61,0x59,0x111f4c,0x1e4b0,0x11236c,0x13f00,0x13ef0),
 (16,0x66,0x5e,0x111f38,0x1e500,0x11242c,0x10090,0x10060),
]
listings=[]
def pin(name,lines=()):
    raw=(G/name).read_bytes()
    for line in lines:assert line in raw.decode(),(name,line)
    listings.append({'private_path':'private/ghidra/'+name,'sha256':sha(raw)})
pin('campaign-start/instructions-00080460.txt',['00080499 MOV EAX,dword ptr [EAX*0x4 + 0x11a9de]','000804a6 MOV dword ptr [ECX + 0x9],EAX'])
pin('camera-player-input/instructions-00080590.txt',['00080596 MOV ECX,0x4a1','000805af MOV ECX,0xdd','000805b4 LEA ESI,[EBX + 0x61a]'])
pin('campaign-level/instructions-00071b70.txt',['00071ce1 MOV EBX,0x3b55a','00071e4f CALL 0x00080590','00072340 MOV EBX,0x34','00072345 MOV EDX,0x118770','00072380 MOV EBX,0x4','00072385 MOV EDX,0x16e0a8','000723d5 CMP dword ptr [0x001831bc],0x6be','000723e1 MOV EBX,0x1af4','0007241a MOV AL,byte ptr [EDX + 0x1c5b]','0007246d MOV dword ptr [EDX + 0x1c67],EAX'])
pin('terrain-camera-refs/instructions-0007d090.txt',['0007d54a TEST byte ptr [0x00118770],0x20','0007d551 JNZ 0x0007d55d','0007d558 CALL 0x0008a9f0','0007d5a2 MOV EDX,dword ptr [ECX + 0x9]','0007d5a5 CALL 0x00082e20','0007d6b4 MOV AL,[0x0011878c]','0007d6e5 MOV EAX,dword ptr [EAX + 0x11a9de]','0007d6eb MOV dword ptr [ECX + 0x9],EAX'])
pin('terrain-camera-refs/instructions-00082e20.txt',['00082e2d CALL 0x0007b140','00082e32 SAR EAX,0x6','00082e4b ADD AH,0xc','00082e5a ADD AL,0x2','00082e78 MOV [0x00184fdc],EAX','00082e87 MOV [0x00184fe0],EAX'])
pin('mission-target/instructions-0008a9f0.txt',['0008aa01 TEST byte ptr [0x00118770],0x20','0008aa08 JNZ 0x0008aac2','0008aaaf LEA EDI,[EBP + 0x36]'])
pin('mission-target/instructions-000884d0.txt',['000884dd MOV EAX,0x9000','000887b4 MOV EAX,0x112d24'])
pin('mission-tail-constructors/instructions-0001e280.txt',['0001e285 MOV EBX,0x111fd8','0001e2b0 CALL 0x000a76d0','0001e2c4 CALL 0x000a76d0','0001e355 LEA EDI,[EAX + 0x1c48]'])
pin('mission-tail-constructors/instructions-000790d0.txt',['000790f0 MOV EAX,EBX'])
pin('mission-tail-readers/instructions-0006d490.txt',['0006d494 MOV ECX,0x1f','0006d499 MOV EDI,EAX','0006d4b2 MOV byte ptr [EAX + 0x17],0x20','0006d4b9 MOV byte ptr [EAX + 0x15],BL','0006d4bc ADD dword ptr [EDX],0x1f'])
pin('mission-tail-readers/instructions-000982c0.txt',['000982ce MOV ECX,EBX','000982d0 CALL 0x000b6930'])
pin('mission-tail-readers/instructions-000b6930.txt',['000b693a MOV byte ptr [EAX],DL','000b6944 SHR ECX,0x2','000b6947 CALL 0x000b6967','000b694d AND ECX,0x3'])
pin('mission-tail-readers/instructions-000b6967.txt',['000b696f MOV dword ptr [EAX],EDX','000b6974 DEC ECX','000b69b8 AND ECX,0x3','000b69cf LEA EAX,[EAX + 0x4]'])
for count,native,packed,descriptor,ctor,vtable,reader,init in families:
    assert word(descriptor+1)==ctor and word(descriptor+13)==native
    assert word(vtable+0x3c)==reader and word(vtable+0x60)==init
    pin(f'mission-tail-constructors/instructions-{ctor:08x}.txt',[f'MOV dword ptr [EAX + 0x23],0x{vtable:x}'])
    pin(f'mission-tail-readers/instructions-{reader:08x}.txt',['CALL 0x0006d490',f'MOV ECX,0x{packed-31:x}','LEA EDI,[EBX + 0x27]',f'ADD dword ptr [EBP],0x{packed-31:x}'])
    pin(f'mission-tail-init/instructions-{init:08x}.txt')
    body=(G/f'mission-tail-init/instructions-{init:08x}.txt').read_text()
    calls=re.findall(r'CALL ([^\r\n]+)',body)
    assert calls==(['0x000982c0'] if init==0x56e20 else []),(hex(init),calls)

source=(G/'campaign-resources/152.dat').read_bytes()
assert len(source)==343958 and sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
assert struct.unpack_from('<I',source)[0]==0x55336
targets=list(struct.unpack_from('<4I',source,0x1c7e))
assert targets==[0x35005100,0,0,0]
assert all(struct.unpack_from('<I',source,20+i*0x70d+9)[0]==0 for i in range(4))
cursor=0x3b55e;native_cursor=0x3b55e;groups=[]
for family,(count,native,packed,descriptor,ctor,vtable,reader,init) in enumerate(families):
    end=cursor+count*packed
    assert end<=len(source)
    header=31;payload=packed-header;payload_end=0x27+payload
    assert payload_end<=native
    active=sum(source[cursor+i*packed+6]!=0 for i in range(count))
    groups.append({'family':family,'count':count,'file_offset':cursor,'file_end_exclusive':end,
                   'packed_stride':packed,'native_state_offset':native_cursor,'native_stride':native,
                   'serialized_header':[0,31],'serialized_payload':[31,packed],
                   'native_header':[0,31],'native_payload':[39,payload_end],
                   'unserialized_native_ranges':[[31,39]]+([[payload_end,native]] if payload_end<native else []),
                   'constructor_descriptor':hex(descriptor),'constructor':hex(ctor),'vtable':hex(vtable),
                   'reader':hex(reader),'active_init':hex(init),'active_raw_byte_nonzero':active,
                   'source_sha256':sha(source[cursor:end])})
    cursor=end;native_cursor+=count*native
assert cursor==0x52466 and native_cursor==0x55336
globals_bytes=source[cursor:cursor+52]
assert len(globals_bytes)==52 and globals_bytes[0]==0x38 and globals_bytes[28]==1
assert globals_bytes[0]&0x20
viewport_offset=cursor+52;extension_offset=viewport_offset+4
assert struct.unpack_from('<I',source,extension_offset)[0]==0x6be
assert extension_offset+4+0x1af4==len(source)
assert source[0x1c5b]==0
height=struct.unpack_from('<h',source,0x355e+(53*128+81)*14+2)[0]
assert height==3136
height2=height*2;shifted=height2>>6
q=abs(shifted)//32*(-1 if shifted<0 else 1)
camera=[(81-q-12)&255,(53-q-2)&255]
assert height2==6272 and q==3 and camera==[66,48]
restore_ranges=[[0,0x4a1],[0x61a,0x61a+0xdd]]
for i in range(4):
    base=20+i*0x70d
    assert all(base+end<=0x1c7e for start,end in restore_ranges)
receipt={'candidate_sha256':candidate,'instruction_contracts':listings,
 'scope':'ExactCDCONQ001 source-only prefix/tail and gatedinitialcamera derivation; not a full live-state emulator.',
 'source':{'entry_id':'root/2/disc/152','size':len(source),'sha256':sha(source),'layout_tag':'0x55336'},
 'start_target_table':{'file_offset':0x1c7e,'count':4,'entry_stride':4,'word_layout':'u16x/u16y packedfraction8 coordinates','values':targets,'first_target_x_y':[20736,13568],'initial_loaded_player_plus9_allzero':True},
 'target_producers':{'80460':'Reads11A9DE+4*player into player+9.','7D090':'Eachactiveplayer iteration unconditionally copies11A9DE+4*player to+9 at7D6E5/6EB before selected-player82E20call7D5A2.'},
 'player_restore_ranges':restore_ranges,
 'serialized_groups':groups,
 'tail_globals':{'file_offset':cursor,'size':52,'destination':'0x118770','flags_byte_offset':0,'flags_value':56,'active_player_count_byte_offset':28,'active_player_count':1,'sha256':sha(globals_bytes)},
 'viewport_snapshot':{'file_offset':viewport_offset,'size':4,'destination':'0x16e0a8','sha256':sha(source[viewport_offset:extension_offset]),'note':'Later82E20overwritescamera; this is not finalinitialcamera.'},
 'extension':{'file_offset':extension_offset,'tag':0x6be,'payload_offset':extension_offset+4,'payload_size':0x1af4,'payload_sha256':sha(source[extension_offset+4:]),'interpretation':'opaque'},
 'initial_target_gates':['Successfulfileload exact55336source.','Selectedplayer0 andactivecount1. Currentselectedindex is retained bymenu; do notgeneralize toarbitrarysessionstate.','Tailflags38preservecampaign20, so8A9F0targetpermutation is skipped.','Eightvirtualreadersresolvedthroughconstructorvtable; activeinitwritesonlyentity/stack, so targettable andmapheight survive.'],
 'first_camera':{'selected_player':0,'target_u32':targets[0],'target_cell':[81,53],'target_fraction':[0,0],'height_record_i16':height,'height_helper_output':height2,'q':q,'camera_cell':camera},
 'b32_file_selector':{'file_offset':0x1c5b,'value':0,'format':'data\\b32%c.dat','character':'a','result':'data/b32a.dat'},
 'safety':['Borrowrawserializedrecords; do not inventnativevptr/resourcepointers or copyentirepackedrecords tonativestruct.','Family2nativeC8 serializesonly[0,1F)and[27,94), leaving94..C8runtime-only.','Base6D490mutates staginginput+15/+17 aftercopy, notsourcefile; source-onlyparserpreservesoriginalbytes.','Opaqueextension/globalbytes remainopaque exceptexplicitflag/countfields.','No original missionexecution or runtime-statecapture occurred.']}
(P/'research/ghidra/mission-load-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS {len(listings)} listingpins; 8 vtable-resolvedtailgroups/1368source records; exactfileend; campaigngatedplayer0target81,53/camera66,48')
