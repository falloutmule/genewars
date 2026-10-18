"""Independent bounded validation of CD LE pages and internal fixups against Ghidra."""
from pathlib import Path
import collections, hashlib, json, struct
P=Path(__file__).resolve().parents[2]
B=(P/'private/ghidra/GW-CD-candidate.exe').read_bytes()
H=struct.unpack_from('<I',B,0x3c)[0]
def u32(o): return struct.unpack_from('<I',B,o)[0]
assert B[H:H+2]==b'LE'
assert hashlib.sha256(B).hexdigest()=='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
page_size=u32(H+0x28)
assert page_size==4096
page_count=u32(H+0x14)
objects=[]
for i in range(u32(H+0x44)):
    size,base,flags,index,count,reserved=struct.unpack_from('<6I',B,H+u32(H+0x40)+24*i)
    objects.append(dict(number=i+1,size=size,base=base,flags=flags,page_index=index,page_count=count))
page_table=H+u32(H+0x48)
data_base=u32(H+0x80)
images={o['number']:bytearray(o['size']) for o in objects}
owners={}
page_evidence=[]
for o in objects:
    for local_page in range(o['page_count']):
        page=o['page_index']+local_page-1
        owners[page]=(o,local_page)
        entry=B[page_table+page*4:page_table+(page+1)*4]
        physical=int.from_bytes(entry[:3],'big')
        assert entry[3]==0 and 1<=physical<=page_count
        amount=min(page_size,o['size']-local_page*page_size)
        disk=data_base+(physical-1)*page_size
        amount=min(amount,len(B)-disk)
        assert amount>0
        images[o['number']][local_page*page_size:local_page*page_size+amount]=B[disk:disk+amount]
        page_evidence.append(dict(object=o['number'],virtual_page=page+1,physical_page=physical,file_offset=disk,bytes=amount))
assert len(owners)==page_count
fixup_pages=struct.unpack_from('<'+str(page_count+1)+'I',B,H+u32(H+0x68))
assert all(a<=b for a,b in zip(fixup_pages,fixup_pages[1:]))
record_base=H+u32(H+0x6c)
fixups=[]
types=collections.Counter()
flags_seen=collections.Counter()
for page in range(page_count):
    o,local_page=owners[page]
    pos=record_base+fixup_pages[page]
    end=record_base+fixup_pages[page+1]
    assert end<=len(B)
    while pos<end:
        record_offset=pos
        source,flags=B[pos:pos+2];pos+=2
        types[source&15]+=1;flags_seen[flags]+=1
        assert flags&3==0, ('noninternal fixup',hex(pos),flags)
        assert not flags&0x0c, ('unsupported chained/additive fixup',hex(pos),flags)
        if source&0x20:
            count=B[pos];pos+=1;offsets=None
        else:
            offsets=[struct.unpack_from('<h',B,pos)[0]];pos+=2
        if flags&0x40: target_object=struct.unpack_from('<H',B,pos)[0];pos+=2
        else: target_object=B[pos];pos+=1
        assert 1<=target_object<=len(objects)
        kind=source&15
        if kind==2: target_offset=0
        elif flags&0x10: target_offset=u32(pos);pos+=4
        else: target_offset=struct.unpack_from('<H',B,pos)[0];pos+=2
        if offsets is None:
            offsets=list(struct.unpack_from('<'+str(count)+'h',B,pos));pos+=count*2
        assert pos<=end
        for source_offset in offsets:
            offset=local_page*page_size+source_offset
            address=o['base']+offset
            target=objects[target_object-1]['base']+target_offset
            assert 0<=target_offset<=objects[target_object-1]['size'], (hex(record_offset),target_object,hex(target_offset),hex(objects[target_object-1]['size']))
            if kind==7: value=target; width=4
            elif kind==8: value=(target-address-4)&0xffffffff;width=4
            elif kind==2: value=target_object-1;width=2
            elif kind==5: value=target&65535;width=2
            else: raise AssertionError(('unsupported source',kind))
            assert 0<=offset and offset+width<=o['size']
            images[o['number']][offset:offset+width]=value.to_bytes(width,'little')
            fixups.append(dict(source_object=o['number'],address=address,target_object=target_object,target=target,type=kind,value=value,width=width,record_offset=record_offset))
    assert pos==end
metadata=json.loads((P/'private/ghidra/validated/program-metadata.json').read_text())
assert metadata['executable_format']=='Linear Executable (LE-Style DOS)'
assert metadata['language']=='x86:LE:32:default'
assert metadata['entry_points']==['000a6e40']
assert u32(H+0x18)==1 and objects[0]['base']+u32(H+0x1c)==0xa6e40
block_checks=[]
for o in objects:
    actual=(P/f"private/ghidra/validated/memory-.object{o['number']}.bin").read_bytes()
    assert bytes(images[o['number']])==actual, ('mapped image mismatch',o['number'],next((i for i,(a,b) in enumerate(zip(images[o['number']],actual)) if a!=b),None))
    block_checks.append(dict(object=o['number'],size=o['size'],all_bytes_equal=True,mapped_sha256=hashlib.sha256(actual).hexdigest()))
    block=next(b for b in metadata['blocks'] if b['name']==f".object{o['number']}")
    assert int(block['start'],16)==o['base'] and block['size']==o['size']
    assert block['read']==bool(o['flags']&1) and block['write']==bool(o['flags']&2) and block['execute']==bool(o['flags']&4)
relocations={int(r['address'],16):r for r in metadata['relocation_samples']}
for f in fixups:
    assert f['address'] in relocations
    r=relocations[f['address']]
    assert r['status']=='APPLIED' and r['type']==f['type']
    assert (r['current_int']&((1<<(f['width']*8))-1))==f['value']
cross=[f for f in fixups if f['source_object']!=f['target_object']]
summary=dict(schema_version=1,candidate_sha256=metadata['sha256'],canonical=False,format=metadata['executable_format'],entry_point='0xA6E40',objects=objects,pages_verified=len(page_evidence),page_size=page_size,fixup_record_types=dict(types),target_flags=dict(flags_seen),fixup_instances=len(fixups),unique_fixup_addresses=len(relocations),cross_object_fixup_instances=len(cross),cross_object_samples=cross[:12],one_past_end_targets=sum(f['target']==objects[f['target_object']-1]['base']+objects[f['target_object']-1]['size'] for f in fixups),block_checks=block_checks,permissions_match_le_flags=True,permission_correction='Unmodified loader gave every object read-only/non-executable. RepairObjectPermissions.java corrected analysis-project permissions from verified LE flags, leaving all input/mapped bytes unchanged.',all_checks_passed=True,limitations=['Static validation only; original executable never run.','Object2 lacks 32-bit flag; its 16-bit thunk code is not covered by this 32-bit function tranche.','Compiler selection borlandcpp is loader default, not compiler identification; register calling conventions require direct instruction inspection.','Ghidra .image overlay contains original file bytes; exclude it from code-address coordinates.'])
(P/'research/ghidra/loader-validation.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
