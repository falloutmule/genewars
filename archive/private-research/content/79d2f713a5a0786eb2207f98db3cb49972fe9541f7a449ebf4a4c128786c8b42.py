"""Static mickey-counter callback, software cursor and Options keyboard route."""
from pathlib import Path
import hashlib
import json
import struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
contracts=[]
def pin(name,lines=()):
 b=(G/name).read_bytes()
 for line in lines:assert line in b.decode(),(name,line)
 contracts.append({'private_path':'private/ghidra/'+name,'sha256':sha(b)})
pin('mouse-callback-refs/instructions-000a3164.txt',['000a327b MOV EDX,dword ptr [0x0019cf70]','000a3286 SAR EDX,0x1','000a328a CALL 0x000a3458','000a32d0 MOV EBX,0xc','000a32d5 MOV ECX,0x7f','000a32da MOV EDI,0xa3524'])
pin('mouse-callback/instructions-000a3524.txt',['000a36b4 MOV word ptr [0x0019f358],SI','000a36bb MOV word ptr [0x0019f35a],DI','000a36d4 MOVSX EAX,word ptr [0x0019f358]','000a36e1 SAR EBX,0x10','000a36e4 SUB EAX,EBX','000a3779 MOVSX EAX,word ptr [0x0019f35a]','000a3789 SUB EAX,EBX','000a381a CMP dword ptr [0x0019f350],0x0','000a3823 CMP dword ptr [0x0019d220],0x0','000a3831 ADD dword ptr [0x0019cf9c],EAX','000a383c ADD dword ptr [0x0019cfa0],EAX'])
pin('mouse-motion/instructions-000a710c.txt',['000a710c MOV DS,word ptr CS:[0xa7115]','000a7114 RET'])
pin('mouse-callback-refs/instructions-000a3118.txt',['000a312c CMP EAX,0x1','000a3131 CMP EAX,0x3f','000a3152 MOV [0x0019e340],EAX'])
pin('mouse-callback-refs/instructions-000a3458.txt',['000a349f MOV [0x0019cf9c],EAX','000a34b2 MOV [0x0019cfa0],EAX','000a34d6 MOV dword ptr [0x0019f350],EDI'])
pin('mouse-motion/instructions-000a340c.txt',['000a341b MOV [0x0019cf84],EAX','000a3426 MOV dword ptr [0x0019cf8c],EBX','000a3436 MOV dword ptr [0x0019cf90],ECX'])
pin('mouse-motion/instructions-000a3884.txt',['000a3896 JL 0x000a38a3','000a38a2 DEC ECX','000a38cf DEC EBX'])
pin('mouse-motion-refs/instructions-000a2ed4.txt',['000a2f37 CMP dword ptr [0x0019f364],0x0','000a2f45 ADD dword ptr [0x0019cf9c],EAX','000a2f50 ADD dword ptr [0x0019cfa0],EAX','000a2f89 MOV dword ptr [0x0019f350],EDI'])
pin('packet-fields/instructions-00070e40.txt',['00070ecb MOV [0x00118771],AX','00070ed7 MOV [0x00118773],AX'])
pin('mouse-motion/instructions-00013b10.txt',['00013b10 SUB EAX,dword ptr [EAX + -0x4]','00013b13 JMP 0x00024200'])
pin('mouse-motion/instructions-00023a80.txt',['00023a94 MOV dword ptr [EBP + 0x9],EAX','00023b7b MOV byte ptr [EBP + 0x126],0xff'])
pin('mouse-motion/instructions-00024200.txt',['00024211 CALL 0x000a87d0','000245e8 MOV EDX,dword ptr [EDX + 0x9]','000245f0 MOV dword ptr [0x00116978],EDX','00024712 JMP dword ptr CS:[EAX*0x4 + 0x241d8]'])
pin('input-scheduling/instructions-00070b00.txt',['00070bcc CALL dword ptr [EDX + 0x8]','00070df8 MOV AL,[0x0019d0e4]','00070dfd CMP AL,0x19','00070e07 CMP AL,0x23','00070e0d CMP AL,0x4a','00070e17 CMP AL,0x10','00070e21 MOV byte ptr [0x0019d0e4],BH'])
pin('ui-input/instructions-000a87d0.txt',['000a87f2 MOV ECX,dword ptr [0x00118970]','000a881a MOV AL,byte ptr [ECX + 0x8]'])
pin('mouse-motion/keyboard-irq.txt',['000a6b5a IN AL,DX','000a6b5b MOV [0x0019d0e4],AL'])
code=(G/'validated/memory-.object1.bin').read_bytes();mem=(G/'validated/memory-.object3.bin').read_bytes()
assert struct.unpack_from('<I',mem,0x12838+8)[0]==0x13b10
handlers=list(struct.unpack_from('<10I',code,0x241d8-0x10000));assert handlers[9]==0x245b6

def trunc(a,d):return abs(a)//d*(-1 if a<0 else 1)
def register_axis(current,previous,remainder,sensitivity):
 d=current-previous
 if not d:return 0,remainder
 if d>0:
  z=d+remainder;q=trunc(z,sensitivity);r=z-q*sensitivity
 else:
  q=trunc(d,sensitivity);r=remainder+d-q*sensitivity
  if r<0:
   q-=1;r=sensitivity+(r-trunc(r,sensitivity)*sensitivity)
 return q,r

def axis(current,previous,remainder,sensitivity):
 assert -32768<=current<=32767 and -32768<=previous<=32767
 assert 1<=sensitivity<=63 and 0<=remainder<sensitivity
 q,r=divmod(current-previous+remainder,sensitivity)
 return q,r

