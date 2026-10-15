"""Literal6A540 scheduler only; virtual68/64 and7AC00 are explicit ports."""
from pathlib import Path
import importlib.util,json,hashlib,sys
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('base',Path(__file__).with_name('actor-environment-terrain-target.py'));base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base)
LIST=P/'private/ghidra/mission-array-refs/instructions-0006a540.txt'
class Scheduler(base.Model):
 def run_case(self,c):
  self.mem={};self.r={r:0 for r in ['EAX','EBX','ECX','EDX','ESI','EDI','EBP','ESP']};self.r['ESP']=0x300000;self.pc=0x6a540;self.z=self.s=self.cf=False
  self.store(0x300000,4,0xdeadbeef);self.store(0x11c2b0,2,c['count']);self.store(0x11878c,1,c.get('players',4))
  for i in range(4):self.store(0x118d74+i*0x70d+0x616,4,100+i)
  for r in c['records']:self.record(r)
  calls=[];pred=[];ui=[]
  for steps in range(20000):
   if self.pc==0xdeadbeef:break
   op,a=self.code[self.pc];nxt=self.next.get(self.pc,0xdeadbeef)
   if op=='MOV':self.put(a[0],self.get(a[1]))
   elif op in ('MOVSX','MOVZX'):self.put(a[0],base.signed(self.get(a[1]),self.width(a[1]))if op=='MOVSX'else self.get(a[1]))
   elif op=='PUSH':self.push(self.get(a[0]))
   elif op=='POP':self.put(a[0],self.pop())
   elif op in ('XOR','ADD','AND','IMUL'):
    x=self.get(a[0]);y=self.get(a[1]);v=x^y if op=='XOR'else x+y if op=='ADD'else x&y if op=='AND'else y*self.get(a[2]);self.put(a[0],v);self.flags(v,self.width(a[0]))
   elif op in ('INC','DEC'):v=self.get(a[0])+(1 if op=='INC'else -1);self.put(a[0],v);self.flags(v,self.width(a[0]))
   elif op in ('CMP','TEST'):
    x=self.get(a[0]);y=self.get(a[1]);self.cf=x<y;self.flags(x-y if op=='CMP'else x&y,self.width(a[0]))
   elif op in ('SETNZ','SETZ'):self.put(a[0],int(not self.z if op=='SETNZ'else self.z))
   elif op in ('JMP','JL','JG','JLE','JNC','JZ','JNZ'):
    if {'JMP':True,'JL':self.s,'JG':not self.s and not self.z,'JLE':self.s or self.z,'JNC':not self.cf,'JZ':self.z,'JNZ':not self.z}[op]:nxt=self.get(a[0])
   elif op=='CALL':
    if self.pc in (0x6a5a9,0x6a5e0,0x6a624):
     ptr=self.r['EAX'];slot=(ptr-0x1542be)//124;assert ptr==0x1542be+slot*124 and 0<=slot<300
     rule=next((r for r in c['records'] if r['slot']==slot),{})
     if self.pc==0x6a5a9:pred.append(slot);self.put('AL',rule.get('predicate',0))
     else:
      calls.append(slot)
      if rule.get('retire'):self.store(ptr+6,1,0);self.store(0x11c2b0,2,self.load(0x11c2b0,2)-1)
      for born in rule.get('births',[]):self.record(born);self.store(0x11c2b0,2,self.load(0x11c2b0,2)+1)
      self.put('AL',rule.get('return',0))
    else:assert self.get(a[0])==0x7ac00;ui.append(self.r['EDX'])
   elif op=='RET':nxt=self.pop()
   else:raise AssertionError((hex(self.pc),op))
   self.pc=nxt
  else:raise AssertionError('instruction budget')
  return {'primary_predicates':pred,'callbacks':calls,'player616':[self.load(0x118d74+i*0x70d+0x616,4)for i in range(4)],'live_count_after':self.load(0x11c2b0,2),'ui_states':ui,'instructions':steps,'newborn_flags':{str(i):self.load(0x1542be+i*124+0x15,1)for i in range(300)if self.load(0x1542be+i*124+6,1)}}
 def record(self,r):
  ptr=0x1542be+r['slot']*124;self.store(ptr+6,1,1);self.store(ptr+0x18,1,r.get('owner',4));self.store(ptr+0x15,1,r.get('newborn',0));self.store(ptr+0x23,4,0x400000)
def cases():
 return [
 {'name':'four_sparse_neutrals','count':4,'records':[{'slot':i}for i in [0,3,9,17]],'expected':[0,3,9,17]},
 {'name':'retirement_keeps_initial_count','count':4,'records':[{'slot':0,'retire':True}]+[{'slot':i}for i in [3,9,17]],'expected':[0,3,9,17]},
 {'name':'owned_counters_and_predicate','count':4,'players':2,'records':[{'slot':0,'owner':0},{'slot':2,'owner':1},{'slot':4,'owner':1,'predicate':1},{'slot':5}],'expected':[0,2,4,5]},
 {'name':'preexisting_newborn_skipped','count':3,'records':[{'slot':0},{'slot':1,'newborn':1},{'slot':4}],'expected':[0,4]},
 {'name':'newborn_second_pass','count':2,'records':[{'slot':0,'return':1,'births':[{'slot':1,'newborn':1}]},{'slot':4}],'expected':[0,4,1]},
 {'name':'empty','count':0,'records':[],'expected':[]}]
def main():
 results=[]
 for c in cases():
  actual=Scheduler([LIST],b'').run_case(c);assert actual['callbacks']==c['expected'],(c,actual)
  assert actual['ui_states']==list(range(0x184cbc,0x184cfc,8))
  if c['name']=='owned_counters_and_predicate':assert actual['player616']==[1,1,102,103]
  else:assert actual['player616']==[0,0,0,0]
  if c['name']=='retirement_keeps_initial_count':assert actual['live_count_after']==3
  if c['name']=='newborn_second_pass':assert actual['newborn_flags']['1']==0
  if c['name']=='preexisting_newborn_skipped':assert actual['newborn_flags']['1']==1
  results.append({'case':c,'status':'passed','source':actual,'bounded_production_expected':'blocked-rollback'if 'newborn'in c['name']or c['name']=='owned_counters_and_predicate'else'passed'})
 out={'status':'passed','scope':'Literal scheduler with controlled virtual callbacks; not creatureAI proof','production_comparison':'not run','cases':results,'source_sha256':hashlib.sha256(LIST.read_bytes()).hexdigest(),'harness_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'ports':['virtual68 predicate return','virtual64 controlled record/count mutations and AL return; preserves source callee-saved registers','7AC00 ordered request only'],'original_execution':False}
 Path(__file__).with_suffix('.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'status':'passed','cases':len(results)}))
if __name__=='__main__':
 try:main()
 except Exception as e:
  Path(__file__).with_suffix('.json').write_text(json.dumps({'status':'error','error':repr(e)})+'\n');raise
