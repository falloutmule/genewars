"""Inactive serialized actor copy and direct production initialization oracle."""
from pathlib import Path
import hashlib,itertools,json,struct,subprocess,sys,runpy
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra';sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
pins=[]
def pin(path,*lines):
 b=(G/path).read_bytes()
 for line in lines:assert line in b.decode(),(path,line)
 pins.append({'private_path':'private/ghidra/'+path,'sha256':sha(b)})
pin('mission-tail-readers/instructions-00057540.txt','00057549 CALL 0x0006d490','0005754e MOV ECX,0x6d','00057553 LEA EDI,[EBX + 0x27]','00057576 JZ 0x00057580')
pin('mission-tail-readers/instructions-0006d490.txt','0006d494 MOV ECX,0x1f','0006d4b2 MOV byte ptr [EAX + 0x17],0x20','0006d4b9 MOV byte ptr [EAX + 0x15],BL')
pin('mission-tail-constructors/instructions-0001e400.txt','0001e40d MOV word ptr [EAX + 0x31],0x0','0001e443 MOV dword ptr [EAX + 0x23],0x112194')
pin('mission-tail-constructors/instructions-0001e280.txt','0001e2c9 MOV EBX,0x111f9c','0001e2ce MOV EDX,0x14','0001e2d8 CALL 0x000a76d0')
pin('actor-pool/instructions-000575c0.txt','000575cc MOV EBX,0x8c','000575f0 CALL dword ptr [ECX + 0x3c]','000575f5 ADD ECX,0xc8')
pin('campaign-level/instructions-00071b70.txt','00071fdc IMUL ESI,ESI,0x8c','0007202a CALL 0x000575c0')
pin('player-creation/instructions-0004fd30.txt','0004fd86 MOV EAX,ECX','0004fdb6 MOV word ptr [ECX + 0x15],0x0','0004fed4 MOV byte ptr [EBP + 0x4a],AH','0004fef4 MOV word ptr [ECX + 0x15],0x280','0004ff0e MOV word ptr [ECX + 0x70],0x0','0004ff16 MOV word ptr [ECX + 0x2b],0x32')
pin('player-creation/instructions-00013eb0.txt','00013eb0 MOV byte ptr [EAX + 0x2],0x0')
pin('player-creation/instructions-0001f480.txt','0001f481 MOV word ptr [EAX + 0x2],0x0')
pin('mission-tail-init/instructions-00056e20.txt','00056e5e MOV EBX,0x2c','00056e63 CALL 0x000982c0','00056e6f MOV dword ptr [ECX + 0xb0],EAX','00056e8d MOV dword ptr [ECX + 0xc4],EAX','00056eab MOV dword ptr [ECX + 0x1f],EAX')
pin('camera-player-input/instructions-00080830.txt','00080891 MOV ESI,dword ptr [EDX + 0x23]','00080899 CALL dword ptr [ESI + 0x60]','000808dc OR byte ptr [EDX + 0x71],0x4')
source=(G/'campaign-resources/152.dat').read_bytes();assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
groups=json.loads((P/'research/ghidra/mission-load-evidence.json').read_text())['serialized_groups'];group=groups[2]
assert group['file_offset']==0x48f4e and group['packed_stride']==140 and group['native_stride']==200
packed_pool=source[0x48f4e:0x48f4e+20*140];assert packed_pool==bytes(20*140)
def inactive_copy(prior,packed):
 assert len(prior)==200 and len(packed)==140 and packed[6]==0
 after=bytearray(prior);after[:31]=packed[:31];after[39:148]=packed[31:]
 return after
copy_vectors=[]
for i in range(256):
 prior=bytes((j*7+i)&255 for j in range(200));packed=bytearray((j*13+i)&255 for j in range(140));packed[6]=0
 after=inactive_copy(prior,packed)
 assert after[31:39]==prior[31:39] and after[148:]==prior[148:]
 assert after[:31]==packed[:31] and after[39:148]==packed[31:]
 if i in (0,1,255):copy_vectors.append({'prior_hex':prior.hex(),'packed_hex':packed.hex(),'after_hex':after.hex()})
