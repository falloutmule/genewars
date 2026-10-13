"""Validate bounded main-menu/font contracts against private static exports and CD assets."""
from pathlib import Path
import hashlib,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra';sha=lambda b:hashlib.sha256(b).hexdigest()
game='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==game
b=(G/'validated/memory-.object3.bin').read_bytes()
u32=lambda va:struct.unpack_from('<I',b,va-0x100000)[0]
def s(va):
 o=va-0x100000;return b[o:b.index(0,o)].decode()
assert u32(0x111ed8)==0x20 and b[0x18968]==0
assert s(0x1061ec)=='data/iftitle.anb' and u32(0x1061ec+28)==0x184ff0
assert s(0x106218)=='data/titlescr.pal' and u32(0x106218+28)==0x18500a
assert s(0x106244)=='data/font2.dat' and u32(0x106244+28)==0x118c94
assert s(0x106270)=='data/font2.tab' and u32(0x106270+28)==0x118c8c
assert u32(0x106270+32)==0x118c90
assert s(0x10e940)=='data/titlescr.dat'
controls=[]
for i,(y,ident) in enumerate([(85,0),(156,1),(227,3),(298,2)]):
 o=0x708c+i*40;flags,x,actual_y,w,h,event=struct.unpack_from('<I4hH',b,o)
 assert (flags,x,actual_y,event)==(0x1048,140,y,ident)
 assert struct.unpack_from('<I',b,o+18)[0]==0
 assert struct.unpack_from('<I',b,o+22)[0]==36
 controls.append(dict(record_va=hex(0x100000+o),id=event,flags=hex(flags),x=x,four_button_y=y,animation_index=0))
