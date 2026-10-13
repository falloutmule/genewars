"""Bounded static metadata assessment; does not repair or execute original data."""
from pathlib import Path
import json,struct,hashlib
P=Path(__file__).resolve().parents[2]
H='2106ce79deaafe53d5fbb6292c78a8c71fb6920b08d1a11b5c44ea3b0ab75e9f'
b=(P/f'private/integration/sprites/{H}.anb').read_bytes()
assert hashlib.sha256(b).hexdigest()==H
n,t,c,f,a=struct.unpack_from('<H4I',b)
assert (len(b),n,t,c,f,a)==(4077,5,3721,3787,3887,4027)
offset,width,height=struct.unpack_from('<IBB',b,t+5*6)
assert (offset,width,height)==(1249,65,36)
component=8
assert struct.unpack_from('<I',b,c+component*10)[0]==5*6
assert struct.unpack_from('<I',b,f+8*14)[0]==component*10 and b[f+8*14+11]==1
assert struct.unpack_from('<I',b,a+4*10)[0]==8*14
next_offset=struct.unpack_from('<I',b,t+6*6)[0]+18
pos=offset+18
completed=[];expanded=0
while pos<next_offset:
    control=b[pos];control=control if control<128 else control-256;pos+=1
    if control>0:pos+=control;expanded+=control
    elif control<0:expanded-=control
    else:completed.append(dict(row=len(completed),expanded_width=expanded,end=pos));expanded=0
assert len(completed)==4 and pos>next_offset
candidate=(P/'private/ghidra/GW-CD-candidate.exe').read_bytes()
out=dict(scope='static exception triage; no guessed repair or original execution',
 bones=dict(input_sha256=H,bytes=len(b),sprite_record=5,width=width,height=height,
   sprite_start=offset+18,next_sprite_start=next_offset,complete_rows_before_next_sprite=completed,
   cursor_crossing_next_sprite=pos,referencing_component=8,referencing_frame=8,referencing_animation=4,
   sentinel_exemption=False,status='resolved_packed4',
   admission_receipt='research/ghidra/bones-evidence.json',
   conclusion='The indexed8 model crosses neighboring sprite data. Actual entity/frame consumers90460/94400 establish packed4; separate hash-bound receipt validates all10 active streams. This diagnostic retains the rejected-model observation.'),
 gpf=dict(cd_candidate_literal_reference=b'gpf' in candidate.lower(),status='blocked',
   conclusion='Reported DEADBEEF/prefix-RNC wrapper and CRC failure have no established support in ANB loader98A60. Do not bypass CRC or invent a variant.'),
   instruction_basis=['0x98A60 fixes sprite data relative to file+18','0xA7730 is the rejected indexed8 model for BONES','0x5A1C0 selects BONES animation base','0x94910/0x94D70 call packed4 consumers90460/94400'],
 tests_passed=True)
(P/'research/ghidra/sprite-exception-assessment.json').write_text(json.dumps(out,indent=2)+'\n')
print('PASS BONES rejected-indexed8 diagnostic retained; packed4 admission is separate; GPF remains blocked')
