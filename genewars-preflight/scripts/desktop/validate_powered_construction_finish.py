"""Focused534D0 completion-bit proof using the shared cancellation oracle."""
from pathlib import Path
import hashlib,json
P=Path(__file__).resolve().parents[2]
def main():
 source=P/'scripts/ghidra/validate_construction_finished.py';text=source.read_text()
 changes={'range(192)':'range(12)','[case//48]':'[case//3]',"struct.pack_into('<h',building,49,1)":"struct.pack_into('<h',building,49,[1,5,257][case%3])","'cases':192":"'cases':12",'construction-finished-admitted-evidence.json':'construction-finished-powered-evidence.json'}
 for old,new in changes.items():
  assert text.count(old)==1,(old,text.count(old));text=text.replace(old,new)
 exec(compile(text,str(source),'exec'),{'__file__':str(source),'__name__':'__main__'})
 path=P/'research/ghidra/construction-finished-powered-evidence.json';r=json.loads(path.read_text());r['fixture_driver_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest();r['shared_oracle_sha256']=hashlib.sha256(source.read_bytes()).hexdigest();r['building_flag_words']=[1,5,257];r['scope']='Actual534D0/51E80 completed selectors1/3/6/11, flags1/5/257: source53D58 masks lowbit1, ignores powerbit4 and highbyte. Whole200B actor+229376B map; existing UI/text ports retained.';path.write_text(json.dumps(r,indent=2)+'\n')
if __name__=='__main__':main()
