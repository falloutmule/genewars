"""Pin terrain modes0/6/15 and overlay resources; no original code execution."""
from pathlib import Path
import hashlib
import json
import struct
from validate_mode5_triangle_bridge import classify, slope, i32, prepare, fixtures

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
sha = lambda b: hashlib.sha256(b).hexdigest()
code = (G / 'validated/memory-.object1.bin').read_bytes()
candidate = sha((G / 'GW-CD-candidate.exe').read_bytes())
assert candidate == '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
targets = {hex(a): {str(m): hex(struct.unpack_from('<I', code, a - 0x10000 + 4*m)[0])
                   for m in [0, 5, 6, 15]} for a in [0xe1a67, 0xe2374, 0xe2ca2, 0xe3121, 0xe3513]}
for a in list(targets)[:4]:
    assert targets[a]['5'] == targets[a]['6']
    assert targets[a]['0'] == targets[a]['15']
assert targets['0xe3513'] == {'0':'0xe357f','5':'0xe3f93','6':'0xe42aa','15':'0xe5b70'}
contracts = {
 'terrain-overlay/mode6-span.txt': ['000e42b8 SHL EAX,0x10', '000e4313 MOV BH,DL', '000e431d MOV DX,AX', '000e4323 MOV BL,AH', '000e4330 MOV AH,CL', '000e4366 MOV BL,byte ptr [ESI + 0xa]', '000e4374 MOV DX,word ptr [ESI + 0x8]'],
 'terrain-overlay/mode6-lanes.txt': ['000e43e0 MOV AL,byte ptr [EBX + EBP*0x1]', '000e43e3 OR AL,AL', '000e43e5 JZ 0x000e43ef', '000e43e7 MOV AL,byte ptr [EAX + 0x188f48]', '000e43ef ADD DX,word ptr [ESP + 0x24]', '000e43f4 ADC BL,byte ptr [ESP + 0x26]', '000e43f8 ADD EDX,dword ptr [ESP + 0x4c]', '000e43fc ADC BH,byte ptr [ESP + 0x32]', '000e4400 ADD ECX,dword ptr [ESP + 0x50]', '000e4404 ADC AH,byte ptr [ESP + 0x3e]'],
 'terrain-triangle/mode0-span.txt': ['000e3588 MOV AL,[0x00102e24]', '000e35a4 ADD EDX,dword ptr [0x00100110]', '000e35e0 STOSB ES:EDI', '000e35e5 STOSW ES:EDI', '000e35e7 STOSD.REP ES:EDI'],
 'terrain-overlay/mode15-span.txt': ['000e5b79 MOVZX EAX,byte ptr [0x00102e24]', '000e5bcc MOV AH,byte ptr [EDI]', '000e5bce MOV AH,byte ptr [EAX + 0x18cf48]', '000e5bd4 MOV byte ptr [EDI],AH', '000e5cf5 DEC dword ptr [ESP + 0x20]'],
 'terrain-overlay/general-right-mode0.txt': ['000e2161 MOV EAX,dword ptr [ESI]', '000e225a MOV dword ptr [EDI],EAX', '000e2270 MOV EBX,dword ptr [ESP + 0x1c]', '000e228f MOV dword ptr [EDI + 0x4],EBX'],
 'terrain-overlay/general-left-mode0.txt': ['000e2ac8 MOV EAX,dword ptr [ESI]', '000e2bd7 MOV EAX,dword ptr [ESP + 0x1c]'],
 'terrain-overlay/flat-bottom-mode0.txt': ['000e3001 MOV EAX,dword ptr [ESI]', '000e3069 LEA EDI,[0x100124]'],
 'terrain-overlay/flat-top-mode0.txt': ['000e347e MOV EAX,dword ptr [ESI]', '000e3483 MOV EBX,dword ptr [EDI]', '000e34f5 MOV dword ptr [EDI + 0x4],EBX'],
 'terrain-overlay-resource-refs/instructions-0001d770.txt': ['0001db7a MOV EBX,0x115bec', '0001db91 CMP EDX,0xa0', '0001dc79 MOV EBX,0x18cf48', '0001dc7e MOV EDX,0x10d3c8', '0001dc88 CALL 0x000154f0', '0001ddd9 MOV dword ptr [EBX + -0x4],ECX'],
}
listings=[]
for name, expected in contracts.items():
    raw=(G/name).read_bytes()
    for line in expected:
        assert line in raw.decode(), (name,line)
    listings.append({'private_path':'private/ghidra/'+name,'sha256':sha(raw)})

