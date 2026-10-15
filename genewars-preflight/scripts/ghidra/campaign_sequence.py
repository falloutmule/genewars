"""Corroborate the standard CD campaign sequence; no original execution."""
from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 m=json.loads((P/'research/game-files-manifest.json').read_text(encoding='utf-8'))['entries'];d=P/'private/reference-run/run-006/drive-c/GAME/LEVELS';rows=[]
 for n in range(1,25):
  files={}
  for name in [f'CONQ{n:03}.DAT',f'HDRCO{n}.DAT']:
   e=next(e for e in m if e.get('entry_id','').startswith('root/2/disc/') and e['path'].split('/')[-1].split(';')[0].upper()==name)
   p=d/name;h=sha(p);assert h==e['sha256'];files[name]={'entry_id':e['entry_id'],'sha256':h,'size':p.stat().st_size}
  b=(d/f'HDRCO{n}.DAT').read_bytes();name=b[32:b.index(0,32)].decode('ascii');rows.append({'progress_before':n-1,'mission_number':n,'name':name,'resources':files})
 listing=P/'private/ghidra/mission-result-refs/result-view-range.txt';assert '0007de00 CMP EDX,0x18' in listing.read_text()
 pins={n:sha(P/n) for n in ['research/game-files-manifest.json','research/campaign/mission-001-contract.json','research/variants/result-transition-review.json','private/ghidra/mission-result-refs/result-view-range.txt','private/ghidra/campaign-mission-refs/instructions-00015b90.txt']}
 out={'status':'corroborated-standard-mode0-campaign-sequence','reference_build':'GENEWARS QA BETA V5','recovered_standard_total':24,'accepted_completable_missions':0,'accepted_transitions':0,'basis':['Fresh campaign progress0; mission/header selection uses progress+1','Victory view increments progress once;7DE00 tests progress24 and routes to ending','All24 corresponding CD mission/header pairs are present and match inventory hashes'],'limits':['No campaign traversal or later mission objective acceptance','HDRCO25,CONQ030/HDRCO30,CONQ040/HDRCO40 are extra resources, not silently added to this standard sequence','Retail equivalence remains unestablished'],'missions':rows,'pins':pins}
 (P/'research/campaign/campaign-sequence.json').write_text(json.dumps(out,indent=2)+'\n',encoding='utf-8');print('24 header/world pairs hash-verified; first',rows[0]['name'],'last',rows[-1]['name'])
if __name__=='__main__':main()
