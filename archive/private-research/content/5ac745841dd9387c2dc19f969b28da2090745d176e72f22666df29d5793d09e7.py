"""Hash-bound family-1 source mapping, resource selection and normal color pass.

No original executable is run. Actual decoded pixels stay under private/ghidra.
"""
from pathlib import Path
import hashlib
import json
import struct

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
sha = lambda b: hashlib.sha256(b).hexdigest()
candidate = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes()) == candidate
contracts = []
def pin(name, needles=()):
    b = (G/name).read_bytes()
    for needle in needles:
        assert needle in b.decode(), (name, needle)
    contracts.append({'private_path':'private/ghidra/'+name,'sha256':sha(b)})

pin('mission-tail-readers/instructions-0006d490.txt', ['0006d494 MOV ECX,0x1f','0006d4bc ADD dword ptr [EDX],0x1f'])
pin('mission-tail-readers/instructions-0006c950.txt', ['0006c95e MOV ECX,0xa','0006c963 LEA EDI,[EBX + 0x27]'])
pin('mission-tail-constructors/instructions-0001e3e0.txt', ['0001e3e0 MOV dword ptr [EAX + 0x23],0x1120ac'])
pin('mission-tail-init/instructions-0006c690.txt', ['0006c691 MOVSX EDX,byte ptr [EAX + 0x2b]','0006c69c MOV EDX,dword ptr [EDX*0x4 + 0x1168c4]'])
pin('animal-consumers/instructions-0001d770.txt', ['0001dfd0 MOV EAX,dword ptr [EDX + 0x116894]','0001dfd9 CALL 0x00098a60','0001dfde MOV dword ptr [EDX + 0x1168c0],EAX','0001dfe4 CMP EDX,0x30'])
pin('sprite-functions/instructions-00098a60.txt', ['00098a88 LEA EBX,[EAX + 0x12]','00098ab7 SAR BX,0x1','00098ac2 SAR DI,0x1','00098b55 MOV EAX,dword ptr [ECX + 0xe]'])
pin('terrain-viewport/instructions-0001f290.txt', ['0001f2bb MOV EBX,dword ptr [EDI]','0001f2bf MOV EDI,dword ptr [EAX + 0x23]','0001f2cd CALL dword ptr [EDI + 0x18]'])
pin('entity-plant/instructions-0006c230.txt', ['0006c23e AND AL,0x2','0006c252 MOV AH,byte ptr [ESI + 0x2e]','0006c25c TEST AH,0x2','0006c3cf CALL 0x00096ac0','0006c3db TEST DL,0x10','0006c448 TEST byte ptr [EAX + 0x15],0x2','0006c457 CMP EAX,0x1e','0006c468 CALL 0x00097a90','0006c47e MOV ECX,0x188f48','0006c494 CALL 0x00097ca0'])
pin('entity-plant/instructions-00097a90.txt', ['00097b6d ADD AX,word ptr [ECX + 0x6]','00097b75 MOVSX EDX,AX','00097b81 CWDE','00097b85 CALL 0x0009a660'])
pin('entity-plant/instructions-00097ca0.txt', ['CALL 0x0009b0d0'])
pin('entity-plant/instructions-0009a660.txt', ['0009af73 MOVSX EAX,byte ptr [EDX]','0009afcb INC EDX'])
pin('entity-plant/instructions-0009b0d0.txt', ['0009bb71 MOVZX EDI,byte ptr [EDI]','0009bb78 MOV CH,byte ptr [EDI + ESI*0x1]'])
pin('entity-plant/instructions-00096ac0.txt', ['00096c12 SAR EAX,0x1','00096c2f CALL 0x00092f80','00096cad RET 0x4'])