def geometry(vertices, clip_height):
    """Geometry-only modes never compute horizontal attribute-gradient width."""
    if not 1 <= clip_height <= 480 or any(any(n not in range(-4096,4097) for n in p[:2]) for p in vertices):
        raise ValueError('safe geometry admission')
    c=classify(vertices)
    if c is None:return []
    kind,t,m,b=c
    h=b[1]-t[1]
    if b[1]<=0 or t[1]>=clip_height:return []
    if kind=='flat_top':
        l0,r0=t[0]<<16,m[0]<<16
        sl,sr=slope(t[0],b[0],h),slope(m[0],b[0],h)
        middle=h;ll,lr=sl,sr
    elif kind=='flat_bottom':
        l0=r0=t[0]<<16
        sl,sr=slope(t[0],m[0],h),slope(t[0],b[0],h)
        middle=h;ll,lr=sl,sr
    else:
        middle=m[1]-t[1]
        long=slope(t[0],b[0],h);upper=slope(t[0],m[0],middle);lower=slope(m[0],b[0],h-middle)
        if kind=='right_middle':
            if upper<=long:return []
            sl,sr,ll,lr=long,upper,long,lower
        else:
            if long<=upper:return []
            sl,sr,ll,lr=upper,long,lower,long
        l0=r0=t[0]<<16
    rows=[]
    for y in range(max(t[1],0),min(b[1],clip_height)):
        k=y-t[1];a=min(k,middle);z=max(k-middle,0)
        left=i32(l0+sl*a+ll*z);right=i32(r0+sr*a+lr*z)
        if k>=middle and kind=='right_middle':right=i32((m[0]<<16)+lr*z)
        if k>=middle and kind=='left_middle':left=i32((m[0]<<16)+ll*z)
        rows.append([y,left,right])
    return rows

for name,v,h in fixtures:
    assert geometry(v,h)==[r[:3] for r in prepare(v,h)['rows']]
zero_width=[[0,0,0,0,0],[1,1,1,1,1],[1,3,2,2,2]]
assert geometry(zero_width,4)==[[0,0,0],[1,21845,65536],[2,43690,65536]]

def coverage(width,left,right):
    l,r=left>>16,right>>16
    if width not in range(1,641) or not -4096<=l<=r<=4096:
        raise ValueError('positivewidth and orderedboundededges required')
    return max(l,0),max(max(l,0),min(r,width))

def mode6(source,base,fade,width,left,right,attrs,steps):
    if len(fade)!=16384:raise ValueError('fade size')
    start,end=coverage(width,left,right)
    if start==end:return {'start':start,'pixels':[],'mask':[]}
    skipped=max(-(left>>16),0)
    values=[i32(a+skipped*d)&0xffffff for a,d in zip(attrs,steps)]
    pixels=[];mask=[]
    for _ in range(start,end):
        u,v,s=[(a>>16)&255 for a in values]
        offset=base+v*256+u
        if not 0<=offset<len(source):raise ValueError('source bounds')
        texel=source[offset]
        if texel:
            if s>=64:raise ValueError('accessed fade row')
            pixels.append(fade[s*256+texel]);mask.append(1)
        else:
            pixels.append(0);mask.append(0)
        values=[(a+d)&0xffffff for a,d in zip(values,steps)]
    return {'start':start,'pixels':pixels,'mask':mask}

