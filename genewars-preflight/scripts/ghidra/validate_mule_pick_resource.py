"""Original5A140+1F260+1E840+95300+952A0 over the real pinned MULE.
Source-only preparation emits private TSV for the ignored host-adapter test.
No native selection/order/gameplay or Rust comparison is claimed here.
"""
from pathlib import Path
import argparse, hashlib, importlib.util, json, struct, uuid
P=Path(__file__).resolve().parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 ap=argparse.ArgumentParser();ap.parse_args()
 work=P/'private/review/mule-pick-resource'/uuid.uuid4().hex;work.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 model=m.Source();files=[P/'private/ghidra/mule-picking-148'/f'instructions-000{x}.txt'for x in ['95300','952a0']]+[P/'private/ghidra/entity-plant/instructions-0001e840.txt']
 model.code.update(m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]))
 resource=P/'private/integration/game-runtime/creatures/MULE.ANB';raw=resource.read_bytes();assert sha(resource)=='f6c62221b52a62088f91ae8d00becfef513282cbf98a5990c1a0859cd8b2d346'
 data=bytearray(raw);base=0x500000;n,tab,comps,frames,anims=struct.unpack_from('<H4I',data)
 # Reproduce the established ANB loader's pointer relocation and signed >>1
 # component coordinate conversion. Retain exact sprite descriptors/pixels.
 for i in range((frames-comps)//10):
  p=comps+10*i;off,x,y=struct.unpack_from('<Ihh',data,p)
  struct.pack_into('<Ihh',data,p,base+tab+off,x>>1,y>>1)
 for i in range((anims-frames)//14):
  p=frames+14*i;off=struct.unpack_from('<I',data,p)[0];struct.pack_into('<I',data,p,base+comps+off)
 for i in range(n):
  for j in [0,4]:
   p=anims+10*i+j;off=struct.unpack_from('<I',data,p)[0];struct.pack_into('<I',data,p,base+frames+off)
 resource_state={base+i:v for i,v in enumerate(data)};rows=[];lines=[]
 class Capture(dict):
  def __getitem__(self,pc):
   if pc==0x95300:seen.append(model.r['EAX'])
   instruction=super().__getitem__(pc)
   if instruction[0]=='MOVSB':
    # Exact missing interpreter opcode: one byte copy, direction clear in
    # this source domain, no arithmetic flags changed. MOV EAX,EAX is a
    # dispatch no-op after applying that instruction, not a source port.
    model.store(model.r['EDI'],1,model.load(model.r['ESI'],1));model.r['EDI']+=1;model.r['ESI']+=1
    return ('MOV',['EAX','EAX'])
   return instruction
 model.code=Capture(model.code)
 for animation in range(n):
  first,last,trailing=struct.unpack_from('<IIH',raw,anims+animation*10);count=trailing&255;assert count==(last-first)//14+1 and count>0
  for current in [0,count-1,count,255]:
   for variation,(scale76,scale77,height)in enumerate([(0,255,-1),(100,255,7194),(255,255,0)]):
    selected=min(current,count-1);fp=frames+first+selected*14;cp=struct.unpack_from('<I',raw,fp)[0];component_count=raw[fp+11];assert component_count>0
    c=comps+cp+(component_count-1)*10;sprite,x,y=struct.unpack_from('<Ihh',raw,c);w,h=raw[tab+sprite+4:tab+sprite+6];factor=max(150,min(256,scale76*scale77//100))
    ground=[-17,23];origin=[ground[0],ground[1]-(height>>6)];point=[origin[0]+int((x>>1)*factor/256),origin[1]+int((y>>1)*factor/256)]
    if variation==1:point=[origin[0]+1000,origin[1]+1000]
    elif variation==2:point[0]+=max(0,(w*factor>>8)-1);point[1]+=max(0,(h*factor>>8)-1)
    assert all(-32768<=v<=32767 for v in ground+point)
    record=bytearray(124);record[6]=1;record[24]=0;record[0x5e]=8;record[0x7a]=4;record[0x19]=animation;record[0x1b]=current;record[0x76]=scale76;record[0x77]=scale77
    struct.pack_into('<i',record,0x11,3354);struct.pack_into('<i',record,0x27,3354+height);struct.pack_into('<II',record,0x1f,base+anims,0x111fec)
    state=resource_state.copy();state.update({0x1542be+i:v for i,v in enumerate(record)})
    for i,v in enumerate(struct.pack('<i',point[1])):state[0x300004+i]=v
    seen=[];report,_,_=model.run(entry=0x5a140,state_before=state,registers={'EAX':0x1542be,'EDX':ground[0],'EBX':ground[1],'ECX':point[0]},instruction_budget=5000)
    hit=model.r['EAX']&255;assert hit in [0,1]and report['draws']==0
    assert seen==[base+fp],('source frame clamp',animation,current,seen,base+fp)
    assert bytes(model.load(0x1542be+i,1)for i in range(124))==record,'source mutated creature'
    lines.append(' '.join(map(str,[record.hex(),*ground,*point,hit,animation,selected,count,height,factor,component_count])))
    rows.append({'animation':animation,'record_frame':current,'source_frame':selected,'source_count':count,'height':height,'factor':factor,'hit':hit,'component_count':component_count,'instructions':report['steps']})
 (work/'cases.tsv').write_text('\n'.join(lines)+'\n')
 paths=[Path(__file__),resource,*files,P/'private/ghidra/actor-activation/instructions-0001f260.txt',P/'private/ghidra/initial-creature/instructions-0005a110.txt',P/'rust/crates/genewars-desktop/src/bin/support/mule_pick.rs']
 result={'status':'source_prepared','cases':len(rows),'rows':rows,'pins':{p.relative_to(P).as_posix():sha(p)for p in paths},'private_run':work.relative_to(P).as_posix(),'fixture':(work/'cases.tsv').relative_to(P).as_posix(),'scope':'Full literal5A140/1F260/1E840/95300/952A0 with loader-normalized real MULE resource; exact source frame address, record unchanged and zero RNG. Rust host comparison pending. Visibility/mixed ordering/control/native acceptance/campaign progress excluded.'}
 (work/'source-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k not in ['rows','pins']}))
if __name__=='__main__':main()
