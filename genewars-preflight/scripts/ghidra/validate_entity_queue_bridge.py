"""Literal linked queue vs independent order; family-1 explicit runtime packets."""
from pathlib import Path
import hashlib
import json
import struct
import random

P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
contracts=[]
def pin(name,lines=()):
    b=(G/name).read_bytes()
    for line in lines:assert line in b.decode(),(name,line)
    contracts.append({'private_path':'private/ghidra/'+name,'sha256':sha(b)})
pin('display-geometry/instructions-00083140.txt',['0008314e CMP dword ptr [0x00116974],0x1863a2','0008315e MOVSX EAX,word ptr [EAX + 0x11bb]','00083165 CMP EAX,0x100','00083186 CMP EBX,-0x28','000831a6 JLE 0x000836ff','000831bf CMP EDX,-0xa','00083222 MOV EAX,dword ptr [ESP + 0x4]','0008324c MOV ECX,0x1df','000832b5 CMP AX,word ptr [EBX + 0x4]','000832b9 JGE 0x000832f8','0008338d MOV dword ptr [EAX + 0xa3b],ESI','0008355b MOVSX EAX,byte ptr [EBX + 0x11d6]','00083577 TEST byte ptr [EBP + 0x15],0x4','000835a0 OR byte ptr [EBP + 0x15],0x8','000836ed MOV byte ptr [EAX + 0x11d8],0x1'])
pin('display-geometry/instructions-00083930.txt',['000839c1 ADD EDX,EDI','000839c6 MOVSX EBX,DX','000839cd CALL dword ptr [EBP + 0x24]','000839d3 AND byte ptr [EAX + 0x15],0xf3','00083a48 MOV DH,byte ptr [ESI + 0x11d8]','00083a74 CMP byte ptr [ESI + 0x11d5],0x0','00083a80 TEST byte ptr [EAX + 0x15],0x8','00083a8d TEST byte ptr [EAX + 0x15],0x4','00083aca MOV word ptr [ESI + 0x11bb],0x0','00083adc MOV byte ptr [ESI + 0x11d8],0x0'])
pin('terrain-visibility/instructions-00083710.txt',['00083756 MOV byte ptr [0x0010aa15],AH','00083807 TEST BH,0x8','0008381b TEST byte ptr [ECX + 0x15],0x4','00083844 CALL 0x0007a900','00083873 OR DH,0xc','000838bd MOV dword ptr [ESP + 0x8],EAX','000838f5 MOV byte ptr [0x0010aa15],CH'])
pin('entity-order/instructions-0001f440.txt',['0001f440 XOR EAX,EAX','0001f442 RET'])
pin('entity-plant/instructions-0001f280.txt',['0001f280 MOV AL,0x1','0001f282 RET'])
pin('entity-order/instructions-0001e760.txt',['0001e765 CMP dword ptr [EAX + 0x27],0x0','0001e76d MOV byte ptr [0x0010aa15],BL','0001e786 CALL 0x00097a90','0001e78b MOV byte ptr [0x0010aa15],CL'])
pin('entity-order-refs/instructions-0001c120.txt',['0001c73b TEST byte ptr [0x00118770],0x4','0001c748 TEST byte ptr [0x00118968],0x2','0001c836 MOV EBX,0x15d40e','0001c850 ADD EBX,0x31','0001c86f CALL 0x00083140'])
pin('entity-order-refs/instructions-0006d0b0.txt',['0006d0b8 MOVSX ESI,word ptr [0x0011c2b2]','0006d104 ADD EBX,0x31','0006d11d TEST byte ptr [EBX + 0x15],0x1','0006d13f CALL dword ptr [EDX + 0x64]','0006d188 TEST byte ptr [EBX + 0x2e],0x10','0006d195 CALL 0x00083140','0006d1cc AND DH,0xfe','0006d1da CALL 0x00083140'])
pin('entity-order/instructions-0006ca90.txt',['0006cb2f MOV byte ptr [EDI + 0x2f],AL'])
pin('display-geometry/instructions-00082bd0.txt',['00082d70 MOV byte ptr [EDI + 0x11d5],0x1'])
pin('display-geometry/instructions-00086c20.txt',['00086d80 CALL 0x00083710','00086dd8 CALL 0x00083930'])
pin('entity-order/instructions-0007a900.txt',['0007a9c1 CALL 0x0007a810'])
pin('entity-order/instructions-0007a810.txt',['0007a8ea CMP byte ptr [EDX + EBX*0x1 + 0x19d0ec],0x0'])
mem=(G/'validated/memory-.object3.bin').read_bytes();code=(G/'validated/memory-.object1.bin').read_bytes()
word=lambda va:struct.unpack_from('<I',mem,va-0x100000)[0]
assert word(0x1120ac+4)==0x1f440 and word(0x1120ac+0x6c)==0x1f280 and word(0x1120ac+0x70)==0x1e760
assert struct.unpack_from('<I',code,0x83120-0x10000)[0]==0x8355b # native type2 post-insert branch