def initialize(prior,player,local,blueprint,style,category,temporary,height,shade,target):
 assert len(prior)==200 and prior[6]==0 and 0<=player<4 and 0<=local<5 and 0<=blueprint<32 and 0<=style<4 and 0<=category<4
 after=bytearray(prior)
 after[2]=after[5]=0;after[6]=3
 struct.pack_into('<H',after,15,player*5+local)
 struct.pack_into('<hh',after,7,*temporary);struct.pack_into('<i',after,17,height);after[23]=shade
 after[11:15]=bytes(4);struct.pack_into('<H',after,21,0x280);after[24]=player
 for o in (25,26,27,29):after[o]=0
 after[28]=1;after[39:43]=bytes(4);struct.pack_into('<H',after,43,50)
 after[45:57]=bytes(12);struct.pack_into('<hhh',after,57,0,512,0)
 after[74]&=0x3b;after[83]=1;after[84]=0;after[85]=1;after[104:108]=bytes(4);after[112:114]=bytes(2);after[116]=blueprint
 # 80830 no-unit branch and56E20 handle-free byte projection.
 after[22]|=8;after[113]|=4;struct.pack_into('<hh',after,7,*target)
 after[148:192]=bytes(44);after[25]=8;after[27]=after[29]=0;after[28]=1
 for a,z in [(31,35),(176,180),(192,196),(196,200)]:after[a:z]=bytes(z-a)
 return bytes(after)
production=P/'rust/crates/genewars-runtime/src/actor_initialization.rs';assert production.is_file()
folder=G/'actor-pool';folder.mkdir(exist_ok=True);harness=folder/'native-actor-init-check.rs'
harness.write_text(r'''#[path="'''+production.as_posix()+r'''"] mod actor;
use actor::*;use std::io::{Read,Write};
fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let a=*p;*p+=n;&b[a..*p]}
fn word(b:&[u8],p:&mut usize)->i16{i16::from_le_bytes(take(b,p,2).try_into().unwrap())}
fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let mut out=Vec::new();
for _ in 0..n{let prior=take(&b,&mut p,200);let player=take(&b,&mut p,1)[0];let local=take(&b,&mut p,1)[0];let index=take(&b,&mut p,1)[0];let style=take(&b,&mut p,1)[0];let category=take(&b,&mut p,1)[0];let temporary=[word(&b,&mut p),word(&b,&mut p)];let height=i32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let shade=take(&b,&mut p,1)[0];let target=[word(&b,&mut p),word(&b,&mut p)];let mut record=[0;0x70d];record[1]=player;let q=0x16+36*index as usize;record[q+0x15]=style;record[q+0x16]=category;
let r=plan_no_unit_actor_initialization(prior,player,local as usize,&record,index,temporary,height,shade,target).unwrap();assert_eq!(r.actor_before,prior);assert_eq!(r.resource_key,ActorResourceTableKey{style,category,flat_index:4*style+category});assert_eq!(r.global_actor_id,5*player as u16+local as u16);assert_eq!(r.temporary_origin,temporary);assert_eq!(r.final_target,target);assert_eq!(r.sampled_origin_height,height);assert_eq!(r.sampled_origin_shade,shade);
assert_eq!(r.handle_bindings,[ActorHandleBinding::BlueprintMetadata{actor_offset:0xc0,handle:BlueprintRecordHandle{player_id:player,blueprint_index:index}},ActorHandleBinding::ResourceDatabase{actor_offset:0xc4,key:r.resource_key},ActorHandleBinding::AnimationDatabase{actor_offset:0x1f,key:r.resource_key},ActorHandleBinding::InternalActorField{actor_offset:0xb0,target_actor_offset:0xc4}]);out.extend(r.actor_after_nonpointer);}
assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}
''',encoding='utf-8')
cases=[];expected=bytearray()
heights=[-2147483648,-65536,-1,0,6272,65534,2147483647]
for i,(player,local,index,style,category) in enumerate(itertools.product(range(4),range(5),range(32),range(4),range(4))):
 prior=bytearray((i*17+j*11)&255 for j in range(200));prior[6]=0
 temporary=[-32768+(i*17)%65536,32767-(i*31)%65536];target=[32767-(i*13)%65536,-32768+(i*29)%65536];height=heights[i%len(heights)];shade=i%256
 cases.append(bytes(prior)+struct.pack('<BBBBBhh iBhh',player,local,index,style,category,*temporary,height,shade,*target))
 expected.extend(initialize(prior,player,local,index,style,category,temporary,height,shade,target))
