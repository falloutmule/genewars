"""Fresh-player restore, full bounded blueprint generator and typed resource keys."""
from pathlib import Path
import hashlib,itertools,json,struct,subprocess,sys
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra';sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
exe=(G/'GW-CD-candidate.exe').read_bytes();assert sha(exe)==candidate
mem=(G/'validated/memory-.object3.bin').read_bytes();code=(G/'validated/memory-.object1.bin').read_bytes()
pins=[]
def pin(path,*lines):
 b=(G/path).read_bytes()
 for line in lines:assert line in b.decode(),(path,line)
 pins.append({'private_path':'private/ghidra/'+path,'sha256':sha(b)})
pin('campaign-refs/instructions-0007c870.txt','0007c8a6 TEST byte ptr [0x00118969],0x80','0007c8dc MOV EBX,0x70d','0007c8e8 CALL 0x000982c0','0007c906 CALL 0x00080460')
pin('campaign-start/instructions-00080460.txt','00080496 MOV byte ptr [ECX],0x2','000804a6 MOV dword ptr [ECX + 0x9],EAX','000804ea INC dword ptr [EAX + 0x119479]','00080546 MOV dword ptr [ECX + 0x62a],0x11800','00080550 MOV ESI,0x10eade')
pin('camera-player-input/instructions-00080590.txt','00080596 MOV ECX,0x4a1','000805af MOV ECX,0xdd','000805b4 LEA ESI,[EBX + 0x61a]')
pin('player-creation/instructions-0007f580.txt','0007f5a0 MOV byte ptr [EDX],0x0','0007f5a3 ADD EDX,0x24','0007f60e IMUL AX,word ptr [0x0011878a],-0x19d3','0007f699 CALL 0x0007fad0','0007f6b9 MOV ECX,0x9','0007f6c0 MOVSD.REP ES:EDI,ESI','0007f6ce CMP EAX,0x5')
pin('player-creation/instructions-0007fad0.txt','0007fdb0 MOV byte ptr [EAX + 0x15],BL','0007fdb3 MOV byte ptr [EAX + 0x16],CH','0007fe5f MOV ECX,dword ptr [EAX + 0x104af0]','0007fe80 MOV ESI,dword ptr [EAX + 0x104af4]','0007fe99 MOV dword ptr [EDX + 0x1c],0x0','0007fea2 MOV dword ptr [EDX + 0x20],0x0','0007ff47 IDIV ECX','0007ff9c CALL 0x000aebb1','0007ffd4 CALL 0x0009a630')
pin('blueprint-init/instructions-0009a630.txt','0009a64a MOV byte ptr [EAX],0x0')
pin('blueprint-init/instructions-000aebb1.txt','000aebbe CMP CL,CH','000aebd0 CMP byte ptr [EAX],0x0')
pin('font-text/instructions-0001d770.txt','0001da68 ADD EDX,0x115e90','0001da71 MOV dword ptr [EDX],EAX')
pin('actor-resource-refs/instructions-0001e220.txt','0001e233 MOV EDX,0x104074','0001e24a CALL 0x000154b0','0001e265 MOV EAX,dword ptr [EDX + 0x1168f4]','0001e26e CALL 0x00098a60','0001e273 MOV dword ptr [EDX + 0x116930],EAX')
pin('palette-functions/instructions-000154b0.txt','000154c8 ADD EBX,0x2c')
pin('sprite-functions/instructions-00098a60.txt','00098a88 LEA EBX,[EAX + 0x12]')
pin('player-creation/instructions-0004fd30.txt','0004fd66 MOV EAX,[0x00112d10]','0004fda6 MOV dword ptr [ECX + 0x11],EAX','0004fdb0 MOV byte ptr [ECX + 0x17],AL','0004fed4 MOV byte ptr [EBP + 0x4a],AH','0004fef4 MOV word ptr [ECX + 0x15],0x280','0004ff2c MOV dword ptr [ECX + 0xc0],EBX')
pin('mission-tail-init/instructions-00056e20.txt','00056e35 MOVSX EDX,byte ptr [ECX + 0x74]','00056e63 CALL 0x000982c0','00056e86 MOV EAX,dword ptr [EDX + EAX*0x4 + 0x1168f4]','00056ea4 MOV EAX,dword ptr [EDX + EAX*0x4 + 0x116934]')
pin('camera-player-input/instructions-00080830.txt','000808dc OR byte ptr [EDX + 0x71],0x4','00080886 MOV EAX,dword ptr [EBX + 0x9]','00080899 CALL dword ptr [ESI + 0x60]')
caps=mem[0x4af0:0x4b30];assert caps==exe[0x1268f0:0x126930]
cap17=[struct.unpack_from('<i',caps,s*16)[0] for s in range(4)]
cap19=[struct.unpack_from('<i',caps,s*16+4)[0] for s in range(4)]
assert cap17==[100,75,75,75] and cap19==[70,100,65,90]
assert list(struct.unpack_from('<4I',code,0x7fab4-0x10000))==[0x7faf7,0x7fba8,0x7fc50,0x7fcff]
label=mem[0xeade:0xeae5];assert label==b'Player\0'
manifest=json.loads((P/'research/game-files-manifest.json').read_text())['entries']
corpus=json.loads((P/'research/integration/sprite-corpus.json').read_text())['results']
resources=[]
for index in range(16):
 o=0x4074+44*index;name=mem[o:mem.index(0,o)].decode('ascii')
 assert struct.unpack_from('<I',mem,o+28)[0]==0x1168f4+index*4
 entry=next(e for e in manifest if e.get('normalized_iso_path','').lower()=='game/'+name)
 c=next(c for c in corpus if c['entry_id']==entry['entry_id'] and c['status']=='passed')
 database=(P/c['private_database']).read_bytes();assert sha(database)==c['decoded_sha256']
 count,tab,component,frame,animation=struct.unpack_from('<H4I',database)
 assert 18<=tab<=component<=frame<=animation and animation+10*count==len(database)
 resources.append({'style':index//4,'category':index%4,'key':index,'descriptor_va':hex(0x104074+44*index),'raw_slot':hex(0x1168f4+index*4),'animation_slot':hex(0x116934+index*4),'source_name':name,'entry_id':entry['entry_id'],'input_sha256':entry['sha256'],'size':entry['size'],'decoded_sha256':sha(database),'decoded_size':len(database),'animation_count':count})
assert mem[0x4074+44*16]==0
text=(P/'private/integration/menu/text.dat').read_bytes()
te=next(e for e in manifest if e['entry_id']=='root/2/disc/592');assert sha(text)==te['sha256']
names=text.split(b'\0')[:128];assert len(names)==len(set(names))==128 and all(0<len(n)<=19 for n in names)
def generate(seed,ring,draw_count,campaign,prior,scratch,names):
 assert 0<=seed<=65535 and 0<=ring<32 and 0<=draw_count<=0xffffffff
 assert len(prior)==32 and all(len(r)==36 for r in prior) and len(scratch)==36
 assert len(names)==128 and len(set(names))==128 and all(0<len(n)<=19 and b'\0' not in n for n in names)
 records=[bytearray(r) for r in prior];scratch=bytearray(scratch);trace=[]
 for r in records:r[0]=0
 def draw(tag):
  nonlocal seed,ring,draw_count
  before=seed;slot=ring;seed=seed*0xe62d&65535;ring=(ring+1)&31;draw_count=(draw_count+1)&0xffffffff
  trace.append({'code':tag,'slot':slot,'seed_before':before,'seed_after':seed,'draw_count_after':draw_count})
  return seed>>8
 for category in range(4):
  start=draw(0x56)%100
  for ordinal in range(5):
   style=(start+ordinal)%4;difficulty=0 if campaign else ordinal
   stat19=min(5*difficulty+[15,40,10,30][style]+draw([0xc4,0xcb,0xd2,0xd9][style])%[25,10,15,20][style],cap19[style])
   stat17=min(10*difficulty+25+draw(0xe6)%10,cap17[style])
   used=[bytes(r).split(b'\0',1)[0] for r in records if r[0]]
   used_style=sum(bool(r[0]) and r[0x15]==style for r in records)
   rank=draw(0xef)%(32-used_style)
   unused=[n for n in names[32*style:32*(style+1)] if n not in used]
   assert len(unused)==32-used_style
   name=unused[rank];scratch[:20]=name.ljust(20,b'\0');scratch[0x14]=0;scratch[0x15]=style;scratch[0x16]=category;scratch[0x17]=stat17;scratch[0x19]=stat19;scratch[0x1a]=stat19;scratch[0x1c:0x24]=bytes(8)
   records[next(i for i,r in enumerate(records) if not r[0])][:]=scratch
 assert len(trace)==64 and sum(bool(r[0]) for r in records)==20
 return {'records_hex':[bytes(r).hex() for r in records],'scratch_hex':scratch.hex(),'seed_after':seed,'ring_after':ring,'draw_count_after':draw_count,'metadata_count':20,'draws':trace}
synthetic=[f'n{i:03d}'.encode() for i in range(128)]
prior=[bytes([i+1])*36 for i in range(32)];scratch=bytes(range(36))
vectors=[]
for seed,campaign in itertools.product([0,1,5428,65535],(False,True)):
 r=generate(seed,31,0xfffffffe,campaign,prior,scratch,synthetic)
 for i,h in enumerate(r['records_hex']):
  b=bytes.fromhex(h)
  if i<20:assert b[24]==24 and b[27]==27 and b[22]==i//5
  else:assert b[0]==0 and b[1:]==prior[i][1:]
 assert r['ring_after']==31 and r['draw_count_after']==62
 vectors.append({'seed_before':seed,'ring_before':31,'draw_count_before':0xfffffffe,'campaign':campaign,'prior_hex':[b.hex() for b in prior],'scratch_before_hex':scratch.hex(),'names_ascii':[n.decode() for n in synthetic],'result':r})
fullcases=0
for seed,campaign in itertools.product(range(128),(False,True)):
 r=generate(seed,0,0,campaign,prior,scratch,synthetic);rows=[bytes.fromhex(h) for h in r['records_hex'][:20]]
 assert len(set(b[:20] for b in rows))==20 and [b[22] for b in rows]==[i//5 for i in range(20)]
 assert all(b[23]<=cap17[b[21]] and b[25]<=cap19[b[21]] for b in rows);fullcases+=1
actual=generate(5428,0,0,True,prior,scratch,names)
folder=G/'blueprint-init';folder.mkdir(exist_ok=True)
payload=(json.dumps(actual,indent=2)+'\n').encode();(folder/'english-explicit-seed-fixture.json').write_bytes(payload)
(folder/'caps.bin').write_bytes(caps)
fixture_payload=(json.dumps(vectors,indent=2)+'\n').encode();(folder/'synthetic-vectors.json').write_bytes(fixture_payload)
def fresh(player,target):
 b=bytearray(0x70d);b[0]=2
 for o in (1,0x4a1,0x6e2):b[o]=player
 struct.pack_into('<hh',b,9,*target)
 for o,v in [(0x67e,1),(0x705,1),(0x62a,0x11800)]:struct.pack_into('<I',b,o,v)
 b[0x6ae]=4;b[0x6e0]=1;b[0x6e9:0x6f0]=label
 return b
fresh_vectors=[{'player':p,'target':t,'after_sha256':sha(fresh(p,t))} for p,t in itertools.product(range(4),[(0,0),(20736,13568),(-32768,32767)])]
for p,t in itertools.product(range(4),[(0,0),(20736,13568),(-32768,32767)]):
 retained=fresh(p,t);source=bytearray([0xa5])*0x70d;restored=source.copy()
 for a,z in [(0,0x4a1),(0x61a,0x6f7)]:restored[a:z]=retained[a:z]
 assert restored[0x4a1]==restored[0x705]==0xa5
# Import the actual production module directly: no copied implementation or
# Cargo/library export dependency. Proprietary names/output remain private.
production=P/'rust/crates/genewars-runtime/src/blueprint.rs'
harness=folder/'native-blueprint-check.rs'
harness.write_text(r'''#[path="'''+production.as_posix()+r'''"] mod blueprint;
use blueprint::*;
use std::io::{Read,Write};
fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let a=*p;*p+=n;&b[a..*p]}
fn u16v(b:&[u8],p:&mut usize)->u16{u16::from_le_bytes(take(b,p,2).try_into().unwrap())}
fn u32v(b:&[u8],p:&mut usize)->u32{u32::from_le_bytes(take(b,p,4).try_into().unwrap())}
fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let n=u32v(&b,&mut p);
let mut banks=Vec::new();for _ in 0..2{let mut bank=Vec::new();for _ in 0..128{let l=take(&b,&mut p,1)[0] as usize;bank.push(take(&b,&mut p,l).to_vec());}banks.push(bank);}
let banks:Vec<_>=banks.iter().map(|b|BlueprintNames::new(&b.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap()).collect();
let mut out=Vec::new();for _ in 0..n{let seed=u16v(&b,&mut p);let ring=take(&b,&mut p,1)[0];let draw_count=u32v(&b,&mut p);let campaign=take(&b,&mut p,1)[0]!=0;let bank=take(&b,&mut p,1)[0] as usize;
let mut prior=[[0;36];32];for r in &mut prior{r.copy_from_slice(take(&b,&mut p,36));}let scratch=take(&b,&mut p,36).try_into().unwrap();
let r=generate_blueprints(&prior,scratch,&banks[bank],BlueprintRng{seed,ring,draw_count},campaign).unwrap();for a in r.records{out.extend(a);}out.extend(r.scratch_after);out.push(r.metadata_count);out.extend(r.rng_after.seed.to_le_bytes());out.push(r.rng_after.ring);out.extend(r.rng_after.draw_count.to_le_bytes());
assert_eq!(r.draws.len(),64);for d in r.draws{let code:u16=match d.purpose{DrawPurpose::CategoryStyleStart{..}=>86,DrawPurpose::Stat19{style,..}=>[196,203,210,217][style as usize],DrawPurpose::Stat17{..}=>230,DrawPurpose::NameRank{..}=>239};out.extend(d.seed_before.to_le_bytes());out.extend(d.seed_after.to_le_bytes());out.push(d.high_byte);out.push(d.ring_slot);out.extend(d.draw_count_after.to_le_bytes());out.extend(code.to_le_bytes());}}
assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}
''',encoding='utf-8')
seeds=list(range(128))+[5428,32768,65534,65535];native_cases=[];expected=bytearray()
for case_id,(seed,campaign,bank) in enumerate(itertools.product(seeds,(False,True),(0,1))):
 before=[bytes((i*13+j*7+case_id)&255 for j in range(36)) for i in range(32)]
 stack=bytes((case_id*11+j*17)&255 for j in range(36));ring=case_id%32;count=(0xfffffff0+case_id)&0xffffffff
 result=generate(seed,ring,count,campaign,before,stack,[synthetic,names][bank])
 native_cases.append(struct.pack('<HBIBB',seed,ring,count,campaign,bank)+b''.join(before)+stack)
 expected.extend(b''.join(bytes.fromhex(h) for h in result['records_hex']));expected.extend(bytes.fromhex(result['scratch_hex']));expected.extend(struct.pack('<BHBI',result['metadata_count'],result['seed_after'],result['ring_after'],result['draw_count_after']))
 for draw in result['draws']:expected.extend(struct.pack('<HHBBIH',draw['seed_before'],draw['seed_after'],draw['seed_after']>>8,draw['slot'],draw['draw_count_after'],draw['code']))
native_input=bytearray(struct.pack('<I',len(native_cases)))
for bank in [synthetic,names]:
 for name in bank:native_input.extend(bytes([len(name)])+name)
native_input.extend(b''.join(native_cases))
sys.path.insert(0,str(P/'scripts'));from cargo import environment
rustc=P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';native_exe=folder/'native-blueprint-check.exe'
subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(harness),'-o',str(native_exe)],env=environment(),check=True,capture_output=True)
native=subprocess.run([str(native_exe)],input=native_input,capture_output=True,check=True).stdout
assert native==expected,'actual production blueprint outputs or complete RNG trace differ'
native_receipt={'production_path':str(production.relative_to(P)).replace('\\','/'),'production_sha256':sha(production.read_bytes()),'harness_private_path':'private/ghidra/blueprint-init/native-blueprint-check.rs','harness_sha256':sha(harness.read_bytes()),'cases':len(native_cases),'complete_record_bytes_per_case':32*36,'rng_draws_per_case':64,'compared_bytes':len(expected),'output_sha256':sha(native),'scope':'Actual Rust module direct import; full32records/scratch/metadata/rngstates/all64drawdetails compared to independent Python model; synthetic and pinnedEnglish names. No original code execution.'}
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,
 'fresh_player':{'record_bytes':0x70d,'scope':'Selected record cleared by7C870 then byte2=0 path80460; explicit selectedplayer0..3 and initialtarget input','label':{'mapped_va':'0x10eade','bytes_with_nul':7,'sha256':sha(label),'ascii_without_nul':'Player'},'stores':'byte0=2;1/4A1/6E2=player;6E3=0;target9=explicitwordpair;dword67E=1;freshentry6AE=4,count6E0=1,dword705=1;dword62A=11800;progress6E7=0;label6E9..6EF','restore_ranges':[[0,0x4a1],[0x61a,0x6f7]],'restore_excludes':['4A1','705'],'vectors':fresh_vectors},
 'generator':{'entry':'0x7f580','helper':'0x7fad0','player_count_byte':'player+15hex','list_base':'player+16hex','record_count':32,'stride':36,'written_records':20,'scratch_required_bytes':36,'preserved_scratch_offsets':[24,27],'scratch_reused_for_all20':True,'clear_policy':'Onlyrecordfirstbytefor32records, pluslistcount=0. Trailing12otherbytespreserved. First20slotsfilledascending.','rng':'Explicitcurrentu16seed,ring0..31,countu32. Eachdrawseed=(seed*E62D)&FFFF; highbyte=seed>>8; ring=(ring+1)&31;countwrap+1. Exactly64draws:onecategorystartthen3perrecord. Loggingcode/tick/stringwritesremainexplicitrequests.','style':'Forcategory0..3, start=nextHigh%100;ordinal0..4;style=(start+ordinal)%4. difficulty=0ifcampaign,elseordinal.','stat19':'min(5*difficulty+base[15,40,10,30][style]+nextHigh%mod[25,10,15,20][style],cap19[style])','stat17':'min(10*difficulty+25+nextHigh%10,cap17[style])','name_choice':'nextHigh%(32-currentactivecountwithsamestyle); rank-thnamefromstyle32-namegroupabsentamongall32recordsusingNULaware20-bytecomparison. Copy20byteswithNULpadding. Safeadmission128uniquenonemptyNUL-freenameslength1..19; originaldoesnotvalidatepool.','writes':'name0..13hex padded20;14=0;15style;16category;17stat17;19stat19;1Astat19;1C..23zero.18and1Bunchanged. Copyall36bytesincludingthoseopaquepriorbytes.','log_codes':{'category_start':86,'style_stat19':[196,203,210,217],'stat17':230,'name_rank':239}},
 'caps':{'mapped_va':'0x104af0','file_offset':'0x1268f0','size':64,'sha256':sha(caps),'private_path':'private/ghidra/blueprint-init/caps.bin','stride':16,'cap17_i32':cap17,'cap19_i32':cap19,'other8bytes_perrow':'opaque for this generator'},
 'name_source':{'entry_id':te['entry_id'],'source_sha256':sha(text),'first_index':0,'count':128,'style_group_size':32,'max_name_bytes':max(map(len,names)),'all_unique':True,'encoding':'opaque originalbytes; do not inferUTF8'},
 'resources':resources,'resource_contract':'1E220 loads44-byte descriptors104074..104334 via154B0 when1168F4NULL, then98A60 extracts animation pointers.56E20 uses signedstyle+15/signedcategory+16: key=4*style+category, database1168F4[key],animation116934[key]. Safe0..3both. Bind opaque Rust resource key, not native pointer or wholeactor. No rendering codec claim here.',
 'actor_init':'4FD30 partialwrites actorprior200B: kindbytes2/5=0(indicespreserved),type6=3,owner18,globalIDF,position7=explicit112D10,terrainheight11/shade17 sampledTHERE,destinationBzero,flags15=0280,anim19/frame1B/subframe1A/delay1Dzero,state1C1,angles39=(0,512,0),velocity2D/2F/31zero,acceleration33/35/37zero,physicalheight270,width2B50,avoid53=1,byte54=0,oldbyte4A&3B,blueprintindex74,flags700,field680,blueprintC0handle. Unmentionedbytespreserved.56E20 zeroes94..BF,setsinternalB0handleC4andC4database/1Fanimationhandlesfromstyle/category.80830 no-unit placement overwritespositiontarget,sets16mask08and71mask04,binds56E20,animation8/frame0/state1C1/delay1D0; doesnotrecomputeheight11. Later72960shade refresh is separate.',
 'fixtures':{'synthetic_private_path':'private/ghidra/blueprint-init/synthetic-vectors.json','synthetic_sha256':sha(fixture_payload),'synthetic_vectors':8,'actual_names_private_path':'private/ghidra/blueprint-init/english-explicit-seed-fixture.json','actual_names_fixture_sha256':sha(payload),'actual_fixture_scope':'Explicitseed5428/priorpatterns/scratch, notobservedliveinitialstate'},
 'full_generator_cases':fullcases,'native_production_comparison':native_receipt,
 'limits':['No assumed currentseed/RNGlogring/profile/scratchbytes.','Generation and resourcebinding do not setactorreadybit. Landingactivationremainsseparate.','Actual namebytes, decoded resources and native afterimage fixture stayprivate.','No whole startup or actor movement tick.']}
(P/'research/ghidra/blueprint-init-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(f'PASS14pins;caps64Braw/mapped;16CDresourcekeys;128uniquenames;{fullcases}fullgenerators;8bytefixtures;12freshrestorevectors;{len(native_cases)}nativefulloutputsandtraces')
