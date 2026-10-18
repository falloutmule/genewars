"""Validate7AF00 map point projection with explicit camera and panel margin."""
from pathlib import Path
import hashlib
import json
import struct

P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
name='entity-plant/instructions-0007af00.txt';raw=(G/name).read_bytes()
for line in ['0007af10 MOVSX EDX,CX','0007af2b MOVSX ESI,BX','0007af4f XOR DH,BH','0007af56 XOR DH,DH','0007af73 SUB ECX,EBP','0007af79 IMUL EBP,dword ptr [0x00184fe4]','0007afdb JL 0x0007b02e','0007b028 SAR EDX,CL','0007b063 SAR EDX,CL','0007b069 MOV CX,word ptr [0x00184fdc]','0007b070 ADD EBX,EBX','0007b075 SUB EBX,ECX','0007b083 SAR BX,0x1','0007b08b SAR AX,0x1','0007b09e MOV EAX,[0x00184fd8]','0007b0a8 XOR ESI,ESI','0007b0aa SAR EDX,0x6','0007b0ad MOV SI,word ptr [0x0016e0c5]','0007b0b4 SAR EDI,0x8','0007b0b7 SAR ECX,0x8','0007b0c6 MOV dword ptr [ESI],EDI','0007b0c8 MOV dword ptr [EDX],ECX']:
    assert line in raw.decode(),line
signed=lambda n,bits:(n+(1<<(bits-1)))%(1<<bits)-(1<<(bits-1))

def project(position,camera,height2,margin):
    assert all(-32768<=v<=32767 for v in position)
    assert all(0<=v<=65535 for v in camera) and 0<=margin<=65535
    assert -65536<=height2<=65534
    dx=signed(position[0]-(camera[0]<<8),15)
    dy=signed(position[1]-(camera[1]<<8),15)
    return [((dx-dy)*32>>8)-margin,((dx+dy)*16>>8)-(height2>>6)]

def register_project(position,camera,height2,margin):
    # Only low16 survives before the word-sized arithmetic right shifts.
    bx=signed(2*position[0]-(camera[0]<<9),16)>>1
    ax=signed(2*position[1]-(camera[1]<<9),16)>>1
    edi=((bx-ax)*32)>>8
    ecx=((bx+ax)*16)>>8
    return [edi-margin,ecx-(height2>>6)]

def height_at(region,position):
    if len(region)!=229376:raise ValueError('exact14-byte-cell map region required')
    x,y=[(v>>8)&127 for v in position];fx,fy=[v&255 for v in position]
    def h(a,b):return struct.unpack_from('<h',region,(((b&127)*128+(a&127))*14)+2)[0]
    a,b,c,d=h(x,y),h(x+1,y),h(x,y+1),h(x+1,y+1)
    n=(d-c)*fx+(c-a)*fy if fx<fy else (b-a)*fx+(d-b)*fy
    return 2*a+(n>>7)

vectors=[];checks=0
positions=[-32768,-32767,-16385,-16384,-16383,-257,-256,-255,-1,0,1,255,256,257,16383,16384,32767]
cameras=[0,1,63,64,127,128,255,256,32767,32768,65535]
heights=[-65536,-65,-64,-63,-1,0,1,63,64,65,65534]
for x in positions:
    for y in positions:
        for cx in cameras:
            cy=cameras[(checks//7)%len(cameras)]
            height=heights[checks%len(heights)];margin=[0,128,65535][checks%3]
            p=project([x,y],[cx,cy],height,margin)
            assert p==register_project([x,y],[cx,cy],height,margin)
            checks+=1
            if len(vectors)<12 and x in [-32768,0,32767] and y in [-32768,0,32767] and cx in [0,128,65535]:
                vectors.append({'position_i16':[x,y],'camera_u16':[cx,cy],'height2':height,'margin_u16':margin,'output_i32':p})
for p,c,h,m in [([0,0],[0,0],0,65535),([16384,0],[0,0],0,0),([-16384,0],[0,0],0,0),([0,0],[65535,32768],-1,128)]:
    vectors.append({'position_i16':p,'camera_u16':c,'height2':h,'margin_u16':m,'output_i32':project(p,c,h,m)})
assert project([0,0],[0,0],0,65535)==[-65535,0]
assert project([16384,0],[0,0],0,0)==[-2048,-1024]

source=(G/'campaign-resources/152.dat').read_bytes();region=source[0x355e:0x3b55e]
assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
points=[]
for index in range(512):
    r=source[0x43d4e+index*41:0x43d4e+(index+1)*41]
    if not r[6]:continue
    position=list(struct.unpack_from('<hh',r,7));height=height_at(region,position)
    result=project(position,[66,48],height,128)
    assert result==register_project(position,[66,48],height,128)
    points.append({'record':index,'position':position,'height2':height,'local_point':result})
assert len(points)==40
private=G/'entity-plant/point-fixture.json';payload=(json.dumps(points,separators=(',',':'))+'\n').encode();private.write_bytes(payload)
receipt={'candidate_sha256':candidate,'instruction_contracts':[{'private_path':'private/ghidra/'+name,'sha256':sha(raw)}],
 'function':'0x7af00','scale_precondition':{'height_factor':2,'horizontal_scale':32,'vertical_scale':16,'proof':'terrain-evidence.json'},
 'inputs':{'position':'two signed16wordswith8fractionbits','camera':'two unsigned16bitpatterns; allu16admitted','map':'exact229376B; signedheightwordcell+2; 128wrap','margin':'unsigned16 zeroextended'},
 'formula':'dx=signextend15(positionX-(cameraX<<8));dy=signextend15(positionY-(cameraY<<8));x=((dx-dy)*32>>8)-margin;y=((dx+dy)*16>>8)-(piecewise_doubled_height>>6)',
 'semantics':'Arithmeticfloor shifts; worddouble/subtract/SAR1implements15-bitwrap, preserving128-celltorusdelta. Height interpolationidentical7B140factor2/shift7. Fullsigned32outputstores, nofinalwordnarrowing.',
 'vectors':vectors,'word_delta_equivalence_checks':checks,
 'actual_fixture':{'source_entry':'root/2/disc/152','source_sha256':sha(source),'numeric_family':1,'active_points':40,'camera':[66,48],'panel_margin':128,'private_path':'private/ghidra/entity-plant/point-fixture.json','sha256':sha(payload)},
 'limits':['Pointhelperonly; caller83140laterstoresx/relativeywordsandsorts/clipsentities, separately.','No startupselectedplayer/factor/visibilitystateinferred; explicitcamera66,48testusespriorgatedmissionreceipt.','Do notreplaceheight2>>6with2*(height>>6)vertexroundingoruseunwrappeddeltas.']}
(P/'research/ghidra/entity-point-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS exact7AF00contract; {checks} word/signed15projection comparisons; 16boundaryvectors; 40actualsourcepoints')
