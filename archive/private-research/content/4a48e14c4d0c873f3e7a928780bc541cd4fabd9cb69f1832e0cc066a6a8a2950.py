"""Validate hash-bound construction debit evidence; never execute original code."""
from pathlib import Path
import hashlib
import json
import struct

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
sha = lambda value: hashlib.sha256(value).hexdigest()
CANDIDATE = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G / 'GW-CD-candidate.exe').read_bytes()) == CANDIDATE
obj1 = (G / 'validated/memory-.object1.bin').read_bytes()
obj3 = (G / 'validated/memory-.object3.bin').read_bytes()
cost_bytes = obj3[0x4e1c:0x4e1c + 56]
costs = list(struct.unpack('<14i', cost_bytes))
assert costs == [600,750,400,400,300,600,250,1000,300,500,900,650,600,300]
u32 = lambda va: struct.unpack_from('<I', obj1, va - 0x10000)[0]
assert u32(0x579ac + 8 * 4) == 0x58271
assert u32(0x534b0 + 3 * 4) == 0x53a33
assert u32(0x507bc) == 0x5122f

contracts = {
 'construction-command/activation-range.txt': [
  '00057f72 TEST byte ptr [EAX + 0x57],0x1',
  '00057f7c AND byte ptr [EAX + 0x57],0xfe',
  '00057f8d CALL 0x000507d0',
  '00057f97 MOV AL,byte ptr [EAX + 0x55]',
  '00057fa9 JMP dword ptr CS:[EAX*0x4 + 0x579ac]'],
 'construction-setup/instructions-000507d0.txt': [
  '0005082d MOV AL,byte ptr [EAX + 0x16]',
  '0005083d JMP dword ptr CS:[EAX*0x4 + 0x507bc]',
  '00051235 MOV byte ptr [EAX + 0x55],0x9',
  '000509e4 LEA ECX,[EAX + 0x82]',
  'MOV word ptr [ECX + 0x6],0x0'],
 'construction-command/dispatch-range.txt': [
  '00058274 CALL 0x000534d0'],
 'building-cost-refs/instructions-000534d0.txt': [
  '00053a3c CALL dword ptr [EDX + 0xac]', '00053a44 JZ 0x00053982',
  '00053a4d MOV AL,byte ptr [EAX + 0x18]', '00053a55 IMUL EAX,EAX,0x70d',
  '00053a62 LEA EDX,[EAX + 0x4a1]', '00053a6b MOV BL,byte ptr [EAX + 0x83]',
  '00053a77 MOVSX EBX,BL', '00053a7a MOVSX EAX,byte ptr [EAX + 0x17]',
  '00053a7e MOV ECX,dword ptr [EBX*0x4 + 0x104e1c]',
  '00053a85 ADD EAX,EAX', '00053a8a SUB ECX,EAX',
  '00053a8f ADD EAX,ESI', '00053a93 JGE 0x00053ab3',
  '00053a9d MOV byte ptr [EAX + 0x7a],0x8', '00053aa1 CALL 0x00051e80',
  '00053ab7 SUB EDX,dword ptr [EAX + 0x2]', '00053abc JG 0x00053aca',
  '00053ac3 NEG ESI', '00053aca SUB EDX,dword ptr [EAX + 0x6]',
  '00053acd MOV dword ptr [EAX + 0x2],0x0', '00053ad9 NEG ECX',
  '00053ae0 MOV DL,byte ptr [EAX]', '00053ae8 CMP byte ptr [EDX + 0x118d76],0x0',
  '00053af7 CMP EDX,0x3e8', '00053afd JG 0x00053b06',
  '00053aff MOV dword ptr [EAX + 0x6],0x3e8',
  '00053b09 MOV word ptr [EAX + 0x6],0x4',
  '0005397c MOV word ptr [EAX + 0x6],0x3',
  '00054206 MOV AX,word ptr [EAX + 0x6]',
  '00054219 JMP dword ptr CS:[EAX*0x4 + 0x534b0]',
  '000549e2 JZ 0x00054206'],
}
listings = []
for name, terms in contracts.items():
 raw = (G / name).read_bytes()
 text = raw.decode()
 for term in terms:
  assert term in text, (name, term)
 listings.append({'private_path': 'private/ghidra/' + name, 'sha256': sha(raw)})

def i32(value):
 value &= 0xffffffff
 return value if value < 0x80000000 else value - 0x100000000

def model(selector, skill, primary, secondary, refill, ready=True):
 """Original numeric branch, after safe selector admission and prerequisite state."""
 if not 0 <= selector < 14:
  return None
 assert -128 <= skill <= 127
 assert i32(primary) == primary and i32(secondary) == secondary
 if not ready:
  return dict(status='not_ready', primary=primary, secondary=secondary, stage=3)
 cost = i32(costs[selector] - 2 * skill)
 if i32(primary + secondary) < cost:
  return dict(status='insufficient', cost=cost, primary=primary, secondary=secondary, stage=3)
 remainder = i32(cost - primary)
 if remainder <= 0:
  primary = i32(-remainder)
 else:
  secondary = i32(-i32(remainder - secondary))
  primary = 0
 if refill and i32(primary + secondary) <= 1000:
  secondary = 1000
 return dict(status='paid', cost=cost, primary=primary, secondary=secondary, stage=4)

