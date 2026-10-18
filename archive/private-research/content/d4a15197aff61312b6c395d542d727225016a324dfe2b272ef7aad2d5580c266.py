"""Validate strip-derived vertex/UV contracts with explicit camera and shade inputs."""
from pathlib import Path
import hashlib
import json
import struct

P=Path(__file__).resolve().parents[2]
G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
prior=json.loads((P/'research/ghidra/terrain-evidence.json').read_text())
assert sha((G/'GW-CD-candidate.exe').read_bytes())==prior['candidate_sha256']
contracts={
 'terrain-map-refs/instructions-0007b7f0.txt':[
 '0007b810 IMUL EAX,EDX','0007b828 ADD EAX,ECX','0007b83b MOV EDI,dword ptr [0x00184fdc]',
 '0007b841 SAR EAX,0x1','0007b843 ADD EDI,EAX','0007b84e MOV EDI,dword ptr [0x00184fe0]',
 '0007b858 SUB EDI,EAX','0007b860 TEST DL,0x1','0007b871 NEG EDI','0007b885 LEA EDI,[EAX + -0x1]',
 '0007b8f1 MOVSX EAX,word ptr [EBX + 0x2]','0007b8fb SAR EAX,0x6','0007b8fe IMUL EDX,EAX',
 '0007b910 MOVSX EAX,word ptr [EBX]','0007b915 SHL EAX,0x8','0007b918 MOV DL,byte ptr [EBX + 0xc]',
 '0007b91b IMUL EDX,EAX','0007b932 ADD EAX,0x8000',
 '0007bb19 MOV EAX,dword ptr [EDX + 0x10]','0007bb3f MOV EAX,dword ptr [EDX + 0x18]',
 '0007bb82 LEA EBX,[ESP + 0x58]','0007bb86 LEA EDX,[ESP + 0x44]','0007bb8a LEA EAX,[ESP + 0x30]',
 '0007bb8e CALL 0x000e1900','0007bd27 MOV EAX,dword ptr [EDX + 0x8]',
 '0007bd4e MOV EAX,dword ptr [EDX + 0x10]','0007bd9d CALL 0x000e1900'],
 'terrain-camera-refs/instructions-000829a0.txt':['000829aa MOVSX EAX,word ptr [EAX]','000829ad MOV [0x00184fdc],EAX','000829b2 MOVSX EAX,word ptr [EDI + 0x2]','000829b6 MOV [0x00184fe0],EAX'],
 'terrain-camera-refs/instructions-00082e20.txt':['00082e2d CALL 0x0007b140','00082e32 SAR EAX,0x6','00082e3f SAR EAX,0x5','00082e4b ADD AH,0xc','00082e5a ADD AL,0x2','00082e66 MOV word ptr [ESI + 0x2],AX','00082e6a MOV word ptr [ESI],DX'],
 'terrain-mode5.txt':['000e40d0 MOV AH,CH','000e40d8 MOV AL,byte ptr [EBX + ESI*0x1]','000e40df MOV AL,byte ptr [EAX + 0x188f48]','000e40e9 MOV byte ptr [EDI],AL'],
}
listings=[]
for path,lines in contracts.items():
 raw=(G/path).read_bytes()
 for line in lines:assert line in raw.decode(),(path,line)
 listings.append({'private_path':'private/ghidra/'+path,'sha256':sha(raw)})

def project(world,camera,height,top,anchor=(0,0)):
 dx,dy=world[0]-camera[0],world[1]-camera[1]
 return [anchor[0]+(dx-dy)*32,anchor[1]+(dx+dy)*16-2*(height>>6)-top]
heights=[-32768,-65,-64,-1,0,63,64,32767]
comparisons=0
parity_vectors=[]
for camera in [(-200,300),(0,0),(127,127),(255,255)]:
 for top in [-100,0,480]:
  for column in range(20):
   parity=column&1
   x0=camera[0]+(column>>1)
   y0=camera[1]-(column>>1)-1-parity
   for k in range(40):
    for h in heights:
     worlds=[(x0+k,y0+k),(x0+k,y0+k+1),(x0+k+1,y0+k+1),(x0+k+1,y0+k+2)]
     base_y=(-1-parity+2*k)*16-top
     screen_x=[(column+1)*32,column*32,(column+1)*32,column*32]
     for i,world in enumerate(worlds):
      strip=[screen_x[i],base_y+i*16-2*(h>>6)]
      assert project(world,camera,h,top)==strip
      comparisons+=1
     if camera==(127,127) and top==0 and column in [0,1] and k==0 and h==0:
      parity_vectors.append({'camera':list(camera),'column':column,'iteration':k,'world_vertices_A_B_C_D':[list(w) for w in worlds],
       'buffer_vertices_A_B_C_D':[project(w,camera,h,top) for w in worlds],
       'first_triangle_world':[list(worlds[i]) for i in [0,2,1]],'second_triangle_world':[list(worlds[i]) for i in [1,2,3]]})
