"""Validate the literal packed-carry mode-five span with explicit prepared inputs."""
from pathlib import Path
import hashlib
import json
import struct

P=Path(__file__).resolve().parents[2]
G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
obj=(G/'validated/memory-.object1.bin').read_bytes()
assert obj[0xe3fab-0x10000:0xe3fb1-0x10000]==bytes.fromhex('83fd00730149')
raw=(G/'terrain-mode5.txt').read_bytes()
for line in ['000e3fa3 MOV EDX,dword ptr [ESP + 0x30]','000e3fa7 MOV EBP,dword ptr [ESP + 0x3c]',
 '000e3fab CMP EBP,0x0','000e3fae JNC 0x000e3fb1','000e3fb0 DEC ECX',
 '000e3fb1 ROL ECX,0x10','000e3fb4 ROL EDX,0x10','000e3fb7 SHR EBP,0x8',
 '000e3fdb SAR EAX,0x10','000e3fde SAR EBP,0x10','000e3ff9 NEG EAX',
 '000e3fff IMUL ECX,EAX','000e4002 ADD ECX,dword ptr [ESI + 0x8]',
 '000e4049 SUB EBP,EAX','000e404b JLE 0x000e429b','000e406e XOR DH,DH',
 '000e4070 AND EBX,0xffff','000e40d0 MOV AH,CH','000e40d2 MOV BL,DL',
 '000e40d4 ADD ECX,dword ptr [ESP + 0x4c]','000e40d8 MOV AL,byte ptr [EBX + ESI*0x1]',
 '000e40db ADC EDX,dword ptr [ESP + 0x50]','000e40df MOV AL,byte ptr [EAX + 0x188f48]',
 '000e40e5 ADC BH,byte ptr [ESP + 0x64]','000e40e9 MOV byte ptr [EDI],AL']:
 assert line in raw.decode(),line
u32=lambda x:x&0xffffffff
rol16=lambda x:u32((u32(x)<<16)|(u32(x)>>16))
def addc(a,b,carry=0):
 total=a+b+carry
 return u32(total),int(total>0xffffffff)

def span(source,base,fade,width,left,right,attrs,steps):
    assert 0<=width<=(1<<31)-1 and len(fade)==16384
    l,r=left>>16,right>>16
    start,end=max(l,0),min(r,width)
    if end<=start:
        return {'start':start,'pixels':b'','samples':[],'final_packed':None}
    skip=max(-l,0)
    u,v,s=[u32(a+skip*d) for a,d in zip(attrs,steps)]
    du,dv,ds=map(u32,steps)
    c=(rol16(u)&0xffff0000)|((s>>8)&0xffff)
    d=(rol16(v)&0xffff0000)|((u>>16)&255)
    vb=(v>>16)&255
    gc=(rol16(du)&0xffff0000)|((ds>>8)&0xffff)
    gd=(rol16(dv)&0xffffff00)|((du>>16)&255)
    gv=(dv>>16)&255
    pixels=bytearray()
    samples=[]
    for x in range(start,end):
        tx,ty,shade=d&255,vb,(c>>8)&255
        address=base+ty*256+tx
        if address<0 or address>=len(source):raise ValueError('source_out_of_bounds')
        if shade>=64:raise ValueError('shade_out_of_bounds')
        index=source[address]
        pixels.append(fade[shade*256+index])
        samples.append([x,tx,ty,shade])
        c,carry=addc(c,gc)
        d,carry=addc(d,gd,carry)
        vb=(vb+gv+carry)&255
    return {'start':start,'pixels':bytes(pixels),'samples':samples,'final_packed':{'c':c,'d':d,'v_byte':vb}}

def instruction_state_step(c,d,vb,gc,gd,gv):
    # Independent register-width sequence, including original carry propagation.
    value=c+gc
    cf=value>>32
    c=value&0xffffffff
    value=d+gd+cf
    cf=value>>32
    d=value&0xffffffff
    vb=(vb+gv+cf)&255
    return c,d,vb
