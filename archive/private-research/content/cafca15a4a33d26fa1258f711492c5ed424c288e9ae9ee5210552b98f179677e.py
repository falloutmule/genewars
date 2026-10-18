p=r"rust/crates/genewars-runtime/src/actor_world/owned_mule_updates.rs"
b=open(p,'rb').read()
old=b"CreatureError::UnsupportedNeighborhood"
new=b"{eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood}"
print("count",b.count(old))
open(p,'wb').write(b.replace(old,new))
