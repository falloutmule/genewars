"""Pin menu residence timer and explicit intro/mouse initialization paths."""
from pathlib import Path
import copy, hashlib, json, struct

P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda data:hashlib.sha256(data).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
pins=[]
def pin(name,*lines):
 data=(G/name).read_bytes()
 for line in lines:assert line in data.decode(),(name,line)
 pins.append({'private_path':'private/ghidra/'+name,'sha256':sha(data)})

pin('font-menu-state/instructions-0007c6a0.txt','0007c787 CALL 0x00080130','0007c7db CALL 0x000a4f96','0007c7e0 MOV dword ptr [ESI + 0x14],EAX')
pin('font-menu/instructions-0007c870.txt','0007ca30 CALL 0x000a4f96','0007ca3d SUB EAX,EDX','0007ca41 DIV EBX','0007ca43 CMP EAX,0x1e','0007ca46 JBE 0x0007ca60','0007ca4d MOV ECX,0x186f2c','0007ca54 MOV dword ptr [0x00116978],ECX')
pin('font-menu/instructions-0007ca70.txt','0007ca8c CALL 0x000a85d0')
pin('menu-replay/instructions-0007c7f0.txt','0007c83e MOV dword ptr [0x0019ff64],EDX')
pin('ui-input/instructions-000a87d0.txt','000a87f2 MOV ECX,dword ptr [0x00118970]')
pin('menu-replay/instructions-000a4f96.txt','000a4f96 JMP 0x000bba90')
pin('menu-replay/instructions-000bba90.txt','000bba93 CALL 0x000bba2a','000bbaa4 LEA EDX,[EAX + 0x83d600]','000bbab2 SUB EAX,ECX')
pin('menu-replay/instructions-000bba2a.txt','000bba2e MOV AH,0x2c','000bba38 CALL 0x000d4427','000bba3f MOV DL,byte ptr [ESP + 0x9]','000bba4f MOV DL,byte ptr [ESP + 0x8]','000bba63 MOV AL,byte ptr [ESP + 0xd]','000bba7a MOV AL,byte ptr [ESP + 0xc]')
pin('menu-replay/instructions-000d4427.txt','000d4444 INT 0x21','000d444e MOV dword ptr [EDI + 0x8],ECX','000d4451 MOV dword ptr [EDI + 0xc],EDX')
pin('terrain-constructor/instructions-0008db90.txt','0008ddd6 MOV dword ptr [EAX],0x112798','0008ddca MOV dword ptr [0x00186f24],ECX','0008dc46 MOV dword ptr [EAX + 0x184fe8],EBP')
pin('menu-replay/instructions-0008de50.txt','0008de50 SUB EAX,0x8','0008de53 JMP 0x0007cb20','0007cb5e CALL 0x0001bdb0')
pin('menu-replay/instructions-0008de30.txt','0008de33 JMP 0x0007cb80','0007cb88 MOV ECX,0x185008','0007cb8f MOV dword ptr [0x00116978],ECX')
pin('menu-replay/instructions-0008de40.txt','0008de43 JMP 0x0007cb70')
pin('menu-replay/instructions-0008de20.txt','0008de23 JMP 0x0007cba0')
pin('menu-replay/instructions-0001bdb0.txt','0001bdfe MOV EAX,0x10d138','0001be0d MOV EAX,0x10d147','0001be20 MOV EAX,0x10d15a','0001be03 CALL 0x0001be50','0001be16 CALL 0x0001be50','0001be29 CALL 0x0001be50')
pin('font-menu-state/instructions-0001be50.txt','0001be5f TEST byte ptr [0x00118968],0x8','0001be91 CALL 0x000979a0','0001bea4 CALL 0x000a3164','0001bf5d CALL 0x000a4fc0','0001bf6b TEST byte ptr [0x00118969],0x20','0001bf85 CALL 0x000979a0','0001bfa6 CALL 0x000a3164')
pin('menu-replay/instructions-000a4fc0.txt','000a4fdb CALL 0x000a51c8','000a4ff1 CALL 0x000a4ff8')
pin('menu-replay/instructions-000a4ff8.txt','000a51aa CMP byte ptr [0x0019cfb6],0x0')
pin('menu-replay/instructions-000a51c8.txt','000a52fc CMP byte ptr [0x0019d01d],0x0')
pin('mode-functions/instructions-000979a0.txt','000979a7 CALL 0x000a13f0')
pin('mode-functions/instructions-000a13f0.txt','000a1417 CALL 0x000a33a8','000a14f2 MOV [0x0019cf6c],EAX','000a1502 MOV [0x0019cf70],EAX')
pin('mouse-motion-refs/instructions-000a33a8.txt','000a33e7 CALL 0x000982c0','000a33f9 MOV dword ptr [0x0019d31c],ESI','000a33ff MOV dword ptr [0x0019f350],ESI')
pin('mouse-callback-refs/instructions-000a3164.txt','000a3175 CMP dword ptr [0x0019d31c],0x0','000a3188 CMP dword ptr [0x0019f354],0x0','000a31e2 MOV dword ptr [0x0019e340],ECX','000a3211 MOV dword ptr [0x0019f350],EBX','000a325d CALL 0x000a3118','000a328a CALL 0x000a3458','000a32c7 CMP dword ptr [0x0019f354],0x0','000a330e MOV dword ptr [0x0019f354],EBP')
pin('menu-replay-refs/instructions-000a3118.txt','000a3152 MOV [0x0019e340],EAX','000a3157 MOV dword ptr [0x0019e344],EDX')
pin('mouse-motion-refs/instructions-000a3458.txt','000a349f MOV [0x0019cf9c],EAX','000a34b2 MOV [0x0019cfa0],EAX','000a34d6 MOV dword ptr [0x0019f350],EDI')
pin('mouse-motion-refs/instructions-000a2ed4.txt','000a2f45 ADD dword ptr [0x0019cf9c],EAX','000a2f89 MOV dword ptr [0x0019f350],EDI')
pin('menu-replay-refs/instructions-0001cdf0.txt','0001cf39 CMP BL,0x6c','0001cf3e CMP BL,0x4c','0001cf50 MOV dword ptr [0x0019d220],EDI')
pin('ui-objects/instructions-00080130.txt','00080139 CMP EBX,0x185008','00080146 MOV EAX,0x5','0008019a CALL 0x0007ad10')
pin('menu-replay/instructions-0007ad10.txt','0007ad20 CALL 0x000a2fb8','0007ad2d CALL 0x000a2f98','0007ad34 CALL 0x000a2fb8')
pin('mouse-callback-refs/instructions-000a2f98.txt','000a2fa7 MOV [0x0019e348],EAX')
pin('mouse-callback/instructions-000a3524.txt','000a36b4 MOV word ptr [0x0019f358],SI','000a36fb MOV ESI,dword ptr [0x0019e340]','000a381a CMP dword ptr [0x0019f350],0x0','000a3823 CMP dword ptr [0x0019d220],0x0')
pin('module-transition-refs/instructions-0001bbc0.txt','0001bcc6 CALL dword ptr [EBX + 0x4]','0001bcd1 CALL dword ptr [EBX]','0001bd36 CALL 0x00070b00')