checks=0
for c in [0,1,0xffff,0x10000,0xffff0000,0xffffffff]:
 for d in [0,1,0xffff,0x10000,0xffff0000,0xffffffff]:
  for gc in [0,1,0xffff,0x10000,0xffff0000,0xffffffff]:
   for gd in [0,1,0xffff,0x10000,0xffff0000,0xffffffff]:
    for vb,gv in [(0,0),(255,0),(0,255),(255,255)]:
     c1,cf=addc(c,gc);d1,cf=addc(d,gd,cf);v1=(vb+gv+cf)&255
     assert (c1,d1,v1)==instruction_state_step(c,d,vb,gc,gd,gv)
     checks+=1
source=bytes((3*x+5*y)&255 for y in range(256) for x in range(256))
fade=bytes((i+7*s)&255 for s in range(64) for i in range(256))
cases=[
 ('positive_steps',16,0,8<<16,[0x28000,0x44000,0xa8000],[0x12000,0x8000,0]),
 ('left_fraction_clipped',6,-0x28000,0x5c000,[0x8000,0x10000,0x48000],[0x8000,0x10000,0x4000]),
 ('right_clipped_negative_u',8,0x28000,0xcffff,[0x1f8000,0x20000,0x140000],[-0x10000,0x4000,0]),
 ('negative_shade_carry',8,0,5<<16,[0x2ffff,0x44000,0xa8000],[0,0,-0x8000]),
 ('byte_coordinate_wrap',8,0,5<<16,[0xffc000,0xff8000,0xa0000],[0x8000,0xc000,0]),
 ('subpixel_empty',8,0x4000,0xc000,[0,0,64<<16],[0,0,0]),
 ('unroll_residue_19',32,0,19<<16,[0x10000,0x20000,0x30000],[0x8000,0x4000,0]),
 ('negative_v',8,0,6<<16,[0x20000,0x48000,0xc0000],[0,-0x4000,0]),
]
vectors=[]
for name,width,left,right,attrs,steps in cases:
 result=span(source,0,fade,width,left,right,attrs,steps)
 vectors.append({'name':name,'input':{'clip_width':width,'left_16_16':left,'right_16_16':right,'attributes_u_v_s':attrs,'gradients_du_dv_ds':steps,'texture_base':0},
                 'output':{'start_x':result['start'],'pixels':list(result['pixels']),'samples_x_u_v_shade':result['samples'],'final_packed':result['final_packed']}})
assert vectors[3]['output']['samples_x_u_v_shade'][1][1]==3
for bad_source,bad_shade,expected in [(bytes(1),0x20000,'source_out_of_bounds'),(source,64<<16,'shade_out_of_bounds')]:
 try:span(bad_source,0,fade,8,0,1<<16,[0x10000,0,bad_shade],[0,0,0])
 except ValueError as e:assert str(e)==expected
 else:raise AssertionError('Expected safe rejection')

# Verify the 16-way unroll's pointer placement without assuming a generic loop.
offsets=list(struct.unpack_from('<16i',obj,0xe3f53-0x10000))
assert offsets==[0]+list(range(-15,0))
jumps=list(struct.unpack_from('<16I',obj,0xe4090-0x10000))
assert jumps[0]==0xe40d0 and jumps[1]==0xe4273
unroll_checks=0
for count in range(1,641):
 rem=count&15
 dest=offsets[rem]
 lane=0 if rem==0 else 16-rem
 remaining=count
 writes=[]
 while True:
  writes.extend(dest+i for i in range(lane,16))
  dest+=16;remaining-=16
  if remaining<=0:break
  lane=0
 assert writes==list(range(count))
 unroll_checks+=1