cases=0
values=[-32768,-32767,-16384,-257,-256,-255,-1,0,1,255,256,257,16383,16384,32766,32767]
for current in values:
 for previous in values:
  for sensitivity in range(1,64):
   for remainder in range(sensitivity):
    assert axis(current,previous,remainder,sensitivity)==register_axis(current,previous,remainder,sensitivity)
    cases+=1

def motion(event,absolute,mickeys,previous=(0,0),position=(320,240),sensitivity=(1,1),remainder=(0,0),active=True,inhibited=False):
 if not active or not event&1:return {'position':list(position),'previous':list(previous),'remainder':list(remainder),'delta':[0,0]}
 out=[axis(mickeys[i],previous[i],remainder[i],sensitivity[i]) for i in range(2)]
 delta=[v[0] for v in out]
 pos=list(position) if inhibited else [max(0,min([639,479][i],position[i]+delta[i])) for i in range(2)]
 return {'position':pos,'previous':list(mickeys),'remainder':[v[1] for v in out],'delta':delta}

vectors=[]
for event,absolute,mickeys,inhibited in [(1,[10,20],[0,0],False),(1,[600,400],[0,0],False),(2,[10,20],[0,0],False),(3,[10,20],[40,-20],False),(1,[320,240],[400,400],False),(1,[320,240],[40,-20],True)]:
 vectors.append({'event_mask':event,'absolute_cx_dx':absolute,'signed_si_di':mickeys,'immediate_inhibition':inhibited,'output':motion(event,absolute,mickeys,inhibited=inhibited)})
assert vectors[0]['output']['position']==vectors[1]['output']['position']==[320,240]
assert vectors[3]['output']['position']==[360,220]
assert vectors[4]['output']['position']==[639,479]
receipt={'candidate_sha256':candidate,'instruction_contracts':contracts,
 'registration':'A3164 registers INT33 function0C,mask007F,callbackA3524. A710C changesDSonly; incomingSI/DI remainavailable.',
 'absolute_vs_relative':'Callbackmovement consumes SI/DI words, not CX/DX absolute coordinates. Copiesoldcounterwords, storesnewSI/DI, subtractssignextendedpreviousfromsignextendedcurrent independently. No16-bitwrap correction inthat subtraction.',
 'axis_formula':'For sensitivity1..63 and remainder0..s-1: (delta, new_remainder)=divmod(current_i16-previous_i16+remainder,s), floor division/nonnegative remainder. Difference computed in signed32. Current andprevious eachsigned16.',
 'gates':'active19D31C nonzero andeventmask01 required for countermovement. Immediateadditiontoowncursor19CF9C/A0 onlywhen19F350==0 and19D220==0. Newcounters/deltas stillstoredbeforeimmediategates. A2ED4 canlaterapplypending19F35C/360 deltas when19F364set, thenclamps/resets; do notinfer permanentfreezefromimmediategatealone.',
 'software_cursor':'InitializationA3458 setsownpositiontohalfofdisplaywidth/height (320,240for640x480), withoutDOSfunction4positioncall. A340Csetsownclamporigin0,size640x480; A3884clamps0..639/0..479. Packet70E40copiesowncurrentlowwords. Buttons latchownpositions, notincomingCX/DX.',
 'hypothesis':'If hostabsoluteeventschangeCX/DXorbuttonsbutleaveSI/DI unchanged, owncursorremainsatsoftwarecenter whilebuttonlatcheswork. This is a static conditionalexplanation; actualDOSBox/hostcallbackregistervaluesnotobservedyet.',
 'options_keyboard':'Module18653C:vtable112838+8=13B10->24200. Optionsinitializationsetseditingindex+126=-1; handlerusesmousemanagerA87D0. ControlID9branch245B6restorespreviousmodulefromobject+9. No standaloneEscapeexitbranch inOptionshandler/mousemanager. Global70B00 dispatchesevent19D0E4only10,19,23,4A; event01falls throughandclears. Activeeditfield157A0hasseparatekeyboardbehavior, outsideclaim.',
 'keyboard_input_source':'IRQrangeA6B44 readsport60andstoresrawbyte19D0E4; options Escape non-exit alone doesnotprove missingkeyboarddelivery.',
 'interface_documentation':{'url':'https://raw.githubusercontent.com/FDOS/mouse/master/int33.lst','checked':'INT33 function0C receivesAXcondition,BXbuttonstate,CX/DXcursorposition,SI/DI mickeycounts; mask0movement,1/2leftedges. No documentation codecopied.'},
 'axis_cases':cases,'vectors':vectors,
 'next_probe':'IndependentemptyDOSprogram: callbackregistration0C/007F; logAX,BX,CX,DX,SI,DI andsequenceinmemoryonlyduringcallback; outputafterunregister. Comparehostabsoluteinjectionversusrelative/capturedmovement. NoINT330Bpollingduringcapture, nooriginalassets/exe.',
 'limits':['Actualemulatormotionpathnotdiagnoseduntilregisterloggerreceipt.','No claimallmouseinhibitionstatescurrentlyzero.','Originalasyncinterrupt/foregroundracesnotmodeled.','StandaloneOptionsEscapeconclusionexcludesactivefieldediting/modalUI.']}
(P/'research/ghidra/mouse-motion-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS15pins; {cases} literal/divmodaxis comparisons;6motionvectors;OptionsID9returnandrawkeyboarddispatch')