def admitted(x,y,margin=128,module=True,count=0):
    assert margin in (0,128) and 0<=count<=256
    return module and count<256 and -40<=x<640-margin+40 and -10<=y<520

def linked_queue(inputs,margin=128,module=True):
    heads=[-1]*480;nodes=[]
    for identity,x,y in inputs:
        if not admitted(x,y,margin,module,len(nodes)):continue
        if y<=0:bucket,residual=0,y
        elif y>=480:bucket,residual=479,y-480
        else:bucket,residual=y,0
        index=len(nodes);node={'id':identity,'x':x,'y':y,'bucket':bucket,'residual':residual,'next':-1};nodes.append(node)
        if 0<y<480:
            node['next']=heads[bucket];heads[bucket]=index
        else:
            previous=-1;current=heads[bucket]
            while current!=-1 and residual>=nodes[current]['residual']:
                previous=current;current=nodes[current]['next']
            node['next']=current
            if previous==-1:heads[bucket]=index
            else:nodes[previous]['next']=index
    result=[]
    for bucket,head in enumerate(heads):
        while head!=-1:
            n=nodes[head];result.append({'id':n['id'],'x_i16':n['x'],'draw_y_i16':bucket+n['residual'],'projected_y':n['y'],'bucket':bucket,'residual_i16':n['residual']});head=n['next']
    return result

def ordered_queue(inputs,margin=128,module=True):
    accepted=[]
    for identity,x,y in inputs:
        if admitted(x,y,margin,module,len(accepted)):accepted.append((len(accepted),identity,x,y))
    accepted.sort(key=lambda p:(min(479,max(0,p[3])),p[3],-p[0] if 0<p[3]<480 else p[0]))
    return [{'id':identity,'x_i16':x,'draw_y_i16':y-1 if y>=480 else y,'projected_y':y,'bucket':min(479,max(0,y)),'residual_i16':y if y<=0 else (y-480 if y>=480 else 0)} for _,identity,x,y in accepted]

def admission_flags(flags,shade,countdown):
    assert 0<=flags<=255 and 0<=shade<=255 and -128<=countdown<=127
    if shade==0:return flags,False
    if countdown>0:flags|=4
    if flags&4:flags|=8
    return flags,True

def draw_packet(flags,mode,gate):
    assert 0<=flags<=255 and 0<=mode<=255
    draw=bool(gate and (mode!=0 or flags&8))
    return {'draw':draw,'dirty_class':(3 if flags&4 else 1) if draw else None,'flags_after':flags&0xf3 if draw else flags}

rng=random.Random(83140);cases=0
for margin in [0,128]:
    for size in [0,1,10,255,256,257,512]:
        for iteration in range(16):
            points=[(i,rng.choice([-41,-40,0,511,551,552,679,680]),rng.choice([-11,-10,-1,0,1,2,478,479,480,481,519,520])) for i in range(size)]
            assert linked_queue(points,margin)==ordered_queue(points,margin);cases+=1
synthetic=[(i,0,y) for i,y in enumerate([-1,0,1,1,-1,480,479,480,519,479,0])]
order=linked_queue(synthetic);assert [r['id'] for r in order]==[0,4,1,10,3,2,9,6,5,7,8]
packet_cases=0
for flags in range(256):
    for mode in range(256):
        for gate in [False,True]:
            r=draw_packet(flags,mode,gate)
            assert r['draw']==bool(gate and (mode or flags&8))
            assert r['flags_after']==(flags&243 if r['draw'] else flags)
            packet_cases+=1
vectors=[]
for flags,shade,countdown in [(4,0,1),(4,1,0),(0,1,0),(0,1,1),(16,255,-128),(255,255,127)]:
    f,g=admission_flags(flags,shade,countdown)
    vectors.append({'flags_before':flags,'shade_u8':shade,'countdown_i8':countdown,'flags_after_admission':f,'raises_context_gate':g,'mode0_draw_packet':draw_packet(f,0,g),'mode1_draw_packet':draw_packet(f,1,g)})