assert u32(0x10712c)==0
contracts={
 'font-draw/instructions-000a7618.txt':['000a761c ADD EAX,0x6','000a7623 MOV ECX,dword ptr [EAX]','000a7629 LEA ESI,[ECX + EBX*0x1]','000a762c MOV dword ptr [EAX],ESI'],
 'font-menu-constructor-tail.txt':['0008dc04 MOV EAX,[0x00111ed8]','0008dc38 ADD EAX,0x7','0008dc4c MOV EAX,0x10708c','0008dc57 MOV [0x00184fec],EAX'],
 'font-text/instructions-0007c6a0.txt':['0007c6b2 MOV EDX,0x1061ec','0007c6e1 MOV EDX,0x118c8c','0007c6eb MOV EBX,dword ptr [ESI + EBX*0x1 + 0x2]','0007c6ef CALL 0x0001f550','0007c714 TEST byte ptr [0x00118968],0x4','0007c720 OR byte ptr [EAX + 0x28],0x1','0007c727 MOV word ptr [EAX + 0x6],0x73','0007c730 MOV word ptr [EAX + 0x56],0xba','0007c739 MOV word ptr [EAX + 0x7e],0x101','0007c748 CALL 0x000a8320','0007c750 MOV [0x0019ff64],EAX','0007c7ce OR AL,0x20'],
 'font-init/instructions-0001f550.txt':['0001f559 MOV EBX,dword ptr [EDX + 0x8]','0001f561 CALL 0x000a7618','0001f568 ADD EAX,0x6','0001f56b MOV dword ptr [ESI],EAX','0001f575 OR EAX,0x80000003','0001f58f MOV dword ptr [ESI + 0x8],EBP'],
 'font-init/instructions-0001f5a0.txt':['0001f5ce LEA ECX,[ESI + -0x20]','0001f5d1 IMUL ECX,ECX,0x6','0001f634 MOV ESI,dword ptr [EBX]','0001f636 MOV CL,byte ptr [ECX + ESI*0x1 + 0x4]','0001f640 DEC ECX'],
 'font-wrapper/instructions-0001f730.txt':['0001f738 CALL 0x0001f5a0','0001f73d SAR EAX,0x1','0001f741 SUB EDX,EAX','0001f745 CALL 0x0001f750'],
 'font-init/instructions-0001f750.txt':['0001fa82 LEA EAX,[EDX + -0x20]','0001fa85 IMUL EAX,EAX,0x6','0001fa88 MOV EBP,dword ptr [ESI]','0001fa8d ADD EBP,EAX','0001fa97 CMP EAX,0x20','0001faac CALL 0x000a7730','0001fabc DEC EAX'],
 'font-menu/instructions-0007ca70.txt':['0007ca87 CALL 0x000aea78','0007ca8c CALL 0x000a85d0','0007ca94 MOV EDX,0x138','0007ca99 MOV ECX,dword ptr [0x0011628c]','0007ca9f MOVSX EBX,word ptr [EAX + 0x6]','0007caa8 ADD EBX,0x19','0007cab0 TEST byte ptr [0x00118968],0x4','0007cac1 MOV ECX,dword ptr [0x00116294]','0007cae9 MOV ECX,dword ptr [0x00116298]','0007caff MOV ECX,dword ptr [0x0011629c]'],
 'font-menu-bind/instructions-000a8320.txt':['000a839c MOV ESI,dword ptr [ECX + 0x12]','000a839f ADD ESI,EBX','000a83a3 MOV dword ptr [ECX + 0x12],ESI','000a83ab MOV EAX,dword ptr [ESI]'],
 'ui-controls/instructions-000a84d0.txt':['000a856b MOV EBX,dword ptr [EAX + 0x12]','000a856e MOV EBX,dword ptr [EBX + 0x4]','000a8573 MOV EBX,dword ptr [EAX + 0x12]','000a8576 MOV EBX,dword ptr [EBX]','000a857e CALL 0x000982e0'],
 'font-menu-bind/instructions-000a85d0.txt':['000a85d6 MOV EBX,dword ptr [0x0019ff64]','000a861c TEST word ptr [EBX],0x101','000a8628 MOV DL,byte ptr [EBX + 0x27]','000a8636 CALL 0x000a84d0'],
 'font-menu-construct/instructions-000aea78.txt':['000aea88 MOV EAX,[0x0019cf6c]','000aea90 MOV EAX,[0x0019cf7c]','000aea98 SHR EAX,0x2','000aea9d MOVSD.REP ES:EDI,ESI','000aeaa2 DEC EBX'],
 'display-geometry/instructions-000a7730.txt':['000a78e1 JZ 0x000a7e77','000a7ea6 CMP EAX,EBP','000a7eaa MOV EAX,EBP','000a7ec3 MOVSX EAX,byte ptr [EDX]','000a7ec6 SUB EBP,EAX','000a7ecb ADD EDX,EAX','000a7f08 MOV EAX,0x1','000a7f15 MOVSX EBX,byte ptr [EDX]','000a7f24 JNZ 0x000a7f15'],
 'font-menu-state/instructions-0001cdf0.txt':['0001d048 CALL 0x000a604c','0001d04d CMP EAX,0x1','0001d05a OR BH,0x4']}
listings=[]
for name,terms in contracts.items():
 raw=(G/name).read_bytes();t=raw.decode()
 for term in terms:assert term in t,(name,term)
 listings.append(dict(private_path='private/ghidra/'+name,sha256=sha(raw)))
corpus=json.loads((P/'research/integration/rnc-corpus.json').read_text())['results']
manifest=json.loads((P/'research/game-files-manifest.json').read_text())['entries']
assets=[]
def asset(entry):
 e=next(e for e in manifest if e['entry_id']==entry);r=next(r for r in corpus if r['entry_id']==entry and r['status']=='passed');raw=(P/r['private_output']).read_bytes();assert len(raw)==r['output_size'] and sha(raw)==r['output_sha256'];assets.append(dict(entry_id=entry,source=e['normalized_iso_path'],input_sha256=e['sha256'],decoded_size=len(raw),decoded_sha256=sha(raw)));return raw
