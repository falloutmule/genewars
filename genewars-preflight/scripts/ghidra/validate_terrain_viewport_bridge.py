"""Pin initial640x480 scene viewport and panel placement, no original execution."""
from pathlib import Path
import hashlib
import json
import struct

P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
contracts={
 'display-geometry/instructions-00082bd0.txt':['00082bda MOV word ptr [EAX + 0x1d],0x80','00082be7 MOV word ptr [EAX + 0x2],0x0','00082bed MOV word ptr [EAX],DX','00082c0d MOV word ptr [EAX],DX','00082c10 MOV word ptr [EAX + 0x2],BX'],
 'terrain-camera-refs/instructions-00082e20.txt':['00082eca MOV EBX,0x14','00082ecf MOV EDX,0x2','00082edd LEA EAX,[ESI + 0x11bd]','00082ee3 CALL 0x000982c0','00082eef MOV word ptr [EAX + 0x2],0x0','00082ef5 MOV word ptr [EAX],DX','00082f15 MOV word ptr [EAX],DX','00082f18 MOV word ptr [EAX + 0x2],BX'],
 'display-geometry/instructions-00086c20.txt':['00086c67 MOV EAX,0x4','00086c7c CMP EAX,0x14','00086c84 MOV DL,byte ptr [ECX + 0x11bd]','00086c8e MOVSX EBX,DL','00086c95 CALL 0x00086840'],
 'terrain-strip-refs/instructions-00086840.txt':['0008687f SUB EAX,0x4','0008688d JZ 0x000869d3','00086a19 MOVSX EAX,word ptr [ESI + 0x35]','00086a28 MOVSX EBX,word ptr [ESI + 0xa]','00086a4c CALL 0x000e88a3','00086a54 OR AH,0x4','00086a61 JNZ 0x00086a82','00086a90 CALL 0x0007b7f0','00086a95 AND byte ptr [ESI + 0x1c],0xfb'],
 'terrain-overlay-resource-refs/instructions-0001d770.txt':['0001db37 CMP EDX,0x1','0001db3a JNZ 0x0001db78','0001db41 MOV EBX,dword ptr [0x0019cf6c]','0001db51 CALL 0x000e88a3','0001dff8 MOV EAX,0x16e0a8','0001e00f CALL 0x00082bd0'],
 'triangle-buffer-refs/instructions-000e88a3.txt':['000e88c9 MOV [0x00100104],EAX','000e88ce SUB EAX,dword ptr [0x00100110]','000e88d4 MOV [0x00100100],EAX'],
 'panel-verified/instructions-00024ad0.txt':['00024ae5 MOV ECX,0x20','00024aee MOVSD.REP ES:EDI,ESI','00024b06 ADD EBX,0x80','00024b0c ADD EDX,EAX'],
 'panel-verified/instructions-00024770.txt':['000247a4 MOV BL,byte ptr [ECX + 0x4]','000247a7 MOVSX EAX,BL','000247fa MOV EDX,dword ptr [EAX*0x4 + 0x105604]'],
}
pins=[]
for name,expected in contracts.items():
    raw=(G/name).read_bytes()
    for line in expected:assert line in raw.decode(),(name,line)
    pins.append({'private_path':'private/ghidra/'+name,'sha256':sha(raw)})
mapped=(G/'validated/memory-.object3.bin').read_bytes()
paths=[]
for i in range(3):
    a=struct.unpack_from('<I',mapped,0x105604-0x100000+4*i)[0]-0x100000
    paths.append(mapped[a:mapped.index(0,a)].decode('ascii'))
assert paths==['data/mainpanl.dat','data/genepanl.dat','data/lpanel.dat']
context={'panel_margin':128,'horizontal_offset':128,'vertical_offset':0,'terrain_width':512,'terrain_height':480}
columns=list(range(4,20))
assert len(columns)==16 and columns[0]*32==128 and (columns[-1]+1)*32==640
# Recreate the non-wrapped86840 initial branch register arguments percolumn.
calls=[]
for c in columns:
    local_c=c-4
    offset=context['horizontal_offset']
    assert offset+local_c*32<640
    buffer_delta=context['vertical_offset']*640+offset-context['panel_margin']
    clip_width=context['terrain_width']+context['panel_margin']
    calls.append({'column':c,'buffer_delta':buffer_delta,'pitch':640,'clip_width':clip_width,'clip_height':480,
                  'top':0,'bottom':480,'dirty_input':2,'context_flags_delta':{'or_before':4,'and_after':251}})
    assert buffer_delta==0 and clip_width==640
panel_points={(x,y) for y in range(480) for x in range(128)}
terrain_points={(x,y) for y in range(480) for x in range(128,640)}
assert not panel_points&terrain_points and len(panel_points|terrain_points)==307200
mission=json.loads((P/'research/ghidra/mission-load-evidence.json').read_text())
assert mission['first_camera']['camera_cell']==[66,48]
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,
 'scope':'Initialnon-scrolled640x480context after82E20camera reset, withpositivepanelmargin and inheritedinitializedsamplerpitch640; scenecomposition geometry and base-terrain dispatch, not allGUI/entity/timing callbacks.',
 'pitch_precondition':'86840passesEBX0toE88A3, preserving100110. Requireinheritedpitch640;1D770setsitfromdisplaywidth at1DB51onlywhenitsdisplaylockpathreturnsEDX1. No unconditionalpitchinitializationclaim.',
 'initializer':{'va':'0x82bd0','caller':'0x1e00f','context':'0x16e0a8','fields':{'0x1d':128,'0x8':128,'0xa':0,'0x33':512,'0x35':480}},
 'camera_reset':{'va':'0x82e20','retains_panel_margin':True,'rewrites_offsets_dimensions':True,'dirty_byte_count':20,'dirty_fill_value':2},
 'initial_strip_calls':calls,
 'rectangles':{'panel':{'x':0,'y':0,'width':128,'height':480,'source_pitch':128,'destination_pitch':640},'terrain':{'x':128,'y':0,'width':512,'height':480,'source_pitch':640,'destination_pitch':640}},
 'palette':'MAIN.PAL association established inrenderer-evidence.json; preserveindices and declareRGB6displayconversion.',
 'panel_resource_selection':{'table':'0x105604','selector':'signedbytepanelobject+4','names':paths,'initial_panel_selector':'UNRESOLVED','diagnostic_GENEPANL':'Explicitresourcechoice only, not yetconfirmedfirstmissionpanel.'},
 'target_gate_reference':'research/ghidra/mission-load-evidence.json','selected_player0_camera':[66,48],
 'partition_pixel_checks':307200,
 'limits':['Currentselectedplayer/starttarget gates remain asmission-loadreceipt.','Dirtycontext4doesnotchange5/6pixels, butitsmap+11ANDrequests mustremainvisible.','Startupcontextflags beyondtemporarybit4 are notinvented; explicitcalleroldflags combinewithOR4.','Panelbackgroundidentity andGUIoverlays must notbeclaimedfromplacementalone.','Scrollwrap, incrementalredraw, alternatevideomodes, entities and completeoriginalvisibility remainoutside thisreceipt.']}
(P/'research/ghidra/terrain-viewport-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS {len(pins)} contracts; 16 initialstripargument vectors; 307200 rectanglepartition checks; explicitpanelresource uncertainty')
