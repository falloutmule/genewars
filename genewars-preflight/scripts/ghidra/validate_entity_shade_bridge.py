"""Derive entity shade from explicit map light/factor data; static code only."""
from pathlib import Path
import hashlib
import json
import struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
contracts=[]
for name,lines in {
 'camera-player-input/instructions-0007b240.txt':['0007b28c MOV BL,byte ptr [ESI + EAX*0x1 + 0xc]','0007b290 MOVSX EAX,word ptr [ESI + EAX*0x1]','0007b2a1 SAR EBX,0x6','0007b2e4 CMP ECX,EDX','0007b31e SAR EAX,0x7','0007b353 SAR EAX,0x7','0007b358 SUB EAX,0x1e00','0007b35d SAR EAX,0x8','0007b360 LEA EDX,[EAX + 0x10]','0007b363 CMP EDX,0x40','0007b373 XOR EAX,EDX'],
 'camera-player-input/instructions-00072960.txt':['0007296d LEA ECX,[EAX + 0x446ae]','000729a8 ADD EDX,0x31','000729bf CALL 0x0007b240','000729c4 MOV byte ptr [EDX + 0x17],AL']}.items():
    b=(G/name).read_bytes()
    for line in lines:assert line in b.decode(),line
    contracts.append({'private_path':'private/ghidra/'+name,'sha256':sha(b)})

def corners(light,factor,fx,fy):
    assert all(-32768<=v<=32767 for v in light) and all(0<=v<=255 for v in factor)
    assert 0<=fx<=255 and 0<=fy<=255
    a,b,c,d=[(l*f)>>6 for l,f in zip(light,factor)]
    n=(d-c)*fx+(c-a)*fy if fx<fy else (b-a)*fx+(d-b)*fy
    p=2*a+(n>>7)
    return max(0,min(64,((p-0x1e00)>>8)+16))

def weighted(light,factor,fx,fy):
    a,b,c,d=[(l*f)//64 for l,f in zip(light,factor)]
    p=(((256-fy)*a+(fy-fx)*c+fx*d) if fx<fy else ((256-fx)*a+(fx-fy)*b+fy*d))//128
    return max(0,min(64,(p-7680)//256+16))

def from_region(region,position):
    if len(region)!=229376:raise ValueError('exact map region required')
    x,y=[(v>>8)&127 for v in position];fx,fy=[v&255 for v in position]
    offsets=[(((b&127)*128+(a&127))*14) for a,b in [(x,y),(x+1,y),(x,y+1),(x+1,y+1)]]
    return corners([struct.unpack_from('<h',region,o)[0] for o in offsets],[region[o+12] for o in offsets],fx,fy)

checks=0;vectors=[]
cases=[([0]*4,[0]*4),([4096]*4,[63]*4),([8192]*4,[63]*4),([-32768,32767,-1,1],[255,255,1,63]),([4096,4097,8192,-8192],[63,62,0,255]),([32767]*4,[255]*4),([-32768]*4,[255]*4)]
for lights,factors in cases:
    for fx in range(256):
        for fy in range(256):
            assert corners(lights,factors,fx,fy)==weighted(lights,factors,fx,fy)
            checks+=1
    for fx,fy in [(0,0),(1,0),(0,1),(127,128),(128,127),(255,255)]:
        vectors.append({'light_i16':lights,'factor_u8':factors,'fraction_xy':[fx,fy],'shade_u8':corners(lights,factors,fx,fy)})
assert corners([4096]*4,[63]*4,0,0)==17
assert corners([8192]*4,[63]*4,0,0)==49
source=(G/'campaign-resources/152.dat').read_bytes();raw=source[0x355e:0x3b55e]
assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
explicit=bytearray(raw)
for o in range(12,len(explicit),14):explicit[o]=63
fixtures=[]
for i in range(512):
    s=source[0x43d4e+i*41:0x43d4e+(i+1)*41]
    if s[6]:
        pos=struct.unpack_from('<hh',s,7)
        fixtures.append({'record':i,'raw_shade':from_region(raw,pos),'explicit_all_factor63_shade':from_region(explicit,pos)})
private=G/'entity-plant/shade-fixture.json';payload=(json.dumps(fixtures,separators=(',',':'))+'\n').encode();private.write_bytes(payload)
receipt={'candidate_sha256':candidate,'instruction_contracts':contracts,
 'input':'Exact229376-byte128x12814-byte-cell region, signed16 position words with8fractionbits. Neighbors wrap128; cell+0 signed16 light and cell+12 unsigned8 factor.',
 'corner_order':['00','10','01','11'],'corner_rule':'q_i=(signed_light_i*unsigned_factor_i)>>6, arithmetic floor independently BEFORE interpolation; q_i is signed32, not narrowed signed16.',
 'interpolation':'fx=positionX&255,fy=positionY&255. If fx<fy: n=(q11-q01)*fx+(q01-q00)*fy. Else: n=(q10-q00)*fx+(q11-q10)*fy. p=2*q00+(n>>7).',
 'output':'clamp(((p-7680)>>8)+16,0,64); unsigned byte result. All shifts arithmetic floor. Signed32 products/intermediates suffice for full i16 light/u8factor domain.',
 'consumer':'72960 iterates active native family1 table446AE..4A8AE, stride31; passes native+7 position to7B240 and stores AL at native+17. Refresh timing and visibility callbacks separate.',
 'vectors':vectors,'weighted_equivalence_cases':checks,
 'actual_fixture':{'source_entry':'root/2/disc/152','sha256':sha(source),'active_points':len(fixtures),'explicit_second_input':'copy of raw map with everyfactor byte replaced63; diagnostic, no original state claim','private_path':'private/ghidra/entity-plant/shade-fixture.json','sha256':sha(payload)},
 'limits':['Does not establish initial factor state, visibility or animation timing.','Do not substitute terrain vertex shade formula or floor once after interpolation; corner floor order matters.','No FADE lookup here. Foreground dispatch separately bypassesFADE for shade>=30 or flags+15 mask02.']}
(P/'research/ghidra/entity-shade-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS2pins; {checks} weighted/corner-floor comparisons; 42vectors; {len(fixtures)}actual source-point fixtures')