mem = (G/'validated/memory-.object3.bin').read_bytes()
u32 = lambda va: struct.unpack_from('<I',mem,va-0x100000)[0]
assert u32(0x1120ac+0x18)==0x6c230 and u32(0x1120ac+0x24)==0x1f290
assert u32(0x1120ac+0x3c)==0x6c950 and u32(0x1120ac+0x60)==0x6c690
manifest = {r['entry_id']:r for r in json.loads((P/'research/game-files-manifest.json').read_text())['entries']}
sources = [
 (347,'grass1','c233be78010920286672e26dba1501947bbb3bff92ed4477045539c3996c6486'),
 (348,'grass2','52eb03c7076ccf67b2d33a33c75ff04bd07b8a97fedbf5b2c3ddace3efb0e9d3'),
 (349,'grass3','e856fcc0afca3091f50d49b0c79db29f9815804398594245d80576b21604c142'),
 (344,'desert1','79c0df3028c6553e5a2884644d0061e5b347cbe6183e8313e648daf1eba04984'),
 (345,'desert2','7310d57e5a6571017752c1bda1cf6fea0fd4cddc6919c9881d31c556a301193c'),
 (346,'desert3','caf8094dcd560dd4da88d9482fa91233bfa126296146bc45e3bfc1be8dbac209'),
 (350,'snow1','69c5b560bdd741ed1206ffbda67cac4a731183f5e9f21c501d187dd656e29044'),
 (351,'snow2','5d1e22180251f8d1a490817cb18574c3d9ddabdc90e27c40e7afd14ffb347a33'),
 (352,'snow3','7a978fb3b44444a11f3cbb60a7611c172550b64febf4712e9e46e0efeb48661a'),
 (341,'all1','df9180a4dace76f722637f6b35f00114ea6a183603fd57c32502957c37378faa'),
 (342,'all2','fd207e9bf58cc5c7320e278be60eb4924941da6c99d593883df31df48aef7170'),
 (343,'all3','3340bf97d82fe34731bbbcc65be884ec1168e34c14120df2b65e8b1c36f2bcf5')]

def indexed8(data,start,width,height,end):
    pixels=bytearray(width*height); mask=bytearray(width*height); p=start
    for y in range(height):
        x=0
        while True:
            assert p<end
            n=struct.unpack_from('<b',data,p)[0];p+=1
            if n==0:break
            if n<0:x-=n
            else:
                assert p+n<=end
                for k in range(n):
                    if x+k<width:pixels[y*width+x+k]=data[p+k];mask[y*width+x+k]=1
                p+=n;x+=n
            # All admitted actual normal frame rows fit. Do not generalize a
            # malformed-stream acceptance policy from the clipped blitter.
            assert x<=width
    return bytes(pixels),bytes(mask),p-start

