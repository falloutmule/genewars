"""Literal family8 periodic-search effect audit using the existing hover loader."""
from pathlib import Path
import subprocess,sys,json,hashlib
P=Path(__file__).resolve().parents[2];W=P/'private/review/family8-idle'
def main():
 W.mkdir(parents=True,exist_ok=True);original=P/'research/variants/family8-hover-source.py';s=original.read_text()
 s=s.replace('private/integration/family8-combined011-seed5428-frame6234/before','private/review/family8-trajectory/579c9bdb947e4419926cb281b7bf0f38/frame6332/before').replace('private/review/family8-hover','private/review/family8-idle').replace('192<=i<256','0<=i<64').replace('pool[7*102:8*102]','pool[4*102:5*102]').replace('base=m.BASE+7*102','base=m.BASE+4*102').replace('range(6234,7234)','range(6332,6333)').replace("else:raise AssertionError('hover did not expire within1000callbacks')",'').replace('if a[0x56]!=1:break','')
 s=s.replace('P=Path(__file__).resolve().parents[2]','P=Path.cwd()')
 for f in ['family8-state4-source.py','initial-creature-source.py','actor-environment-terrain-target.py']:s=s.replace("Path(__file__).with_name('"+f+"')","P/'research/variants/"+f+"'")
 needle='after,nrng,n,requests,trace=model.step(a,map_data,frame,rng,extra_state=state,base=base)';assert needle in s
 prefix=s.replace('inputs=bytearray();expected=bytearray();particles=bytearray();timeline=[];steps=0',"model.code[0x6fc52]=('JMP',['0x00070790'])\n inputs=bytearray();expected=bytearray();particles=bytearray();timeline=[];steps=0")
 prefix=prefix.replace(needle,needle.replace('base=base)','base=base,entry=0x6fb60)')+"\n  expected_prefix=bytearray(a);expected_prefix[0x49]&=0x9f\n  assert after==expected_prefix and nrng==rng\n  assert bytes(model.saved_state[0x11c2be+i]for i in range(229376))==map_data\n  print('PREFIX',json.dumps(dict(instructions=n,queries=trace.count(0x6d230),search=0x305f0 in trace,callback=trace.count(0x1ea80),draws=(nrng[2]-rng[2])&0xffffffff)))")
 reports=[]
 for mask in [0,128,255]:
  for transient in [0,32]:
   for frame in [6332,6333]:
    script=prefix.replace('for frame in range(6332,6333):',f'for frame in range({frame},{frame+1}):\n  a=bytearray(a);a[0x53]={mask};a[0x49]|={transient}')
    driver=W/'prefix-driver.py';driver.write_text(script);r=subprocess.run([sys.executable,'-B',str(driver)],cwd=P,capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
    data=json.loads(next(x[7:]for x in r.stdout.decode().splitlines()if x.startswith('PREFIX ')));data.update(mask=mask,transient=transient,frame=frame);reports.append(data)
 assert any(r['callback'] for r in reports)and any(r['search']and not r['callback']for r in reports)
 driver=W/'full-driver.py';driver.write_text(s.replace(needle,needle+"\n  assert 0x305f0 in trace and 0x1ea80 in trace"));r=subprocess.run([sys.executable,'-B',str(driver)],cwd=P,capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
 full=json.loads((W/'source.json').read_text());full['effect_audit']=reports;full['effect_audit_instructions']=sum(r['instructions']for r in reports);full['pins'][Path(__file__).relative_to(P).as_posix()]=hashlib.sha256(Path(__file__).read_bytes()).hexdigest();full['pins'][original.relative_to(P).as_posix()]=hashlib.sha256(original.read_bytes()).hexdigest();full['scope']='actual corrected6332 whole callback plus12 prefix audits; only searchscratch changes'
 capture=P/'private/review/family8-trajectory/579c9bdb947e4419926cb281b7bf0f38'
 for p in [capture/'compilation.json',capture/'result.json']:full['pins'][p.relative_to(P).as_posix()]=hashlib.sha256(p.read_bytes()).hexdigest()
 (W/'source.json').write_text(json.dumps(full,indent=2)+'\n');print(json.dumps(dict(prefix_cases=len(reports),prefix_instructions=full['effect_audit_instructions'],full_instructions=full['literal_instructions'],outcomes=[dict(mask=r['mask'],search=r['search'],queries=r['queries'],callback=r['callback'])for r in reports])))
if __name__=='__main__':main()
