"""Independent bounded initial viewport register/rectangle review; no originals."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct
ROOT=Path(__file__).resolve().parents[2]
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def main():
 proof_path=ROOT/'research/ghidra/terrain-viewport-evidence.json';proof=json.loads(proof_path.read_bytes());pins=[]
 for p in proof['instruction_contracts']:
  pin=record(ROOT/p['private_path']);assert pin['sha256']==p['sha256'];pins.append(pin)
 ctx=bytearray(0x11d8);struct.pack_into('<H',ctx,0x1d,128)
 # 82BD0 followed by camera-reset82E20: both write low16 dimensions,
 # preserving margin. Reset fills20 dirty bytes with2.
 struct.pack_into('<HH',ctx,8,128,0);struct.pack_into('<HH',ctx,0x33,640-128,480);ctx[0x11bd:0x11bd+20]=bytes([2])*20
 word=lambda offset:struct.unpack_from('<h',ctx,offset)[0]
 assert {hex(off):word(off) for off in [0x1d,8,10,0x33,0x35]}==proof['initializer']['fields']
 listing=(ROOT/'private/ghidra/terrain-strip-refs/instructions-00086840.txt').read_text()
 for term in ['0008687f SUB EAX,0x4','0008688d JZ 0x000869d3','000869ea JL 0x00086a19','00086a2c ADD EDX,EAX','00086a2e SUB ECX,EAX','00086a3f SHL EAX,0x7','00086a46 XOR EBX,EBX','00086a4a XOR EDX,EDX','00086a4c CALL 0x000e88a3','00086a54 OR AH,0x4','00086a61 JNZ 0x00086a82','00086a84 MOVSX ECX,word ptr [ESI + 0x35]','00086a8a XOR EBX,EBX','00086a90 CALL 0x0007b7f0','00086a95 AND byte ptr [ESI + 0x1c],0xfb']:assert term in listing
 sampler=(ROOT/'private/ghidra/triangle-buffer-refs/instructions-000e88a3.txt').read_text()
 for term in ['000e88b9 OR EAX,EAX','000e88bb JZ 0x000e88c2','000e88bd MOV [0x00100110],EAX']:assert term in sampler
 startup=(ROOT/'private/ghidra/terrain-overlay-resource-refs/instructions-0001d770.txt').read_text()
 for term in ['0001db37 CMP EDX,0x1','0001db3a JNZ 0x0001db78','0001db41 MOV EBX,dword ptr [0x0019cf6c]','0001db51 CALL 0x000e88a3']:assert term in startup
 calls=[];flag_cases=0
 for old_flags in range(256):
  for column in range(4,20):
   local=column-4;eax=local*32+word(8);assert eax<640
   # Non-scrolled, non-wrapped86A19 path. EAX=base + top*640
   # + offset-margin; EBX=0 preserves inherited sampler pitch.
   buffer_delta=word(10)*640+word(8)-word(0x1d);clip_width=word(0x33)+word(0x1d)
   assert buffer_delta==0 and clip_width==640
   assert (old_flags|4)&251==old_flags&251
   if old_flags==0:calls.append(dict(column=column,buffer_delta=buffer_delta,pitch=640,clip_width=clip_width,clip_height=word(0x35),top=0,bottom=word(0x35),dirty_input=ctx[0x11bd+column],context_flags_delta=dict(or_before=4,and_after=251)))
   flag_cases+=1
 assert calls==proof['initial_strip_calls']
 mapped=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes();initial_pitch=struct.unpack_from('<I',mapped,0x110)[0];assert initial_pitch==0
 # EBX0 preserves either value:640 must be a runtime precondition, not an
 # inference from display dimensions or zero-initialized sampler storage.
 paths=[]
 for i in range(3):
  offset=struct.unpack_from('<I',mapped,0x5604+4*i)[0]-0x100000;paths.append(mapped[offset:mapped.index(0,offset)].decode())
 assert paths==proof['panel_resource_selection']['names']
 panel_positions=set();terrain_positions=set();source_positions=[]
 for y in range(480):
  for x in range(640):
   offset=y*640+x
   if x<128:panel_positions.add(offset);source_positions.append(y*128+x)
   else:terrain_positions.add(offset)
 assert not panel_positions&terrain_positions and len(panel_positions|terrain_positions)==307200 and source_positions==list(range(61440))
 assert proof['panel_resource_selection']['initial_panel_selector']=='UNRESOLVED'
 result=dict(status='passed-independent-initial-viewport-context-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,evidence=record(proof_path),instruction_pins=pins,strip_argument_vectors=len(calls),all_old_flag_column_cases=flag_cases,rectangle_positions_checked=307200,panel_source_indices_checked=61440,initial_static_sampler_pitch=initial_pitch,inherited_runtime_pitch_precondition=640,
  temporary_flag_semantics='Each strip sees oldflags|4; source then clears4 rather than restoring an originally set4',panel_selection='UNRESOLVED; GENEPANL only explicit diagnostic resource choice',review_source=record(Path(__file__)),
  limits=['Initial non-scrolled positive128margin/display640x480, initialized samplerpitch640 anddirty2 gates required','Display-lock pitch initialization is conditional; zeroEBX sampler call onlypreserves inheritedpitch','Selectedplayer0camera66,48 stillsubject tosuccessful source-load/campaign/selection gates','No scrollwrap, incremental redrawing, foreground entities, visibility callbacks or GUI timing reconstructed'])
 (ROOT/'research/variants/viewport-context-review.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='instruction_pins'}))
if __name__=='__main__':main()
