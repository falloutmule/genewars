import subprocess,sys
head=subprocess.run(['git','show','HEAD:genewars-preflight/rust/crates/genewars-runtime/src/actor_world/owned_mule_updates.rs'],capture_output=True).stdout
cur=open(sys.argv[1],'rb').read()
# find the changed region: locate 'GW_F23_PROBE' in cur, and matching old text in head
i=cur.find(b'GW_F23_PROBE')
j=head.find(b'8=>{},_=>return Err(CreatureError::UnsupportedNeighborhood.into())}}raw=reference(e,3);}')
k=cur.find(b'raw=reference(e,3);}',i)
print("head old ends at",j,"cur new ends at",k)
so=head[j-20:j+80]; sc=cur[i-80:i+120]
print("HEAD:",so)
print("CUR :",sc)
# brace delta in the replaced span
hs=head[j:head.find(b'raw=reference',j)]
cs=cur[cur.find(b'_=>{',i-200):k]
print("head span opens/closes",hs.count(b'{'),hs.count(b'}'))
print("cur  span opens/closes",cs.count(b'{'),cs.count(b'}'))
print("CURSPAN:",cs)