rows=[]
for slot,(entry,name,input_sha) in enumerate(sources):
    m=manifest[f'root/2/disc/{entry}'];assert m['sha256']==input_sha
    descriptor=0x103b78+slot*44
    offset=descriptor-0x100000
    runtime_name=mem[offset:mem.index(0,offset)].decode('ascii')
    assert runtime_name.lower()==f'data/plant/{name}.anb'
    assert u32(descriptor+0x1c)==0x116894+slot*4
    b=(P/f'private/integration/rnc-decoded/{input_sha}.bin').read_bytes()
    count,tab,comp,frame,anim=struct.unpack_from('<H4I',b)
    assert count==9 and 18<=tab<=comp<=frame<=anim<=len(b) and anim+count*10==len(b)
    first,last,frames,aflags=struct.unpack_from('<IIBB',b,anim)
    assert first==0 and frames==1
    component_offset=struct.unpack_from('<I',b,frame+first)[0]
    ncomp=b[frame+first+11];assert ncomp==1
    sprite_rel,x,y,flags,pad=struct.unpack_from('<IhhBB',b,comp+component_offset)
    assert sprite_rel%6==0 and flags==0
    sprite,width,height=struct.unpack_from('<IBB',b,tab+sprite_rel)
    pixels,mask,consumed=indexed8(b,18+sprite,width,height,tab)
    out=G/f'entity-plant/{name}-animation0-frame0'
    out.with_suffix('.indices').write_bytes(pixels);out.with_suffix('.mask').write_bytes(mask)
    rows.append({'slot':slot,'entry_id':m['entry_id'],'source_name':runtime_name,'input_sha256':input_sha,'decoded_sha256':sha(b),'decoded_size':len(b),'descriptor_va':hex(descriptor),'raw_pointer_slot':hex(0x116894+slot*4),'animation_pointer_slot':hex(0x1168c4+slot*4),'animation_count':count,'animation':0,'animation_frame_count':frames,'frame_within_animation':0,'frame_offset':first,'component_count':ncomp,'component_flags':flags,'component_xy_after_sar1':[x>>1,y>>1],'sprite_record':sprite_rel//6,'width':width,'height':height,'consumed_rle_bytes':consumed,'indices_sha256':sha(pixels),'opacity_sha256':sha(mask)})

source=(G/'campaign-resources/152.dat').read_bytes()
assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
active=[]
for i in range(512):
    s=source[0x43d4e+i*41:0x43d4e+(i+1)*41]
    native=bytearray(49);native[:31]=s[:31];native[39:]=s[31:]
    assert native[7:11]==s[7:11] and native[0x2b]==s[0x23]
    if not s[6]:continue
    assert s[0x19]==s[0x1b]==0 and s[0x15]==4 and s[0x17]==0 and native[0x2e]==0
    active.append(s[0x23])
assert len(active)==40
dispatch=lambda flags,shade:'indexed8-direct' if flags&2 or shade>=30 else 'indexed8-fade'
dispatch_counts={}
for flags in range(256):
    for shade in range(256):
        mode=dispatch(flags,shade)
        literal='indexed8-direct' if flags&2 else ('indexed8-fade' if shade<0x1e else 'indexed8-direct')
        assert mode==literal
        dispatch_counts[mode]=dispatch_counts.get(mode,0)+1
vectors=[{'flags_at_0x15':f,'shade_at_0x17':s,'dispatch':dispatch(f,s),'fade_row':s if dispatch(f,s).endswith('fade') else None} for f,s in [(0,0),(0,29),(0,30),(0,255),(2,0),(4,0),(4,29),(4,30)]]
receipt={'candidate_sha256':candidate,'instruction_contracts':contracts,'numeric_family':1,
 'source_layout':{'file_base':'0x43d4e','records':512,'serialized_stride':41,'native_state_offset':'0x446ae','native_stride':49,'copies':[[0,0,31],[31,39,10]],'fields':{'position_i16x2':{'source':7,'native':7},'flags_u8':{'source':21,'native':21},'shade_u8':{'source':23,'native':23},'animation_u8':{'source':25,'native':25},'frame_u8':{'source':27,'native':27},'resource_selector_i8':{'source':35,'native':43},'jitter_flags_u8':{'source':38,'native':46}},'note':'6D490 copies the 31-byte header before mutating the staging source flags/shade; these mutations do not change the already copied native header.'},
 'resources':rows,'resource_selection':'native signed byte+0x2B -> table1168C4[selector]; safe admission0..11. Loader1DFD0 binds raw116894[slot] through98A60 to animation1168C4[slot].',
 'frame_resolution':'animations_base + u8(entity+0x19)*10; first frame pointer at animation+0; then u8(entity+0x1B)*14. Frame byte is relative to the selected animation first frame, not the entire frame section.',
 'admission':{'context_byte_16e0c4_clear_mask':2,'entity_byte_0x15_clear_mask':16,'entity_byte_0x2e_clear_mask':2,'component_flags':0,'resource_selector':[0,11],'animation_and_frame':'checked against resource/selected animation counts','coordinates':'explicit signed16 origin; signed16 wrapping after adding loader-halved component offsets'},
 'color':'positive signed control consumes that many literal index bytes, negative skips -control pixels, zero ends row; literal zero is opaque. Direct branch9A660 preserves indices; FADE branch9B0D0 maps all literal values through row shade*256. All admitted frame0 streams fit width and stop before TAB.',
 'dispatch_vectors':vectors,'dispatch_cases':65536,'dispatch_counts':dispatch_counts,
 'fade':'explicit FADE table; shade byte0..29 indexes row*256; shade30..255 or flags+0x15 mask0x02 bypasses FADE. Source flags+0x15 mask0x10 flashing excluded, distinct from direct mask0x02.',
 'shadow_request':{'before_color':True,'caller':'0x6c3a3..0x6c3cf','callee':'0x96ac0','arguments':{'EAX':'signed16 x origin','EDX':'signed16 y origin','EBX':'resolved same frame pointer','ECX':0,'stack_extra_dword':0},'component_normal_path':'x=originX+signed componentX; y=originY+truncate_toward_zero(componentY/2); CL0 to92F80. Shadow raster semantics separate; do not replace by foreground opacity.'},
 'conq001_fixture':{'entry_id':'root/2/disc/152','sha256':sha(source),'active_records':40,'resource_selector_counts':{str(i):active.count(i) for i in sorted(set(active))},'animation':0,'frame':0,'flags_at_0x15':4,'source_shade_at_0x17':0,'jitter_flags_at_native_0x2e':0},
 'limits':['Normal foreground only; shadow request must remain explicit until separately implemented.','No entity visibility, ordering, animation time, initial active state, or shade-update timing inferred.','No packed4 decoder here; direct literal-byte reads prove indexed8.','Cached full-frame decode may require all declared EOL terminators as a stricter safe-resource admission than clipped runtime early exit.']}
(P/'research/ghidra/plant-color-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('PASS 13 instruction contracts; 12 hash-bound plant resources/frame0 RLEs; 40 source mappings; 65536 dispatch cases')
