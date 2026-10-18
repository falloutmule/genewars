from pathlib import Path
import json,sys,hashlib
P=Path(__file__).resolve().parents[2];w=P/'private/review/krell-family8-zone-travel1457-070/continuation';before=P/'private/review/krell-mule-callbacks-070/frame-1457-slot-2-family8-before';after=Path(sys.argv[1]);outputs={}
back=bytearray((before/'backing.bin').read_bytes());back[:0x355e]=(w/'prefix.bin').read_bytes();back[0x355e:0x3b55e]=(w/'map.bin').read_bytes();plants=(w/'plants.bin').read_bytes()
for i in range(512):
 a=plants[i*49:(i+1)*49];back[0x43d4e+i*41:0x43d4e+(i+1)*41]=a[:31]+a[39:]
back[20:20+7220]=(before/'backing.bin').read_bytes()[20:20+7220] # players are authoritative separate buffer; backing mirror remains serialized
outputs['backing']=bytes(back);outputs['players']=(w/'prefix.bin').read_bytes()[20:20+7220];outputs['rng']=(w/'rng.bin').read_bytes()
for name in ['actors','buildings','creatures','effects','family8']:outputs[name]=(w/f'{name}.bin').read_bytes()
strides={'actors':200,'buildings':67,'creatures':124,'effects':81,'family8':102};diffs={};count=0
for name,source in outputs.items():
 a=bytearray(source);b=bytearray((after/f'{name}.bin').read_bytes())
 if name in strides:
  for i in range(0,len(a),strides[name]):a[i+31:i+39]=bytes(8);b[i+31:i+39]=bytes(8)
 differences=[{'offset':i,'source':x,'production':y}for i,(x,y)in enumerate(zip(a,b))if x!=y]
 if differences:diffs[name]={'count':len(differences),'first':differences[:20]}
 count+=len(a)
r={'status':'passed' if not diffs else 'error','compared_bytes':count,'differences':diffs,'source':str(w),'production':str(after),'ports':['audio engine requests not executed','pointerfields31..38 normalized','render queue request port'],'source_script_sha256':hashlib.sha256(Path(__file__).with_name('krell-family8-1457-070-source.py').read_bytes()).hexdigest()};Path(__file__).with_suffix('.json').write_text(json.dumps(r,indent=2));print(json.dumps(r,indent=2))
