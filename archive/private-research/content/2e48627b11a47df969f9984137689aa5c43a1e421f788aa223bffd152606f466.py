"""Static DOS callback, mode-two packet and two-arrow capture contracts."""
from pathlib import Path
import hashlib,json,itertools
P=Path(__file__).resolve().parents[2]; G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
contracts={
 'mouse-callback-refs/instructions-000a3164.txt':['MOV EBX,0xc','MOV ECX,0x7f','MOV EDI,0xa3524','MOV EAX,0x33','CALL 0x000bb43b'],
 'mouse-callback/instructions-000a3524.txt':['TEST AL,0x2','MOV byte ptr [0x0019cfb6],DL','MOV byte ptr [0x0019cfb3],DL','TEST AL,0x4','MOV byte ptr [0x0019cfb6],CL','TEST AL,0x8','MOV byte ptr [0x0019cfb7],DH','TEST AL,0x10','MOV byte ptr [0x0019cfb7],CH'],
 'packet-fields/instructions-00070e40.txt':['CMP AL,0x2','MOV [0x00118771],AX','MOV [0x00118773],AX','OR byte ptr [0x00118779],0x5','AND DL,0xf8','OR DH,0x4','OR byte ptr [0x00118779],0x40','AND byte ptr [0x0011877a],0xfe'],
 'input-scheduling/instructions-00070b00.txt':['00070b78 CALL 0x00070e40','00070bcc CALL dword ptr [EDX + 0x8]'],
 'packet-fields/instructions-000715a0.txt':['MOV byte ptr [0x0019cfb6],AH','MOV [0x0019cfb3],AL','MOV byte ptr [0x0019cfb7],AH','MOV [0x0019cfb4],AL'],
 'ui-input/instructions-000a87d0.txt':['MOV ESI,0xffffffff','MOV AX,[0x001a011c]','CMP EAX,0x1','TEST byte ptr [ECX + 0x8],0x2','000a8bea TEST CL,0x4','000a8bed JZ 0x000a8cbd','000a8cd7 MOV EAX,ESI','MOV ESI,0xfffffffe','MOV AL,[0x0010aafd]','MOV [0x001a011c],AX','000a8c4c DEC EDI','000a8ca4 MOV dword ptr [0x0019ff88],ECX'],
}
listings=[]
for name,terms in contracts.items():
    raw=(G/name).read_bytes(); txt=raw.decode()
    for term in terms: assert term in txt,(name,term)
    listings.append(dict(private_path='private/ghidra/'+name,sha256=sha(raw)))
def button(old,latched,held):
    if latched:return (1,old,1)
    if held:return (0,0,1)
    return (0,old,0)
vectors=[]
for old,latched,held in itertools.product([0,1],repeat=3):
    down,up,newheld=button(old,latched,held)
    vectors.append(dict(old_packet_held=old,latched_press=latched,physical_held=held,down=down,up=up,new_packet_held=newheld))
assert button(0,1,0)==(1,0,1) and button(1,0,0)==(0,1,0)
def capture(old,bits,hit):
    if old is not None:return (None,old) if bits&2 else (old,-1)
    return (hit,-2) if bits&1 and hit is not None else (None,-1)
cases=[]
for old in [None,27,28]:
    for bits in range(256):
        for hit in [None,27,28]:
            new,event=capture(old,bits,hit)
            assert old is None or new in [old,None]
            cases.append((old,bits,hit,new,event))
state,event=capture(None,3,27); assert (state,event)==(27,-2)
state,event=capture(state,0,None); assert (state,event)==(27,-1)
state,event=capture(state,2,None); assert (state,event)==(None,27)
receipt=dict(candidate_sha256='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',scope='Static mode-two mouse packet and two controls with initial delayone',callback_va='0xa3524',registration_va='0xa32d0',producer_va='0x70e40',producer_caller_va='0x70b00',packet_pointer_va='0x118970',packet_storage_va='0x118771',coordinate_type='signed16 lowwords of signed32 currentglobals19CF9C/19CFA0',packet_bits=dict(byte8={'0':'left press event','1':'left release event','2':'left packet-held state','6':'right press event','7':'right release event'},byte9={'0':'right packet-held state'}),button_vectors=vectors,capture_case_count=len(cases),capture_vectors=[dict(old_capture=None,byte8=3,hit=27,new_capture=27,event=-2),dict(old_capture=27,byte8=0,hit=None,new_capture=27,event=-1),dict(old_capture=27,byte8=2,hit=None,new_capture=None,event=27)],countdown=dict(source_va='0x10aafd',source_width='u8',state_va='0x1a011c',state_width='u16',custom_initial_value=1,greater_than_one='decrement per managercall; arrivalatone emits capturedID and clearscapture without releasecheck; excluded fromdelayoneport'),private_listings=listings,limits=['Modeone keyboard producer not ported','Mouse mickey scaling/clamping not ported','Wallclock tickrate and outer loop scheduling not established','Original async callback/producer race behavior not claimed','Screen list allocation/count and unchecked textcopy still unresolved'])
(P/'research/ghidra/input-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',newline='\n')
print(json.dumps(dict(status='passed',listing_contracts=len(listings),button_vectors=len(vectors),capture_cases=len(cases))))