anb=asset('root/2/disc/92');background=asset('root/2/disc/135');palette=asset('root/2/disc/136');font=asset('root/2/disc/75');tab=asset('root/2/disc/76')
assert len(background)==640*480 and len(palette)==768 and max(palette)<=63
n,t,c,f,a=struct.unpack_from('<H4I',anb);assert (n,t,c,f,a)==(1,57983,58043,58133,58175)
first,last,count,flags=struct.unpack_from('<2I2B',anb,a);assert (first,last,count,flags)==(0,28,3,0)
states=[]
for name,offset,wanted in [('normal',first,[1,2,3]),('selected',last,[7,8,9])]:
 co,w,h=struct.unpack_from('<I2h',anb,f+offset);cnt=anb[f+offset+11];assert (w>>1,h>>1,cnt)==(346,75,3);parts=[]
 for i,(sprite,x,y,sw,sh) in enumerate(zip(wanted,[0,116,236],[0,11,3],[116,120,110],[75,50,71])):
  sp,cx,cy,cf,pad=struct.unpack_from('<I2h2B',anb,c+co+i*10);po,pw,ph=struct.unpack_from('<IBB',anb,t+sp)
  assert (sp//6,cx>>1,cy>>1,cf,pw,ph)==(sprite,x,y,0,sw,sh)
  parts.append(dict(sprite_index=sprite,x=x,y=y,width=pw,height=ph,flags=cf))
 states.append(dict(state=name,frame_index=offset//14,width=346,height=75,components=parts))
assert len(tab)==1170 and struct.unpack_from('<IBB',tab,204)==(3917,30,27)
text_entry=next(e for e in manifest if e['entry_id']=='root/2/disc/592')
text=(P/'private/reference-run/run-002/drive-c/GAME/DATA/TEXT/ENG/TEXT.DAT').read_bytes();assert sha(text)==text_entry['sha256']
pos=0;strings=[]
for i in range(604):
 end=text.index(0,pos);strings.append(text[pos:end]);pos=end+1
assert pos==36609 and text[pos:]==b'\0'
labels=[]
for index,y in [(255,140),(258,211),(259,282),(257,181)]:
 line=strings[index];assert all(32<=v<=225 for v in line);width=sum(tab[(v-31)*6+4]-1 for v in line)
 labels.append(dict(text_index=index,byte_count=len(line),text_sha256=sha(line),width=width,x_center=312,left=312-(width>>1),three_button_y=y if index!=257 else None))
assert [r['width'] for r in labels]==[246,151,85,178]
out=dict(status='verified_bounded_static_menu_and_font_contract',candidate_sha256=game,screen_va='0x184fe8',control_table_va='0x10708c',palette_pointer_va='0x18500a',palette_source='TITLESCR.PAL',framebuffer_contract='640x480 indexed row-major, full clip, zero draw flags and UI origin(0,0) required',controls=controls,three_button_positions=[[140,115],[140,186],[140,257]],animation_index=0,states=states,labels=labels,assets=assets,font_contract=dict(table_record_size=6,table_prefix_bytes=6,dat_base_bias=0,absolute_tab_offset='(byte_code-31)*6',space_tab_offset=6,A_tab_offset=204,A_record=[3917,30,27],advance='width-1',scope='one unkerned line, bytes>=32 and bounded by admitted table; space advances without painting',rle='signed control: positive literal bytes including opaque zero; negative transparent skip; zero row end; clip writes, consume full literal runs',safe_deviation='Require final row zero for complete bounded stream; runtime may return immediately after final visible row'),text_contract=dict(count=604,consumed=36609,trailing_bytes=1,encoding='opaque bytes'),unknowns=['wallclock callback timing','complete network initialization semantics','live runtime draw flags; native normal composition requires zero','mouse-motion runtime cause'],private_listings=listings)
(P/'research/ghidra/font-menu-evidence.json').write_text(json.dumps(out,indent=2)+'\n')
print('PASS font/menu: 14 listing contracts, 5 exact CD assets, 4 control records, 6 frame components, 604 text entries, 4 centered label widths.')