cases = [
 (0,0,600,0,False,True), (0,0,601,0,False,True),
 (0,0,599,0,False,True), (0,0,100,500,False,True),
 (0,0,100,501,False,True), (0,0,700,0,True,True),
 (0,0,1600,0,True,True), (0,0,1601,0,True,True),
 (0,0,0,600,True,True), (6,127,0,0,False,True),
 (6,-128,506,0,False,True), (6,-128,505,0,False,True),
 (0,0,2147483647,1,False,True), (6,127,-2147483648,-2147483648,False,True),
 (0,0,600,0,False,False), (0,0,2147483647,0,True,True),
 (0,0,-1,602,False,True), (13,127,46,0,False,True),
]
vectors = [dict(selector=a, skill=b, primary=c, secondary=d, refill=e,
                ready=f, expected=model(a,b,c,d,e,f)) for a,b,c,d,e,f in cases]
assert vectors[0]['expected'] == dict(status='paid', cost=600, primary=0, secondary=0, stage=4)
assert vectors[2]['expected']['status'] == 'insufficient'
assert vectors[5]['expected']['primary'] == 100 and vectors[5]['expected']['secondary'] == 1000
assert vectors[6]['expected']['secondary'] == 1000 and vectors[7]['expected']['secondary'] == 0
assert vectors[9]['expected']['cost'] == -4 and vectors[9]['expected']['primary'] == 4
assert vectors[12]['expected']['status'] == 'insufficient'
assert model(14,0,1000,1000,False) is None and model(-1,0,1000,1000,False) is None
cost_vectors = [(selector, skill, i32(base - 2 * skill))
                for selector,base in enumerate(costs) for skill in range(-128,128)]
assert len(cost_vectors) == 3584
receipt = dict(
 candidate_sha256=CANDIDATE, status='verified_bounded_numeric_transaction',
 source='Exact CD candidate, static instructions; no original program execution',
 transaction_va='0x53a33', function_va='0x534d0',
 command_chain=['pending entity+0x57 bit0 -> 0x57f8d -> 0x507d0',
  'profile byte+0x16=0 -> initializer 0x5122f sets dispatch byte entity+0x55=9',
  'dispatch 0x57fa9 table index8 -> 0x58271 -> 0x534d0',
  'mode byte entity+0x7e=0 and stage u16 entity+0x88=3 -> 0x53a33'],
 costs_va='0x104e1c', costs=costs, cost_table_sha256=sha(cost_bytes),
 layout=dict(owner='unsigned byte entity+0x18', selector='signed byte entity+0x83; safe admitted range 0..13',
  skill='signed byte [entity+0xc0]+0x17', stage='unsigned16 entity+0x88',
  resource_base='0x119215 + owner*0x70d', primary='signed32 resource+2', secondary='signed32 resource+6',
  refill_flag='byte at 0x118d76 + unsigned(resource[0])*0x70d; any nonzero'),
 prerequisite='vtable at entity+0x23, callback slot+0xac returns AL nonzero; caller must supply readiness',
 formula='cost = wrapping_i32(base_cost - 2*signed8_skill); compare signed wrapping_i32(primary+secondary) >= cost',
 insufficient='Pools untouched by debit branch; entity+0x7a=8, calls 0x51e80(entity,1), then returns. Pure model does not emulate cancellation side effects.',
 not_ready='Calls 0x526c0(entity,0) through 0x53982; model only leaves pools and stage unchanged, excluding helper side effects.',
 paid='Spend primary first using signed wrapping cost-primary comparison; otherwise subtract remainder from secondary and clear primary. If refill flag nonzero and new signed wrapping total<=1000, set secondary=1000. Set stage=4.',
 tests=dict(listing_contracts=len(contracts), cost_skill_combinations=len(cost_vectors),
  cost_skill_vectors_sha256=sha(json.dumps(cost_vectors,separators=(',',':')).encode()), transaction_vectors=len(vectors)),
 vectors=vectors, private_listings=listings,
 limits=['Safe API rejects selectors outside0..13; original signed array access has no local guard',
  'Not an entire build command or world simulation; callback, movement, reservations, cancellation and later building creation excluded',
  'No proven physical input binding, scheduling cadence, skill gameplay range, refill-flag meaning or balance invariant',
  'Negative adjusted costs and signed overflow preserved, not clamped',
  'Two original temporary pool writes before NEG collapse to one final pure transaction result',
  'Synthetic model vectors and instruction/hash assertions are not execution of the original binary'])
(P/'research/ghidra/construction-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',newline='\n')
print(json.dumps(dict(status='passed',listing_contracts=len(contracts),cost_skill_combinations=len(cost_vectors),transaction_vectors=len(vectors))))
