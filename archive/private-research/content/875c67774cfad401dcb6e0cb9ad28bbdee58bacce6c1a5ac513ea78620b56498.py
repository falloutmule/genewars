import sys
b=open(sys.argv[1],'rb').read()
depth=0;instr=False;esc=False;line=1
prev=0
for i,c in enumerate(b):
    c=c.to_bytes(1,'big')
    if c==b'\n':
        line+=1
    if instr:
        if esc:esc=False
        elif c==b'\\':esc=True
        elif c==b'"':instr=False
    else:
        if c==b'"':instr=True
        elif c==b'{':depth+=1
        elif c==b'}':
            depth-=1
            if depth<0:print("NEG at line",line);break
    if line>=430 and line<=437 and c==b'\n':
        print("line",line,"depth",depth)
