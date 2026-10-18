import sys
b=open(sys.argv[1],'rb').read()
depth_stack=[];instr=False;esc=False;line=1
i=0;n=len(b)
while i<n:
    c=b[i:i+1]
    if c==b'\n':line+=1
    if instr:
        if esc:esc=False
        elif c==b'\\':esc=True
        elif c==b'"':instr=False
    else:
        if c==b'"':instr=True
        elif c==b'{':depth_stack.append(line)
        elif c==b'}':
            if depth_stack:depth_stack.pop()
    i+=1
print("unclosed opens at lines:",depth_stack)
