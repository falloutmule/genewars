from pathlib import Path
import importlib.util,sys,shutil,struct,json,contextlib,io
P=Path(__file__).resolve().parents[2];W=P/'private/review/mule-corpse-motion';W.mkdir(parents=True,exist_ok=True)
s=importlib.util.spec_from_file_location('mule',P/'research/variants/creature-hungry-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
START=P/'private/review/family8-trajectory/1aca75c94f0c4dc5a1322c590db20fdb/frame8990/boundary'
a=(START/'creatures.bin').read_bytes()[:124];inputs=bytearray();expected=bytearray();timeline=[];steps=0;pins={}
for frame in range(8990,9022):
 w=W/f'frame{frame}';w.mkdir(exist_ok=True)
 for name in ['backing.bin','creatures.bin','actors.bin','effects.bin','family8.bin','players.bin','rng.bin']:shutil.copyfile(START/name,w/name)
 pool=bytearray((w/'creatures.bin').read_bytes());pool[:124]=a;(w/'creatures.bin').write_bytes(pool)
 sys.argv=['source',str(frame),'--snapshot='+str(w.relative_to(P))]
 with contextlib.redirect_stdout(io.StringIO()):m.main()
 after=bytearray((w/f'source-after{frame}.bin').read_bytes());after[31:39]=a[31:39]
 inputs+=struct.pack('<I',frame)+a;expected+=after;report=json.loads((w/f'source-trace{frame}.json').read_text());assert report['report']['draws']==0 and report['report']['map_changed_bytes']==0
 steps+=report['report']['steps'];pins.update({r['path']:r['sha256']for r in report['listing_pins']});timeline.append(dict(frame=frame,state=after[94],speed=struct.unpack_from('<h',after,61)[0],vel=struct.unpack_from('<3h',after,45)))
 a=bytes(after)
(W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected);(W/'map.bin').write_bytes((START/'backing.bin').read_bytes()[0x355e:0x3b55e])
(W/'source.json').write_text(json.dumps(dict(callbacks=32,literal_instructions=steps,listing_pins=pins,timeline=timeline,input_pins={x.relative_to(P).as_posix():__import__('hashlib').sha256(x.read_bytes()).hexdigest() for x in list(START.glob('*.bin')) if not x.name.startswith('source-')},model_pins={x.relative_to(P).as_posix():__import__('hashlib').sha256(x.read_bytes()).hexdigest() for x in [P/'research/variants/creature-hungry-source.py',P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',Path(__file__)]}),indent=2)+'\n')
print(json.dumps(dict(status='literal-moving-corpse-fixtures',callbacks=32,literal_instructions=steps)))
