"""Metadata and synthetic contracts for two statically verified custom UI controls."""
from pathlib import Path
import hashlib,json,struct
P=Path(__file__).resolve().parents[2]
R=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
code=(R/'validated/memory-.object1.bin').read_bytes()
data=(R/'validated/memory-.object3.bin').read_bytes()
candidate=(R/'GW-CD-candidate.exe').read_bytes()
assert sha(candidate)=='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert b'ifeditor' not in candidate.lower()
assert data[0x106530-0x100000:0x106530-0x100000+18].split(b'\0')[0]==b'data/ifcustom.anb'
assert struct.unpack_from('<I',data,0x106530-0x100000+28)[0]==0x18501c

rows=json.loads((P/'research/integration/rnc-corpus.json').read_text())['results']
asset=next(r for r in rows if 'GENEWARS.img' in r['source_path'] and 'IFCUSTOM.ANB' in r['source_path'])
anb=(P/f"private/integration/rnc-decoded/{asset['input_sha256']}.bin").read_bytes()
assert sha(anb)==asset['output_sha256']=='daf41bf104dd5d6436f7af23158a5d23d624c12780e3e66177b3b5aec2fbe006'
n,table,components,frames,animations=struct.unpack_from('<H4I',anb)
assert (n,table,components,frames,animations)==(13,35496,35718,36068,36530)
assert animations+n*10==len(anb)
assert struct.unpack_from('<I',code,0x7e3b0-0x10000+(27-10)*4)[0]==0x7e8dc
assert struct.unpack_from('<I',code,0x7e3b0-0x10000+(28-10)*4)[0]==0x7e8ef

controls=[]
for va,wanted_id,position,anim in [(0x1072bc,27,(455,229),6),(0x1072e4,28,(455,261),7)]:
    off=va-0x100000
    flags,x,y,width,height,ident=struct.unpack_from('<I4hH',data,off)
    animation_offset=struct.unpack_from('<I',data,off+18)[0]
    assert (flags,x,y,ident,animation_offset)==(0x48,*position,wanted_id,anim*10)
    assert width==height==0
    first,last,count,animflags=struct.unpack_from('<2I2B',anb,animations+animation_offset)
    assert count==2 and last-first==14
    states=[]
    for label,frame_offset in [('normal',first),('selected',last)]:
        component_offset,fw,fh=struct.unpack_from('<I2h',anb,frames+frame_offset)
        component_count=anb[frames+frame_offset+11]
        assert component_count==1
        sprite_offset,cx,cy,cflags,_=struct.unpack_from('<I2h2B',anb,components+component_offset)
        assert sprite_offset%6==0 and cx==cflags==0
        assert cy==(0 if label=='normal' else 2)
        pixel_offset,sw,sh=struct.unpack_from('<IBB',anb,table+sprite_offset)
        assert (fw>>1,fh>>1)==(19,31) and (sw,sh)==(19,32)
        states.append(dict(state=label,frame_index=frame_offset//14,component_index=component_offset//10,
            sprite_index=sprite_offset//6,width=sw,height=sh,frame_bounds=[fw>>1,fh>>1],
            component_position=[cx>>1,cy>>1],sprite_data_offset=pixel_offset+18))
    controls.append(dict(record_va=hex(va),id=ident,flags=hex(flags),position=[x,y],animation_index=anim,states=states))

checks=[('ui-objects/constructor-range.txt',['MOV ESI,0x107154','MOV dword ptr [0x00185018],ESI','MOV dword ptr [0x00185014],EDX']),
 ('ui-custom/instructions-0007e0b0.txt',['MOV EDX,0x106530','CALL 0x00098a60','CALL 0x000a8320','MOV byte ptr [ECX + 0x16],0x0']),
 ('ui-controls/instructions-0007e400.txt',['CALL 0x000a87d0','0007e8dc','DEC BL','INC byte ptr [EAX + 0x16]','MOV byte ptr [EBX + 0x19],AL']),
 ('ui-input/instructions-000a87d0.txt',['TEST byte ptr [ECX + 0x8],0x2','MOV byte ptr [EAX + 0x26],0x0','MOV dword ptr [0x0019ff88],ECX']),
 ('ui-controls/instructions-000a86d0.txt',['CALL 0x00098c40','TEST byte ptr [EAX],0x40']),
 ('ui-frame/instructions-00098c40.txt',['CALL 0x00098b60','MOVZX ESI,byte ptr [EAX + 0xb]'])]
receipts=[]
for name,tokens in checks:
    path=R/name
    listing=path.read_text()
    assert all(t in listing for t in tokens),(name,tokens)
    receipts.append(dict(private_path='private/ghidra/'+name,sha256=sha(path.read_bytes())))

def transition(value,event):
    if event==27:return max(value-1,0)
    if event==28:return (value+1)&255
    return value
vectors=[]
for value in [0,1,254,255]:
    for event in [27,28,-1,-2]:vectors.append(dict(before=value,event_id=event,after=transition(value,event)))
assert all(transition(v,27)==(v-1 if v else 0) and transition(v,28)==(v+1)%256
           and transition(v,-1)==transition(v,-2)==v for v in range(256))
def select(value,event,x,y,packetflag):
    if packetflag and event==-1 and 180<=x<454 and 224<=y<296:
        return (value+(y-224)//12)&255
    return None
selection_vectors=[]
for value,event,x,y,flag in [(0,-1,180,224,1),(0,-1,453,295,1),(255,-1,200,295,1),
 (0,-1,179,224,1),(0,-1,454,224,1),(0,-1,180,223,1),(0,-1,180,296,1),
 (0,27,200,240,1),(0,-1,200,240,0)]:
    selection_vectors.append(dict(offset=value,event_id=event,x=x,y=y,packetflag=flag,selected=select(value,event,x,y,flag)))
assert [v['selected'] for v in selection_vectors]==[0,5,4,None,None,None,None,None,None]
out=dict(scope='provisional CD candidate; static semantics only; no original execution',
 candidate_sha256=sha(candidate),ifeditor_literal_present=False,
 asset=dict(entry_id=asset['entry_id'],source_path=asset['source_path'],input_sha256=asset['input_sha256'],
  decoded_sha256=sha(anb),decoded_bytes=len(anb)),
 screen_state_va='0x185014',controls_pointer_va='0x185018',control_table_va='0x107154',
 asset_pointer_va='0x18501c',initializer_va='0x7e0b0',input_dispatch_va='0x7e400',
 draw_dispatch_va='0x7e9c0',control_stride=40,controls=controls,
 offset_field='state+0x16 u8',selected_field='state+0x19 u8',offset_vectors=vectors,
 selection_vectors=selection_vectors,receipts=receipts,
 tests=dict(offset_domain_values=256,offset_events=4,selection_boundary_cases=len(selection_vectors),all_passed=True),
 limitations=['Event-to-offset helper only, not the entire custom screen.',
 'No logical item-count clamp is present in these two event cases; increment wraps.',
 'Sprite hit test uses opaque pixels of components, bypassing the 19x31 frame rectangle; sprite raster is19x32.',
 'Later input-evidence.json establishes physical button mapping; unrelated control flags and wallclock cadence remain outside scope.'])
(P/'research/ghidra/ui-evidence.json').write_text(json.dumps(out,indent=2)+'\n',newline='\n')
print('PASS exact CD custom UI binding, 4 sprite/frame states, 256 offset values, 9 selection boundary cases')
