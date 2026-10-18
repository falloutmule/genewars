"""Pin normal half-height indexed8 shadow and compare cached vs stream models."""
from pathlib import Path
import hashlib
import json
import struct

P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
plant=json.loads((P/'research/ghidra/plant-color-evidence.json').read_text())
candidate=plant['candidate_sha256'];assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
contracts=[]
def pin(name,needles):
    b=(G/name).read_bytes()
    for n in needles:assert n in b.decode(),(name,n)
    contracts.append({'private_path':'private/ghidra/'+name,'sha256':sha(b)})
pin('entity-plant/instructions-00096ac0.txt',['00096c0d SAR EDX,0x1f','00096c10 SUB EAX,EDX','00096c12 SAR EAX,0x1','00096c16 LEA EDX,[EAX + EBP*0x1]','00096c2b ADD EAX,dword ptr [ESP + 0x4]','00096c2f CALL 0x00092f80'])
pin('entity-shadow-panel/instructions-00092f80.txt',['00092f99 SHR BL,0x1','00092fa9 CALL 0x0008f660','00092fbb ADD BH,BH','00092fbf AND AL,0x1','0009321e ADD EBX,EBX','00093230 JZ 0x000933c4','00093347 SHL EDI,0x8','0009334a MOV BH,byte ptr [EDI + ESI*0x1 + 0x18cf48]','00093351 MOV byte ptr [EDX],BH','00093390 XOR CL,0x1','000933cd MOVSX EBX,byte ptr [EAX]'])
pin('display-geometry/instructions-0008f660.txt',['0008f6e2 MOV DL,byte ptr [ECX + 0x5]','0008f7c4 SUB DL,CH','0008f7e1 MOV byte ptr [ESI + 0x4],DL','0008f8d0 CMP byte ptr [0x0010aa15],0x0','0008f8e2 MOV EAX,EBP'])
pin('terrain-overlay-resource-refs/instructions-0001d770.txt',['0001dc79 MOV EBX,0x18cf48','0001dc7e MOV EDX,0x10d3c8','0001dc88 CALL 0x000154f0'])
modes=json.loads((P/'research/ghidra/terrain-modes-evidence.json').read_text())
resource=modes['mode15_resource']
gla=(P/f"private/integration/rnc-decoded/{resource['source_sha256']}.bin").read_bytes()
assert len(gla)==65536 and sha(gla)==resource['decoded_sha256']
mem=(G/'validated/memory-.object3.bin').read_bytes();off=0xd3c8
assert mem[off:mem.index(0,off)].lower()==b'data/main.gla'