# Derive independent24-bit stepping from the original ADD/ADC register chain.
def chain(attrs,steps):
    u,v,s=[a&0xffffff for a in attrs]
    edx=((v&0xffff)<<16)|(u&0xffff)
    ecx=((s&0xffff)<<16)|0x1234
    ui,vi,si=u>>16,v>>16,s>>16
    total=(edx&0xffff)+(steps[0]&0xffff)
    edx=(edx&0xffff0000)|(total&0xffff)
    ui=(ui+((steps[0]>>16)&255)+(total>>16))&255
    total=edx+((steps[1]<<16)&0xffffffff)
    edx=total&0xffffffff
    vi=(vi+((steps[1]>>16)&255)+(total>>32))&255
    total=ecx+((steps[2]<<16)&0xffffffff)
    ecx=total&0xffffffff
    si=(si+((steps[2]>>16)&255)+(total>>32))&255
    return [(ui<<16)|(edx&0xffff),(vi<<16)|(edx>>16),(si<<16)|(ecx>>16)]

carry_checks=0
for a in [0,1,0xffff,0x10000,0x12ffff,0xffffff,0x80000000,0x7fffffff]:
    for d in [0,1,-1,0xffff,0x10000,-0x10000,0x7fffffff,-0x80000000]:
        for salt in range(17):
            attrs=[i32(a),i32(a+salt*0x16789),i32(a-salt*0x54321)]
            steps=[d,i32(d+salt*0x12345),i32(d-salt*0x4321)]
            assert chain(attrs,steps)==[(x+y)&0xffffff for x,y in zip(attrs,steps)]
            carry_checks+=1

source=bytes((3*(i&255)+5*(i>>8))&255 for i in range(65536))
fade=bytes((index+7*shade)&255 for shade in range(64) for index in range(256))
vectors=[]
for name,width,left,right,attrs,steps in [
 ('opaque_zero_skip',8,0,8<<16,[0,0,32<<16],[1<<16,0,0]),
 ('negative_clip',8,-3<<16,5<<16,[0x18000,0x24000,0x208000],[0x8000,0x4000,-0x1000]),
 ('negative_steps',8,0,8<<16,[0x68000,0x74000,0x208000],[-0x8000,-0x4000,-0x1000]),
 ('retained_shade_fraction',8,0,8<<16,[0x18000,0x24000,0x20fffe],[0,0,1]),
]:
    vectors.append({'name':name,'clip_width':width,'left_fixed':left,'right_fixed':right,'attrs_u_v_s':attrs,'steps_du_dv_ds':steps,'output':mode6(source,0,fade,width,left,right,attrs,steps)})
assert mode6(bytes(65536),0,fade,1,0,1<<16,[0,0,255<<16],[0,0,0])['mask']==[0]