points=json.loads((G/'entity-plant/point-fixture.json').read_text())
actual=linked_queue([(r['record'],*r['local_point']) for r in points])
assert actual==ordered_queue([(r['record'],*r['local_point']) for r in points])
private=G/'entity-plant/queue-fixture.json';payload=(json.dumps(actual,separators=(',',':'))+'\n').encode();private.write_bytes(payload)
receipt={'candidate_sha256':candidate,'instruction_contracts':contracts,
 'domain':{'display_pitch':640,'display_height':480,'margin_u16':[0,128],'count_i16':'safe0..256; original onlysigned>=256rejects, negative unsafe count excluded','capacity':256,'active_module':'global116974 ==1863A2','input_coordinates':'explicit signed32 projected point from7AF00; admitted bounds fit signed16','input_sequence':'caller supplied; not implicitly source order; other entity families may consume capacity'},
 'family1':'native type byte+6=2, vtable1120AC:+4=1F440 returns0;+6C=1F280 returns1;+70=1E760 dry dirty-mark oldframe ifcachedpointer+27 nonnull',
 'admission':'x>=-40 AND x<640-margin+40 AND y>=-10 AND y<520, modulegate andcount<256; family1virtualheight0. No perentity shade check before queue insertion.',
 'queue':'y<=0:bucket0,residual=y, sorted signedresidualascending stableequal.0<y<480:buckety,residual0, inserthead/reverseequal. y>=480:bucket479,residual=y-480, sorted signedresidualascending stableequal. Iterate buckets0..479 andlinkedchain; xstoredsigned16, drawY=signed16(bucket+residual). Bottomy>=480 reconstructsy-1.',
 'admission_packet':'Optional separate83140 post-insertion helper, native type2 and context+1C mask01 clear (no selection interaction). Ifshade+17==0 preserveflags and do notraisecontext11D8. Otherwise raise11D8; ifsignedbytecontext11D6>0 setflags+15mask04; ifflags04 thenOR08 andrequestoldframe dirtymark virtual70. Cachedframe-null suppressesdirtycallbody, notflags08.',
 'draw_packet':'83930 family1predicatealways1, selectedentityNone. Requirescontext11D8nonzero; draws iffcontext11D5u8!=0 ORentityflags+15mask08. Onactualdraw dirtyclass10AA14=3 ifflags04else1, callsvirtual24 (1F290), thenflags &=F3. Skippedflags unchanged. Allheads/count/context11D8 resetafterpass.',
 'dirty_prepass':'83710 disablespixelwrite10AA15=0, predicate1, skipsflags08; flags04 bypassesdirtybboxquery. Otherwise1E840 selectsclampedframe and7A900/7A810 checksnonzero32x32dirtyblocks, notpixelopacity. Onaccepted drydraw ORflags0C; restartbucket0 so newlydirty components can propagate. Finally10AA15=1. This prepass is not implemented by the purequeue/drawpacket helpers.',
 'runtime_vs_source':'RawCONQ00140plants have type2/flags04/shade00/animation00/frame00.6C690initializesruntimepointersbutdoesnotrefreshshade.72960updatesnative+17 fromexplicitmaplight/factor;6CA90scheduledvirtual64 mutatesruntimefields;83140and83710alsochangeflags. Rawsource is not a full live-state snapshot.',
 'source_order_routes':'1C120 full-table path when flags118770mask04clear OR118968mask02set inserts active family0 ascending thenfamily1 ascending thenotherfamilies. Ordinaryupdate calls6D0B0: scans ascendinguntilsignedcount11C2B2 activeunflaggedslotsconsumed, maycallvirtual64 forupto2scheduledrecords, excludesnative2E mask10, thensecondpassclearsflags01 fornewrecords. Mustnotassert sourceorder withoutthese runtimegates/counts.',
 'initial_context':'82BD0sets11D5=1; no universalfirstdraw11D8value assumed. Any admittedpositive-shade entity can raiseglobalcontext11D8; zero-shadeplant maystilldraw whenanotherentityraisesit.',
 'ordering_cases':cases+1,'draw_packet_cases':packet_cases,'synthetic_order_vector':{'input_y':[-1,0,1,1,-1,480,479,480,519,479,0],'output':order},'packet_vectors':vectors,
 'actual_fixture':{'source_entry':'root/2/disc/152','camera':[66,48],'margin':128,'input_points':len(points),'admitted_points':len(actual),'private_path':'private/ghidra/entity-plant/queue-fixture.json','sha256':sha(payload),'scope':'Only40explicitfamily1points, emptyqueue, nootherfamilies/capacitycompetition; notcompletefirstscene'},
 'limits':['No complete live-state/first-scene admission or opaque callbacks simulated.','Purequeue assumes fresh empty buckets and count0; inputsequenceexplicit.','Selectedentity/flash interaction excluded byselectedNone andcontext+1Cmask01clear.','Incrementaldirty bookkeeping andprepass need explicitseparatemodel, notassumedallvisible.']}
(P/'research/ghidra/entity-queue-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS13pins; {cases+1} linked/sorted queues; {packet_cases} drawpackets; {len(actual)}/{len(points)}explicitactualpoints admitted')