def cached_shadow(mask,sw,sh,x,y,w,h,color,table):
    assert len(mask)==sw*sh and 0<=color<=255 and len(table)==65536
    result=bytearray((i*37+11)&255 for i in range(w*h))
    for sy in range(sh//2):
        dy=y+sy
        if not 0<=dy<h:continue
        for sx in range(sw):
            dx=x+sx
            if 0<=dx<w and mask[(2*sy+1)*sw+sx]:
                p=dy*w+dx;result[p]=table[color*256+result[p]]
    return bytes(result)

def literal_shadow(data,start,sw,sh,x,y,w,h,color,table):
    # Separate source-control traversal: first skip every even row, then apply
    # the opaque positive-run footprint of the following row. Ignore literals.
    result=bytearray((i*37+11)&255 for i in range(w*h));p=start
    for row in range(2*(sh//2)):
        sx=0;dy=y+row//2
        while True:
            n=struct.unpack_from('<b',data,p)[0];p+=1
            if not n:break
            if n<0:sx-=n
            else:
                if row&1 and 0<=dy<h:
                    for k in range(n):
                        dx=x+sx+k
                        if 0<=dx<w:
                            q=dy*w+dx;result[q]=table[color*256+result[q]]
                p+=n;sx+=n
            assert sx<=sw
    return bytes(result)

fixtures=[];checks=0
for r in plant['resources']:
    b=(P/f"private/integration/rnc-decoded/{r['input_sha256']}.bin").read_bytes()
    tab=struct.unpack_from('<I',b,2)[0]
    rel=struct.unpack_from('<I',b,tab+6*r['sprite_record'])[0]
    mask=(G/f"entity-plant/{Path(r['source_name']).stem}-animation0-frame0.mask").read_bytes()
    assert sha(mask)==r['opacity_sha256']
    sw,sh=r['width'],r['height']
    for w,h in [(1,1),(7,5),(64,48)]:
        for x,y in [(-sw,-sh),(-sw+1,-sh//2),(-3,-2),(0,0),(1,1),(w-1,h-1),(w,h)]:
            for color in [0,97,255]:
                a=cached_shadow(mask,sw,sh,x,y,w,h,color,gla)
                assert a==literal_shadow(b,18+rel,sw,sh,x,y,w,h,color,gla)
                checks+=1
    output=cached_shadow(mask,sw,sh,3,4,64,48,0,gla)
    path=G/f"entity-plant/{Path(r['source_name']).stem}-shadow0.bin";path.write_bytes(output)
    fixtures.append({'slot':r['slot'],'width':sw,'source_height':sh,'shadow_height':sh//2,'source_mask_sha256':sha(mask),'destination':'64x48 pattern(i*37+11)mod256','placement':[3,4],'color':0,'output_sha256':sha(output),'private_path':'private/ghidra/entity-plant/'+path.name})
# Odd height and opaque zero literals distinguish every-second-row semantics.
synthetic=bytes([1,99,0,1,0,0,1,88,0,1,77,0,1,66,0])
syntheticmask=bytes([1]*5)
asymmetric=bytes(((c*3+d*5)&255) for c in range(256) for d in range(256))
assert cached_shadow(syntheticmask,1,5,0,0,2,3,97,asymmetric)==literal_shadow(synthetic,0,1,5,0,0,2,3,97,asymmetric)
vectors=[{'component_y':n,'half_y_trunc0':abs(n)//2*(-1 if n<0 else 1)} for n in [-32768,-32767,-3,-2,-1,0,1,2,3,32767]]
receipt={'candidate_sha256':candidate,'instruction_contracts':contracts,'resource':resource,
 'admission':'Normal context flags0/component flags0/96AC0 extra argument0; explicit signed16 frame origin, positive bounded clip dimensions/pitch and enabled8F660 drawing (10AA15 nonzero). Destination and decoded frame validated before writes.',
 'frame_component_placement':'Shadow x=origin_i16_x+component_signed_x; shadow y=origin_i16_y+truncate_toward_zero(component_signed_y/2). Full signed32 additions here; unlike foreground, no final signed16 narrowing in96AC0 normal path.',
 'raster':'Descriptor height is floor(source_height/2). Output row k reads only opacity of original row2*k+1; even rows are consumed but not written. Odd final source row is discarded. Top clipping by k shadow rows consumes2*k source rows and retains the same odd-row phase. Horizontal clipping never changes source row selection.',
 'sample':'For every positive-run pixel, destination=MAIN.GLA[(color_u8<<8)+existing_destination_u8]; source literal bytes are consumed but their values ignored. Negative runs preserve destination. Literal0 remains opaque. Color0 supplied by6C230; explicitcolor0..255 sample API also supported by92F80.',
 'coverage':'Half-open rectangle intersection of shadow sprite with explicit clip. No pixel outside intersection; fully clipped or zero halfheight safely empty. Safe zero clip width/height accepted as empty without asserting equivalence to original zero-dimension behavior.',
 'source_side_effects':'8F660 also marks dirty-block metadata before returning a draw pointer. This pure pixel contract excludes dirty bookkeeping and requires its drawing gate enabled.',
 'component_half_vectors':vectors,'cached_vs_stream_cases':checks+1,'actual_fixtures':fixtures,
 'limits':['No flip/scale/component flags, extra argument1 or flash/context transforms admitted.','No source literal colors or palette colors inferred from shadow result.','Separate from entity visibility, ordering and animation.']}
(P/'research/ghidra/plant-shadow-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS4pins; {checks+1} cached/stream clipped shadow comparisons; 12actualGLA shadow fixtures')