projection_vectors=[]
for world,camera,h,top,anchor in [([0,0],[0,0],0,0,[0,0]),([1,0],[0,0],0,0,[0,0]),([1,1],[0,0],0,0,[0,0]),([0,1],[0,0],0,0,[0,0]),([128,127],[127,127],64,7,[0,0]),([4,3],[1,1],65,7,[100,200]),([-1,0],[0,0],-1,0,[0,0]),([0,0],[0,0],-65,0,[0,0])]:
 projection_vectors.append({'world_cell':world,'camera_cell':camera,'height_i16':h,'vertical_top':top,'caller_anchor':anchor,'position':project(world,camera,h,top,anchor)})
uv=prior['uv_orientation_table']['corners_u16_16']
triangles=[[0,2,3],[0,1,2]]
uv_vectors=[]
for orientation,values in enumerate(uv):
 corners=[values[i:i+2] for i in range(0,8,2)]
 uv_vectors.append({'orientation':orientation,'corners_00_10_11_01':corners,'triangles':[[corners[i] for i in tri] for tri in triangles]})
shade_vectors=[{'light_i16':light,'factor_u8':factor,'shade_16_16':4*light*factor+0x8000}
               for light,factor in [(0,0),(8192,0),(8192,64),(3840,64),(8192,255),(-1,64),(-32768,255),(32767,255)]]
for light in range(-32768,32768):
 for factor in [0,1,64,255]:
  intermediate=light*256*factor
  assert -(1<<31)<=intermediate<(1<<31)
  assert intermediate%64==0 and intermediate//64+32768==4*light*factor+32768
results=json.loads((P/'research/integration/rnc-corpus.json').read_text())['results']
fade_result=next(r for r in results if r['entry_id']=='root/2/disc/61' and r['status']=='passed')
fade=(P/fade_result['private_output']).read_bytes()
assert len(fade)==64*256 and sha(fade)==fade_result['output_sha256']
fade_rows=[{'shade':i,'row_sha256':sha(fade[i*256:(i+1)*256])} for i in range(64)]
fade_vectors=[{'shade':s,'source_index':i,'output_index':fade[s*256+i]} for s in [0,1,32,63] for i in [0,1,127,255]]
receipt={'candidate_sha256':prior['candidate_sha256'],'instruction_contracts':listings,
 'vertex_projection':{'function_va':'0x7b7f0','coordinate_domain':'unwrapped integer cells; map lookup wraps separately','camera_globals':['0x184fdc','0x184fe0'],
 'formula':'x=(dx-dy)*32; y=(dx+dy)*16 - 2*(height>>6) - explicit_vertical_top',
 'caller_anchor':'Optional additive placement is caller policy; source strip coordinates use anchor0.',
 'arithmetic':'Source uses signed32 wrapping arithmetic. Safe checked port may reject overflowing intermediate coordinates explicitly.',
 'vectors':projection_vectors,'parity_vectors':parity_vectors,'strip_equivalence_vertex_comparisons':comparisons},
 'triangle_binding':{'corner_order':['00','10','11','01'],'indices':triangles,'first_call':'0x7bb8e','second_call':'0x7bd9d','first_uv_offsets':[0,16,24],'second_uv_offsets':[0,8,16],'orientation_vectors':uv_vectors},
 'vertex_shade':{'source_fields':{'light_signed16':0,'factor_unsigned8':12},'formula':'4*light*factor + 0x8000','output':'signed32 16.16 shade','vectors':shade_vectors,'numeric_checks':65536*4,'meaning':'Factor is an explicit raw lighting operand; its visibility producer/gameplay semantics remain unresolved.'},
 'fade_lookup':{'entry_id':'root/2/disc/61','input_sha256':fade_result['input_sha256'],'decoded_size':len(fade),'decoded_sha256':sha(fade),'rows':64,'columns':256,'formula':'fade[shade_index*256 + source_palette_index]','safe_shade_indices':[0,63],'row_receipts':fade_rows,'vectors':fade_vectors,'zero_index_transparent':False},
 'limits':['This is the strip vertex path, not arbitrary-point projection7AF00.','No startup camera, player visibility or shade factor is invented.','Triangle coverage, clipping, gradient rounding and edge ownership are not yet established; do not substitute a generic rasterizer and claim original pixels.','The camera object setter829A0 sign-extends two stored cell words;82E20 has a separate target-anchoring routine, not an inferred startup position.']}
(P/'research/ghidra/terrain-projection-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS {len(listings)} instruction contracts; {comparisons} alternating-strip vertex comparisons; 8 UV bindings; 262144 shade arithmetic checks; 64 fade rows')
