"""Safe storage bounds from custom-screen initialization and list maintenance."""
from pathlib import Path
import hashlib,json
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
checks={
 'ui-custom/instructions-0007e0b0.txt':['MOV EBX,0x8ca','LEA EAX,[ECX + 0x507]','MOV byte ptr [ECX + 0xdd1],0x0'],
 'ui-controls/instructions-0007e9c0.txt':['CALL 0x0007ef30'],
 'list-layout/instructions-0007ef30.txt':['MOV AL,byte ptr [EBP + 0xdd1]','CMP EAX,0xa','JGE 0x0007f065','ADD EBX,0xe1','LEA ECX,[EBX + 0xe]','MOV ECX,0x28','LEA EDI,[EBX + 0x8]','MOV ECX,0x58','LEA EDI,[EBX + 0x30]','INC byte ptr [EBP + 0xdd1]','DEC byte ptr [EBP + 0xdd1]','CMP EAX,0x190'],
 'ui-controls/instructions-0007e400.txt':['LEA EDX,[EAX*0x8 + 0x0]','SUB EDX,EAX','SHL EDX,0x5','ADD EDX,EAX','LEA ESI,[EBX + 0x507]','ADD ESI,0xe'],
}
listings=[]
for path,terms in checks.items():
 raw=(G/path).read_bytes();txt=raw.decode()
 for term in terms:assert term in txt,(path,term)
 listings.append(dict(private_path='private/ghidra/'+path,sha256=hashlib.sha256(raw).hexdigest()))
assert 0x507+10*0xe1==0xdd1 and 0x8ca==10*0xe1
valid=0
for count in range(11):
 for selected in range(256):
  accept=selected<count and selected<10
  if accept:
   start=0x507+selected*0xe1
   assert 0x507<=start and start+0xe1<=0xdd1
   assert start+0xe==0x515+selected*0xe1
   valid+=1
receipt=dict(candidate_sha256='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',scope='Static custom-screen record capacity; safe bounds recommendation, no unchecked original textcopy',initializer_va='0x7e0b0',maintainer_va='0x7ef30',record_storage_offset='0x507',record_stride=225,capacity=10,zeroed_storage_bytes=2250,count_offset='0xdd1',count_type='u8',record_name_offset='0xe',record_header_offset=8,record_header_bytes=40,following_metadata_offset='0x30',following_metadata_bytes=88,safe_name_window_bytes=34,safe_selection_rule='count<=10 AND selected<count; require NUL in record[0xE:0x30], then validate destination limit and encoding',selection_vectors_checked=2816,accepted_combinations=valid,private_listings=listings,unresolved=['Incoming record semantics/encoding and guaranteed string terminator are not proven','Original drawloop limits rowcount by count rather than count-offset; do not port unchecked accesses','Absolute allocation size/other object fields and full maintainer lifecycle not reconstructed','Original unchecked copy tostate+3D5 excluded'])
(P/'research/ghidra/list-layout-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',newline='\n')
print(json.dumps(dict(status='passed',listing_contracts=len(listings),selection_vectors=2816,capacity=10)))