sys.path.insert(0,str(P/'scripts'));from cargo import environment
rustc=P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';native_exe=folder/'native-actor-init-check.exe'
subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(harness),'-o',str(native_exe)],env=environment(),check=True,capture_output=True)
native=subprocess.run([str(native_exe)],input=struct.pack('<I',len(cases))+b''.join(cases),capture_output=True,check=True).stdout
assert native==expected,'actual actor initializer differs from independent instruction-write model'
# Compose the actual owned PlayerState transaction with actual generation,
# selection and initializer modules; compare independent complete afterimages.
model=runpy.run_path(str(P/'scripts/ghidra/validate_blueprint_init_bridge.py'))
generate=model['generate'];names=model['names'];synthetic=model['synthetic']
transaction_harness=folder/'native-player-startup-check.rs'
transaction_sources={name:P/path for name,path in [('player_creation','rust/crates/genewars-core/src/player_creation.rs'),('blueprint','rust/crates/genewars-runtime/src/blueprint.rs'),('actor_initialization','rust/crates/genewars-runtime/src/actor_initialization.rs'),('player_state','rust/crates/genewars-runtime/src/player_state.rs')]}
imports='extern crate self as genewars_core;\npub mod cd_candidate {pub use crate::player_creation::*;}\n'+''.join('#[path="'+path.as_posix()+'"] mod '+name+';\n' for name,path in transaction_sources.items())
transaction_harness.write_text(imports+r'''
use std::io::{Read,Write};use blueprint::*;use player_state::*;use actor_initialization::*;
fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let a=*p;*p+=n;&b[a..*p]}
fn word(b:&[u8],p:&mut usize)->i16{i16::from_le_bytes(take(b,p,2).try_into().unwrap())}
fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let mut banks=Vec::new();for _ in 0..2{let mut a=Vec::new();for _ in 0..128{let l=take(&b,&mut p,1)[0] as usize;a.push(take(&b,&mut p,l).to_vec());}banks.push(a);}let banks:Vec<_>=banks.iter().map(|a|BlueprintNames::new(&a.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap()).collect();let mut out=Vec::new();
for _ in 0..n{let mut player=PlayerState::from_record(take(&b,&mut p,0x70d)).unwrap();let scratch=take(&b,&mut p,36).try_into().unwrap();let seed=u16::from_le_bytes(take(&b,&mut p,2).try_into().unwrap());let ring=take(&b,&mut p,1)[0];let draw_count=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let bank=take(&b,&mut p,1)[0] as usize;let mask=take(&b,&mut p,1)[0];let mut actors=[[0;200];5];for a in &mut actors{a.copy_from_slice(take(&b,&mut p,200));}let position=[word(&b,&mut p),word(&b,&mut p)];let height=i32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let shade=take(&b,&mut p,1)[0];let target=[word(&b,&mut p),word(&b,&mut p)];let generated=player.regenerate_blueprints(scratch,&banks[bank],BlueprintRng{seed,ring,draw_count},true).unwrap();let before=player.clone();let prior=actors;
let result=player.prepare_fresh_actor_partition(&actors,mask,ActorOriginSample{position,height,shade},target).unwrap();assert_eq!(player,before);assert_eq!(actors,prior);assert!(result.landing_factory_pending);assert_eq!(result.initializations.len(),result.selection.created_count as usize);
for r in &result.initializations{let key=r.resource_key;assert_eq!(key.flat_index,4*key.style+key.category);assert_eq!(r.handle_bindings,[ActorHandleBinding::BlueprintMetadata{actor_offset:0xc0,handle:BlueprintRecordHandle{player_id:r.player_id,blueprint_index:r.blueprint_index}},ActorHandleBinding::ResourceDatabase{actor_offset:0xc4,key},ActorHandleBinding::AnimationDatabase{actor_offset:0x1f,key},ActorHandleBinding::InternalActorField{actor_offset:0xb0,target_actor_offset:0xc4}]);}
out.extend(result.player.bytes());for a in result.actors{out.extend(a);}out.push(result.selection.attempt_count);out.push(result.selection.created_count);out.push(result.landing_factory_pending as u8);out.extend(generated.rng_after.seed.to_le_bytes());out.push(generated.rng_after.ring);out.extend(generated.rng_after.draw_count.to_le_bytes());
let mut badbytes=*player.bytes();badbytes[0x16+10*36+0x15]=255;let bad=PlayerState::from_record(&badbytes).unwrap();let badbefore=bad.clone();let rejected=bad.prepare_fresh_actor_partition(&actors,5,ActorOriginSample{position,height,shade},target);assert!(matches!(rejected,Err(PlayerStateError::Initialization(_))));assert_eq!(bad,badbefore);assert_eq!(actors,prior);out.push(1);}
assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}
''',encoding='utf-8')
transaction_inputs=[];transaction_expected=bytearray()
for i in range(528):
 player=i%4;mask=[0,1,2,4,5,15,255,128][i%8];bank=i%2;seed=([0,1,5428,32768,65535]+list(range(128)))[i%133];ring=i%32;count=(0xfffffff0+i)&0xffffffff
 before=bytearray((i*19+j*7)&255 for j in range(0x70d));before[1]=player;before[2]=before[3]=before[0x6e7]=0
 stack=bytes((i*31+j*13)&255 for j in range(36));prior=[bytes(before[0x16+36*k:0x16+36*(k+1)]) for k in range(32)]
 generated=generate(seed,ring,count,True,prior,stack,[synthetic,names][bank]);after=before.copy()
 for k,h in enumerate(generated['records_hex']):after[0x16+36*k:0x16+36*(k+1)]=bytes.fromhex(h)
 after[0x15]=20
 prior_actors=[bytearray((i*11+k*23+j*5)&255 for j in range(200)) for k in range(5)]
 for a in prior_actors:a[6]=0
 actors=[bytes(a) for a in prior_actors];position=[-32768+(i*31)%65536,32767-(i*17)%65536];target=[32767-(i*23)%65536,-32768+(i*43)%65536];height=heights[i%len(heights)];shade=i%256
 indices=([0] if mask&1 else [])+([10] if mask&4 else [])
 for local,index in enumerate(indices):
  q=0x16+36*index;after[q+0x20]|=1
  actors[local]=initialize(actors[local],player,local,index,after[q+0x15],after[q+0x16],position,height,shade,target)
  after[4+local]=5*player+local;after[3]+=1
 struct.pack_into('<hh',after,9,*target)
 transaction_inputs.append(bytes(before)+stack+struct.pack('<HBIBB',seed,ring,count,bank,mask)+b''.join(prior_actors)+struct.pack('<hh iBhh',*position,height,shade,*target))
 transaction_expected.extend(after);transaction_expected.extend(b''.join(actors));transaction_expected.extend(bytes([len(indices),len(indices),1]));transaction_expected.extend(struct.pack('<HBI',generated['seed_after'],generated['ring_after'],generated['draw_count_after']));transaction_expected.append(1)
