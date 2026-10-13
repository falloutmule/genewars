"""Full native shade-input domain and every bounded FADE lookup; no GUI."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
sha=lambda b:hashlib.sha256(b).hexdigest()
def record(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=sha(b))

def main():
 proof_path=ROOT/'research/ghidra/terrain-projection-evidence.json';proof=json.loads(proof_path.read_bytes());listings=[]
 for item in proof['instruction_contracts']:
  actual=record(ROOT/item['private_path']);assert actual['sha256']==item['sha256'];listings.append(actual)
 text=(ROOT/'private/ghidra/terrain-map-refs/instructions-0007b7f0.txt').read_text()
 for term in ['0007b910 MOVSX EAX,word ptr [EBX]','0007b915 SHL EAX,0x8','0007b918 MOV DL,byte ptr [EBX + 0xc]','0007b91b IMUL EDX,EAX','0007b920 SAR EDX,0x1f','0007b923 SHL EDX,0x6','0007b926 SBB EAX,EDX','0007b928 SAR EAX,0x6','0007b932 ADD EAX,0x8000']:assert term in text
 work=ROOT/'private/review/terrain-shade';work.mkdir(parents=True,exist_ok=True)
 corpus=json.loads((ROOT/'research/integration/rnc-corpus.json').read_bytes())['results'];entry=next(r for r in corpus if r['entry_id']=='root/2/disc/61' and r['status']=='passed')
 fade_path=ROOT/entry['private_output'];fade=fade_path.read_bytes();assert len(fade)==16384 and sha(fade)==proof['fade_lookup']['decoded_sha256']==entry['output_sha256']
 source=ROOT/'private/reference-run/run-004/drive-c/GAME/DATA/FADE.DAT';assert sha(source.read_bytes())==proof['fade_lookup']['input_sha256']
 exe=work/'review.exe';rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe'
 r=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(exe),str(work),str(fade_path)],capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
 path=work/'shades.bin';assert path.stat().st_size==65536*256*4
 # Model the actual x86 SHL/IMUL/SAR/SHL/SBB/SAR sequence. SHL of
 # signmask-1 by6 has carry1; SBB therefore adds63 for a negative product.
 # Product always divisible256, so truncation correction leaves /64 exact.
 minimum=2**31;maximum=-2**31;checked=0
 with path.open('rb') as native:
  for light in range(-32768,32768):
   expected=[]
   for factor in range(256):
    product=(light<<8)*factor;assert -2**31<=product<2**31
    adjusted=product+63 if product<0 else product
    value=(adjusted>>6)+0x8000
    expected.append(value);minimum=min(minimum,value);maximum=max(maximum,value);checked+=1
   assert native.read(1024)==struct.pack('<256i',*expected),light
  assert not native.read(1)
 assert (work/'mapped-fade.bin').read_bytes()==fade
 for row in proof['fade_lookup']['row_receipts']:assert sha(fade[row['shade']*256:(row['shade']+1)*256])==row['row_sha256']
 # Demonstrate zero is mapped, not discarded: final row's source0 ->148.
 assert fade[63*256]==148
 receipt=dict(status='passed-independent-native-shade-fade-review',checked_utc=datetime.now(timezone.utc).isoformat(),original_or_gui_execution=False,
  all_signed16_light_unsigned8_factor_pairs=checked,shade_min=minimum,shade_max=maximum,fade_rows=64,fade_indices=16384,invalid_shade_endpoint_pairs=384,
  invalid_lengths=[0,1,16383,16385],zero_is_not_transparent=dict(shade=63,source_index=0,output_index=148),
  evidence=record(proof_path),private_listings=listings,inputs=[record(source),record(fade_path)],harness=record(exe),
  source=record(ROOT/'rust/crates/genewars-render/src/terrain.rs'),review_sources=[record(Path(__file__)),record(Path(__file__).with_suffix('.rs'))],
  limits=['No shade-factor producer, visibility meaning or selection inferred','Shade row is explicit0..63; no clamp/automatic conversion of16.16shade','No gradients, rasterization, clipping or completeframe equivalence claimed'])
 (ROOT/'research/variants/terrain-shade-review.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:v for k,v in receipt.items() if k not in ['private_listings','inputs','harness','source','review_sources']}))

if __name__=='__main__':main()
