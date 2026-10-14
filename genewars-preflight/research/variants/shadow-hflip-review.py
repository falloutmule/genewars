"""Source-control stream versus native component-hflip shadow; full pitched buffers."""
from pathlib import Path
import sys,struct,hashlib,json,subprocess,shutil,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
W=P/'private/review/shadow-hflip'/uuid.uuid4().hex;W.mkdir(parents=True)
source=P/'rust/crates/genewars-render/src/entity_shadow.rs'
anb=(P/'private/integration/game-runtime/effects/EFF.ANB').read_bytes()
assert hashlib.sha256(anb).hexdigest()=='f45f1201e50ce67199f1e1e8448d250f221cad71f1328ee44b7bacf9ee62c253'
gla=(P/'private/integration/game-runtime/terrain/MAIN.GLA').read_bytes();assert len(gla)==65536
tab,components,frames,animations=struct.unpack_from('<4I',anb,2)
first,last,_=struct.unpack_from('<IIH',anb,animations+33*10);assert (first,last)==(2646,2688)
fixtures=[]
for frame in range(first,last+1,14):
    co=struct.unpack_from('<I',anb,frames+frame)[0];assert anb[frames+frame+11]==1
    ref,ox,oy,flags,_=struct.unpack_from('<IhhBB',anb,components+co)
    ptr,sw,sh=struct.unpack_from('<IBB',anb,tab+ref)
    fixtures.append((anb,18+ptr,sw,sh,ox>>1,oy>>1,flags))
# Asymmetric odd-height mask, opaque literal0 and padding/clipping discriminate orientation.
fixtures.append((bytes([3,1,2,3,0,1,0,255,1,9,0,3,4,5,6,0,255,2,0,7,0,3,8,8,8,0]),0,3,5,-3,-3,1))
data=bytearray();expected=bytearray();cases=0
for raw,start,sw,sh,ox,oy,flags in fixtures:
    mask=bytearray(sw*sh);rows=[];p=start
    for row in range(sh):
        runs=[];sx=0
        while True:
            control=struct.unpack_from('<b',raw,p)[0];p+=1
            if not control:break
            if control>0:
                runs.append((sx,control));mask[row*sw+sx:row*sw+sx+control]=bytes([1])*control;p+=control
            sx+=abs(control)
            assert sx<=sw
        rows.append(runs)
    for x,y in [(-sw,-sh),(-3,-2),(0,0),(10,12),(63,47),(64,48),(32767,32767)]:
        for flip in [0,1]:
            for color in [0,97,255]:
                data+=struct.pack('<BBhhhhBB',sw,sh,x,y,ox,oy,color,flip)+mask
                frame=bytearray((i*37+11)&255 for i in range(67*48))
                px=x+ox;py=y+(abs(oy)//2)*(-1 if oy<0 else 1)
                #92FF4..9301A begins at clipped right edge;93149..64 decrements
                #destination for each positive-run source sample. Odd rows only.
                for row in range(1,sh//2*2,2):
                    dy=py+row//2
                    for sx,length in rows[row]:
                        for k in range(length):
                            dx=px+(sw-1-sx-k if flip else sx+k)
                            if 0<=dx<64 and 0<=dy<48:
                                at=dy*67+dx;frame[at]=gla[color*256+frame[at]]
                expected+=frame;cases+=1
(W/'input.bin').write_bytes(data);(W/'gla.bin').write_bytes(gla);(W/'expected.bin').write_bytes(expected)
shutil.copy2(source,W/'entity_shadow.rs');shutil.copy2(Path(__file__).with_suffix('.rs'),W/'probe.rs')
deps=P/'private/build/rust-target/debug/deps';cmd=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(W/'probe.rs'),'-L',str(deps),'-o',str(W/'probe.exe')]
for name in ['genewars_render','genewars_assets']:
    lib=max(deps.glob('lib'+name+'-*.rlib'),key=lambda f:f.stat().st_mtime);cmd+=['--extern',name+'='+str(lib)]
for command in [cmd,[str(W/'probe.exe'),str(W)]]:
    r=subprocess.run(command,env=environment(),capture_output=True,text=True,timeout=60);assert r.returncode==0,(r.stdout,r.stderr)
actual=(W/'actual.bin').read_bytes();assert actual==expected,'full framebuffer mismatch'
pins=[source,Path(__file__),Path(__file__).with_suffix('.rs'),P/'private/ghidra/entity-plant/instructions-00096ac0.txt',P/'private/ghidra/entity-shadow-panel/instructions-00092f80.txt']
receipt={'status':'passed','cases':cases,'bytes_compared':len(actual),'private_run':W.relative_to(P).as_posix(),'source_pins':{f.relative_to(P).as_posix():hashlib.sha256(f.read_bytes()).hexdigest() for f in pins},'scope':'wrapperflags0 componentflags0/1 extra0; four actualEFF33 sprites plus asymmetric odd-height/opaque-zero fixture; normal and hflip source-control traversal; full pitched buffers preserve padding','original_execution':False}
(W/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