results=json.loads((P/'research/integration/rnc-corpus.json').read_text())['results']
ar=next(r for r in results if r['entry_id']=='root/2/disc/40' and r['status']=='passed')
fr=next(r for r in results if r['entry_id']=='root/2/disc/61' and r['status']=='passed')
atlas=(P/ar['private_output']).read_bytes();actual_fade=(P/fr['private_output']).read_bytes()
assert sha(atlas)==ar['output_sha256'] and sha(actual_fade)==fr['output_sha256']
actual=[]
private=G/'terrain-span';private.mkdir(exist_ok=True)
for tile in [0,7,8,159]:
 base=(tile//8)*8192+(tile%8)*32
 attrs=[0x18000,0x24000,0x208000];steps=[0x8000,0,0]
 result=span(atlas,base,actual_fade,16,2<<16,10<<16,attrs,steps)
 name=f'tile-{tile:03}.bin';(private/name).write_bytes(result['pixels'])
 actual.append({'tile':tile,'texture_base':base,'clip_width':16,'left_16_16':2<<16,'right_16_16':10<<16,'attributes_u_v_s':attrs,'gradients_du_dv_ds':steps,'start_x':result['start'],'pixel_count':len(result['pixels']),'output_sha256':sha(result['pixels']),'private_output':'private/ghidra/terrain-span/'+name})
receipt={'candidate_sha256':candidate,'instruction_contracts':[{'private_path':'private/ghidra/terrain-mode5.txt','sha256':sha(raw)}],
 'scope':'One prepared mode-five horizontal span; triangle edge/gradient preparation remains separate.',
 'coverage':'For positive clip_width: [max(left_16_16>>16,0), min(right_16_16>>16,clip_width)); right exclusive; no fractional ceil',
 'zero_width_safety_deviation':'Safe zero-width returns empty; original negative-left path can enter16-lane unroll with count0 after clampingright to0. No source-equivalence claim for zero-width.',
 'clip':'When integer left is negative, advance full32 attributes by -left*gradient with wrapping32 before packing.',
 'packing':{'C':'(rol16(U)&0xFFFF0000)|((S_unsigned>>8)&0xFFFF)','D':'(rol16(V)&0xFFFF0000)|((U_unsigned>>16)&255)','v_byte':'(V_unsigned>>16)&255','gradient_C':'(rol16(dU)&0xFFFF0000)|((dS_unsigned>>8)&0xFFFF)','gradient_D':'(rol16(dV)&0xFFFFFF00)|((dU_unsigned>>16)&255)','gradient_v_byte':'(dV_unsigned>>16)&255'},
 'sample':'u=D&255; v=v_byte; shade=(C>>8)&255; source[texture_base+v*256+u]; fade[shade*256+source_index]; always emit',
 'advance':'C=ADD32(C,gradient_C); D=ADC32(D,gradient_D,carry_C); v_byte=ADC8(v_byte,gradient_v_byte,carry_D)',
 'quirk':'CMP dS,0 / JNC always skips DEC gradient-U; preserve literal bytes83FD00730149, including negative shade gradients.',
 'synthetic_source':'65536 bytes: source[y*256+x]=(3*x+5*y)&255','synthetic_fade':'16384 bytes: fade[shade*256+index]=(index+7*shade)&255',
 'synthetic_vectors':vectors,'carry_checks':checks,'unroll_count_checks':unroll_checks,
 'actual_resource_fixtures':actual,'atlas_decoded_sha256':sha(atlas),'fade_decoded_sha256':sha(actual_fade),
 'safe_rejections':['FADE length must equal16384; shade byte>=64 rejects rather than wrapping/clamping.','Every full-source address must be in bounds; do not clamp u/v to32 or compact tile extent.','Compute and validate an output span before applying writes to avoid partial mutation on a rejected sample.'],
 'limits':['These are precomputed span inputs, not yet a triangle API.','Packed carry coupling and shade low8 truncation must not be replaced by independent fixed-point attribute updates.','Original vertical row setup/coverage and triangle culling remain separate evidence.']}
(P/'research/ghidra/mode5-span-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS literal mode5 contract; {checks} packed-carry cases; {unroll_checks} unroll widths; 8 synthetic spans; 4 actual atlas/FADE fixtures')