atlas=(P/'private/integration/rnc-decoded/481677e31b05d9e4ba428932a4eaf9003fae4fe320f90daf22a947cbd3e4ef9c.bin').read_bytes()
actual_fade=(P/'private/integration/rnc-decoded/ddef38a3e63fbb3680aac15569f12e02f141036ffa270ddd48cdc82bce361503.bin').read_bytes()
gla=(P/'private/integration/rnc-decoded/c5e12a5b4325c8302cab0fb3ba3aaa654e2e044131763298815766d8a6f47a58.bin').read_bytes()
assert len(atlas)==163840 and len(actual_fade)==16384 and len(gla)==65536
assert sha(gla)=='bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3'
private=G/'terrain-overlay';actual=[]
for tile in [157,158,159]:
    assert 0x115bec+tile*4==0x115e60+(tile-157)*4
    base=(tile//8)*8192+(tile%8)*32
    output=bytearray();mask=bytearray()
    for row in range(32):
        r=mode6(atlas,base,actual_fade,32,0,32<<16,[0,row<<16,32<<16],[1<<16,0,0])
        output.extend(r['pixels']);mask.extend(r['mask'])
    name=f'mode6-tile-{tile}.bin';(private/name).write_bytes(output+mask)
    actual.append({'tile':tile,'texture_base':base,'shade_row':32,'opaque':sum(mask),'pixels_sha256':sha(output),'mask_sha256':sha(mask),'private_output':'private/ghidra/terrain-overlay/'+name})
gla_vectors=[]
for color in [0,97,255]:
    output=bytes(gla[d*256+color] for d in range(256))
    name=f'mode15-color-{color}.bin';(private/name).write_bytes(output)
    gla_vectors.append({'color':color,'destination_input':'0..255','output_sha256':sha(output),'private_output':'private/ghidra/terrain-overlay/'+name})
mapped=(G/'validated/memory-.object3.bin').read_bytes()
assert mapped[0x10d3c8-0x100000:0x10d3c8-0x100000+14]==b'data/main.gla\0'
receipt={'candidate_sha256':candidate,'instruction_contracts':listings,'dispatch_targets':targets,
 'mode0':'Geometry-only rows; fillcoveredspanwithunsignedcolorbyte. Stripcolor=0.',
 'mode15':'Samegeometry-onlyrows; output=MAIN.GLA[existingdestination*256+unsignedcolorbyte], alwayswrite. Stripcolor=0.',
 'mode6':'Same20Bpreparedrows/gradientsasmode5; independent24-bit U,V,S fixedpoint (16fraction+8integer), eachsamplefollowedbywrappingaddition. Sourcezero skipsFADElookup/write; nonzero outputFADE[(S>>16&255)*256+source].',
 'admission':'Coordinates-4096..4096/clipheight1..480 aspriortriangle; spanclipwidth1..640; orderedintegerleft<=right within-4096..4096; exactFADE16384/GLA65536; alltextureaddresseschecked; shade<64onlywhenFADEactuallyread.',
 'mode0_15_geometry':'Samecyclicorder/slopeculling/shortedge-reset/verticalcoverageaspriortriangle; noattributesorroundedhorizontalgradientwidthcalculation. Generalroundedwidth0isadmittedhere, unlikeunsafeattribute-prep.',
 'zero_gradient_width_geometry_vector':{'vertices':zero_width,'clip_height':4,'rows':geometry(zero_width,4)},
 'mode6_synthetic_vectors':vectors,'independent24bit_chain_checks':carry_checks,
 'mode6_actual_fixtures':actual,'mode15_actual_fixtures':gla_vectors,
 'overlay_pointer_producer':{'function':'0x1d770','table_base':'0x115bec','overlay_table':'0x115e60','atlas_tiles':[157,158,159],'formula':'base+(tile//8)*8192+(tile%8)*32','source_entry':'root/2/disc/40','source_sha256':'481677e31b05d9e4ba428932a4eaf9003fae4fe320f90daf22a947cbd3e4ef9c','decoded_sha256':sha(atlas)},
 'mode15_resource':{'entry_id':'root/2/disc/107','path':'GAME/DATA/MAIN.GLA','source_size':34551,'source_sha256':'c5e12a5b4325c8302cab0fb3ba3aaa654e2e044131763298815766d8a6f47a58','decoded_size':len(gla),'decoded_sha256':sha(gla),'load_call':'0x1dc88','destination':'0x18cf48'},
 'limits':['Explicitcallerinputs; no startupcamera/factor choices.','Negativezero-width originalunroll behavior is excluded bypositiveclipwidth admission.','Mode6 masks must preserveexistingdestination onzeros; no arbitrary transparency color.']}
(P/'research/ghidra/terrain-modes-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS {len(listings)} contracts; {carry_checks} mode6 carry-chain checks; 4 synthetic spans; 3 exactoverlay tiles; 3 fullMAIN.GLA columns; geometry-onlyzero-width case')