transaction_input=bytearray(struct.pack('<I',len(transaction_inputs)))
for bank in [synthetic,names]:
 for name in bank:transaction_input.extend(bytes([len(name)])+name)
transaction_input.extend(b''.join(transaction_inputs));transaction_exe=folder/'native-player-startup-check.exe'
subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(transaction_harness),'-o',str(transaction_exe)],env=environment(),check=True,capture_output=True)
transaction_native=subprocess.run([str(transaction_exe)],input=transaction_input,capture_output=True,check=True).stdout
assert transaction_native==transaction_expected,'owned player transaction afterimages differ from independent composition model'
transaction_receipt={'sources':[{'path':str(path.relative_to(P)).replace('\\','/'),'sha256':sha(path.read_bytes())} for path in transaction_sources.values()],'harness_private_path':'private/ghidra/actor-pool/native-player-startup-check.rs','harness_sha256':sha(transaction_harness.read_bytes()),'cases':len(transaction_inputs),'compared_bytes':len(transaction_expected),'output_sha256':sha(transaction_native),'scope':'Actual generator -> ownedPlayerState -> selection -> actorinitialization. Full0x70Dplayer/5x200actors/attemptcounts/landingpending/RNGafterstates+alltypedbindings;528invalidsecondstylescheckatomicrollback.'}
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,
 'source_pool':{'entry_id':'root/2/disc/152','source_sha256':sha(source),'family':2,'file_offset':0x48f4e,'packed_stride':140,'native_base':'0x16360e','native_stride':200,'count':20,'packed_sha256':sha(packed_pool),'all_packed_bytes_zero':True,'copies':[{'source':[0,31],'destination':[0,31]},{'source':[31,140],'destination':[39,148]}],'preserved_prior_native_ranges':[[31,39],[148,200]],'inactive':'byte6zero skips56E20 activeinitializer; no automatictailclear','staging_mutation':'6D490 modifies staging source+15/+17 AFTERcopy. Destinationheader retains originalbytes. Inputsourceartifactmustremainimmutable inport.','constructor':'1E400 zeroesninewordsofvelocity/acceleration/angles2D..3Fandwritesvtable23; readerpayload subsequently overwritesthoseserializedwords. Vtable/resourcehandlesremain typedruntimeinputs.','safety':'Inactiveconversionrequiresprior200bytesandpacked140byteswithbyte6zero. Do notzero-fillruntimegapsoracceptactivecopywithoutresourceinitializer.'},
 'copy_vectors':copy_vectors,'copy_cases':256,
 'initialization':'4FD30+80830no-unit+56E20. Exactfieldcontractinblueprint-init-evidence.json; pointerdestinations1F/B0/C0/C4 representedbyfourtypedbindingswithzerobyteplaceholders, not forgednativeaddresses. Preservedvtable23..27 bytesremainopaque; thisportdoesnotcallthem.',
 'native_comparison':{'production_path':str(production.relative_to(P)).replace('\\','/'),'production_sha256':sha(production.read_bytes()),'harness_private_path':'private/ghidra/actor-pool/native-actor-init-check.rs','harness_sha256':sha(harness.read_bytes()),'cases':len(cases),'compared_bytes':len(expected),'output_sha256':sha(native),'fields':'All200afterimagebytes, priorpreservation, completekey/ID/temporary/finalpoint/height/shade/fourhandlebindings; all4playersx5slotsx32blueprintindicesx4stylesx4categories, variedprior/coordinates/heights/shade'},
 'owned_player_transaction_comparison':transaction_receipt,
 'limits':['No initialruntimegapbytesguessed.','Temporaryoriginheight/shadeareexplicitinputs, notsampledfromfinaltarget.','No slotallocation/playerappend/blueprintflagmutation or landingactivationinsideinitializer.','No claimthatnewactorsareready; word70is0400.']}
(P/'research/ghidra/actor-initialization-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(f'PASS11pins;20allzeropackedactors;256splitcopycases;{len(cases)}nativeactorafterimages/fourbindings;{len(transaction_inputs)}ownedtransactions/secondstylerollbacks')