mem=(G/'validated/memory-.object3.bin').read_bytes()
mapped=[]
for va,values in [(0x1127b8,[0,8]),(0x112798,[0x8de50,0x8de40,0x8de30,0x8de20]),(0x111ed4,[0,32]),(0x112898,[0x13c30,0x13c20,0x13c10,0x13c00])]:
 data=mem[va-0x100000:va-0x100000+4*len(values)]
 assert list(struct.unpack('<'+'I'*len(values),data))==values
 mapped.append({'va':hex(va),'length':len(data),'sha256':sha(data),'values':[hex(v) for v in values]})
movies=[]
for va,expected in [(0x10d138,'intro\\logo.smk'),(0x10d147,'intro\\biointro.smk'),(0x10d15a,'intro\\gw.smk')]:
 o=va-0x100000;data=mem[o:mem.index(0,o)+1];assert data[:-1].decode()==expected
 movies.append({'va':hex(va),'relative_path':expected,'length':len(data),'sha256':sha(data)})

def timer(now,start,requested=None):
 elapsed=(now-start)&0xffffffff
 return {'elapsed_u32':elapsed,'quotient':elapsed//100,'next_module':'0x186F2C' if elapsed//100>30 else requested}

vectors=[]
for start in [0,1,0x12345678,0xfffffff0]:
 for delta in [0,2999,3000,3099,3100,3101,0xffffffff]:
  for requested in [None,'0x18653C','0x186F19']:
   now=(start+delta)&0xffffffff;out=timer(now,start,requested)
   assert (out['next_module']=='0x186F2C')==(delta>=3100)
   vectors.append({'start_u32':start,'now_u32':now,'requested_module':requested,'result':out})
clock_cases=0
for hours in range(24):
 for minutes in range(60):
  for seconds in range(60):
   for hundredths in [0,1,99]:
    reg=hours*60+minutes;reg=reg*60+seconds;reg=(reg+reg*24)*4+hundredths
    assert reg==((hours*3600+minutes*60+seconds)*100+hundredths)
    clock_cases+=1

def reinitialize(state,width,height):
 result=copy.deepcopy(state)
 if result['active']:return result
 result.update(active=True,sensitivity=[1,1],position=[width//2,height//2],foreground_inhibition=0)
 result['registration_calls']+=int(not result['registered']);result['registered']=True
 return result

mouse_cases=0;mouse_vectors=[]
for cli_inhibition in [0,1]:
 for registered in [False,True]:
  state={'active':True,'registered':registered,'sensitivity':[7,9],'position':[77,99],'foreground_inhibition':1,'cli_inhibition':cli_inhibition,'previous_mickeys':[123,-456],'remainders':[3,4],'registration_calls':0}
  before=copy.deepcopy(state);assert reinitialize(state,640,480)==before
  for name,w,h in [('logo-low',320,200),('logo-restore',640,480),('bio-low',320,200),('bio-restore',640,480)]:
   state.update(active=False,foreground_inhibition=0)
   state=reinitialize(state,w,h)
   assert state['position']==[w//2,h//2] and state['sensitivity']==[1,1]
   assert state['cli_inhibition']==cli_inhibition and state['previous_mickeys']==before['previous_mickeys'] and state['remainders']==before['remainders']
   mouse_cases+=1
  assert state['registration_calls']==int(not registered)
  mouse_vectors.append({'initially_registered':registered,'cli_inhibition':cli_inhibition,'after_normal_restore':state})

refs=json.loads((G/'menu-replay-refs/reference-graph.json').read_text())
writes=[r for r in refs if r['root']=='0019d220' and r['type']=='WRITE']
assert len(writes)==1 and writes[0]['from']=='0001cf50'
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,'mapped_module_tables':mapped,'movie_paths':movies,
 'timer':{'start_store':'7C7DB/E0: A4F96 -> menu object+0x14, at end of menu initialization','test':'7CA30..7CA54: ((now_u32-start_u32) mod2^32)/100 >30 using unsigned DIV','threshold_centiseconds':3100,'threshold_seconds':31,'activity':'The menu input/draw/exit functions and control-manager call do not reset this timestamp for motion, a press or release. This is a residence timer, not a demonstrated input-reset inactivity timer.','precedence':'Timer runs at the shared tail after action selection. Once expired, it overwrites any earlier next-module request with186F2C in that handler invocation. A terminal process exit need not return to the timer.','clock':'A4F96 jumpsBBA90; BBA2A callsINT21/AH2C throughD4427 and forms ((hours*60+minutes)*60+seconds)*100+hundredths. BBA90 subtracts baseline1A109C, adding8640000 if day counter is below baseline. Clock changes/wrap can also cause unsigned timer underflow; normal31s derivation assumes forward clock progression.'},
 'replay_chain':'8DB90 constructs186F24 descriptoroffset8, so186F2C vtable112798 has init8DE50→7CB20→1BDB0. That calls1BE50 forlogo.smk,biointro.smk,gw.smk. Next module handler8DE30→7CB80 schedulesmenu185008. Module initialization re-enters7C6A0 and refreshes its timestamp. Movie helper can skip playback when118968mask08 set.',
 'mouse_mode_path':{'admission':'Successful mode setup/mouse initialization, playback not skipped, restoreflag118969mask20set. Does not establish actual run006 branch state.','logo_and_bio':'1BE50 low-resolution argumentDL1 calls979A0(mode1,320,200), thenA3164(NULL,1,1). After playback it restores979A0(mode13,640,480) andA3164(NULL,1,1) under restoreflag20.','gw_movie':'Third call has DL0 and no mode switch/reinitialization in1BE50; preserves preceding restored mouse state.','deactivation':'979A0→A13F0 callsA33A8. Successful active teardown sets19D31C=0 and19F350=0; it does not clear already-registered19F354.','reinitialization':'A3164 returns-1 without changes if already active. Otherwise it installs sensitivity1/1, bounds from own display dimensions and centered ownposition (320,240 for640x480); clears19F350. When19F354 is nonzero it skipsDOSreset/function0C re-registration. Only initial registration sets mask007F/callbackA3524.','counter_preservation':'Known teardown/reinit zeroed regions [19D320,19E340),[19E350,19F350) exclude saved mickeywords19F358/35A and division remainders19D314/318. These routines do not reset those fields. Driver counter behavior across graphics modes remains external.'},
 'menu_cursor':'7C6A0→80130 selects menu cursor record5 at115E74+30, then7AD10 updates image/hotspot throughA2FB8/A2F98. This does not callA3458/recenter orA3118/change sensitivity. Returning to center can result from preceding mode reinitialization, not an observed movement failure diagnosis.',
 'inhibition':{'foreground19F350':'Mode teardown/reinitialization and completed image setter clear it; A2DA0 temporarily sets it during cursor/background drawing andA2ED4 clears it/appends pending motion. No claim about an asynchronous sampled runtime value.','command_line19D220':'Only direct write in reference graph is1CF50, setting1 for command-line L/l. Normal video/menu functions reviewed do not clear it. Direct-reference analysis does not rule out every indirect write; actual process arguments/runtime memory remain external evidence.','movement':'A3524 still needs active19D31C andeventmask01; consumes signedSI/DI, notabsoluteCX/DX. Immediate ownposition addition requires19F350=0 and19D220=0; pending path must also be considered.'},
 'clock_cases':clock_cases,'timer_vectors':vectors,'mouse_transition_cases':mouse_cases,'mouse_transition_vectors':mouse_vectors,
 'diagnostic':'For a future authorized reference session, timestamp the first labelled menu and complete calibrated captured relative movement, visible cursor-position check and press/release within20s; stop before31s. Do not interpret replay afterthatboundary as evidenceofclickactivation. Ifmovement fails, preservecapture/timestamps andstop ratherthanrepeatblindtargetclicks. An independent empty logger could first exercise matching low/high graphics mode changes, since same-mode callback success doesnotprove across-mode driver counters/registration. No original-code modification is proposed.',
 'limits':['No original execution in this lane; no claim about run006 host action success or exact transition cause.','No emulator-driver callback registers measured here; probe012 observations are external.','No whole-menu keyboard behavior or arbitrary graphics/video transitive library semantics emulated.']}
(P/'research/ghidra/menu-replay-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS {len(pins)} pins; {len(mapped)} module tables; {clock_cases} DOS clock arithmetic cases; {len(vectors)} timer vectors; {mouse_cases} gated mouse transitions.')
