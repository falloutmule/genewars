"""Test abstract integer semantics observed in static disassembly; no original execution."""
from pathlib import Path
import hashlib, json, math, struct
P=Path(__file__).resolve().parents[2]
image=(P/'private/ghidra/validated/memory-.object1.bin').read_bytes()
HASH='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
def i16(v):
    v &= 65535
    return v if v<32768 else v-65536
def observed_delta(a,b): return i16(((b&65535)<<1)-((a&65535)<<1))>>1
def portable_delta(a,b): return ((b-a+16384)%32768)-16384
def observed_squared(a,b):
    x,y=(observed_delta(s,t) for s,t in zip(a,b))
    return (x*x+y*y)&0xffffffff
def portable_squared(a,b): return sum(portable_delta(s,t)**2 for s,t in zip(a,b))
table=list(struct.unpack_from('<32H',image,0x97952-0x10000))
assert table==[min(65535,math.isqrt(1<<(i+1))) for i in range(32)]
def observed_isqrt(n):
    if n==0:return 0
    estimate=table[n.bit_length()-1]
    while n//estimate<estimate:
        estimate=(estimate+n//estimate)>>1
    return estimate
delta_checks=0
for delta in range(-65535,65536):
    assert observed_delta(0,delta)==portable_delta(0,delta)
    delta_checks+=1
pairs=[((0,0),(0,0)),((0,0),(3,4)),((0,0),(32767,0)),((0,0),(16383,0)),((0,0),(16384,0)),((0,0),(16385,0)),((0,0),(32767,32767)),((0,0),(16384,16384)),((-32768,-32768),(0,0)),((-32768,32767),(32767,-32768)),((100,200),(110,220)),((32760,10),(7,10))]
vectors=[]
for a,b in pairs:
    n=observed_squared(a,b)
    assert n==portable_squared(a,b)
    vectors.append(dict(source=list(a),target=list(b),squared=n,distance=observed_isqrt(n)))
sqrt_inputs=set(range(100000))
for bit in range(32):
    center=1<<bit
    sqrt_inputs.update(n for n in range(max(0,center-3),min(0xffffffff,center+3)+1))
for r in [255,256,1023,1024,16383,16384,23170,32767,32768,46340,65534,65535]:
    sqrt_inputs.update(n for n in range(r*r-2,min(0xffffffff,r*r+2)+1))
seed=0xc0decafe
for _ in range(100000):
    seed=(seed*1664525+1013904223)&0xffffffff
    sqrt_inputs.add(seed)
sqrt_inputs.update([0xffffffff,0xfffffffe,0x80000000,0x7fffffff,536870912])
for n in sorted(sqrt_inputs):assert observed_isqrt(n)==math.isqrt(n),n
samples=[0,1,2,3,4,8,9,15,16,17,65535,65536,536870912,0x7fffffff,0x80000000,0xffffffff]
funcs=[]
metadata=json.loads((P/'private/ghidra/validated/program-metadata.json').read_text())
for addr,size,rule in [(0x73690,95,'wrapped_squared_distance'),(0x737f0,102,'wrapped_floor_distance'),(0x97910,66,'floor_sqrt_u32'),(0x432c0,124,'record_radius_predicate'),(0x8d580,106,'set_grid_flag_0x10'),(0x8d5f0,105,'clear_grid_flag_0x10')]:
    funcs.append(dict(address=hex(addr),object=1,object_offset=hex(addr-0x10000),size=size,mapped_bytes_sha256=hashlib.sha256(image[addr-0x10000:addr-0x10000+size]).hexdigest(),independent_name=rule,confidence='Observed',variant_scope='CD provisional analysis candidate only',callers=[f['entry'] for f in metadata['functions'] if f'{addr:08x}' in f['calls']]))
output=dict(schema_version=1,source_sha256=HASH,canonical=False,original_execution_performed=False,method='Static instruction review plus independent integer semantics model; vectors are model-derived, not original execution traces.',functions=funcs,distance_vectors=vectors,sqrt_vectors=[dict(input=n,expected=observed_isqrt(n)) for n in samples],checks=dict(all_passed=True,delta_semantics_checks=delta_checks,sqrt_model_vs_mathematical_checks=len(sqrt_inputs),sqrt_table_matches_derived_upper_bounds=True),limits=['Feature identities and world units unknown; these functions establish arithmetic and record layouts only.','Compiler default produces unreliable inferred C signatures; EAX/EDX register flow and instruction width take precedence.'])
(P/'research/ghidra/primitive-evidence.json').write_text(json.dumps(output,indent=2)+'\n')
print(json.dumps(output['checks']))
