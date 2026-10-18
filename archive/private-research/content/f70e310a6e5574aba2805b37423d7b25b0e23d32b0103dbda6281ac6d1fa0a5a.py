p=r"rust/crates/genewars-runtime/src/actor_world/owned_mule_updates.rs"
b=open(p,'rb').read()
old=b"5 if self.effects.as_ref().unwrap().record(word(&raw,0)as usize)?[69]!=0x11&&self.effects.as_ref().unwrap().binding(word(&raw,0)as usize)?.is_some()=>{},8=>{},_=>{if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_PROBE cellwalk kind={} food_guard={} e24={}\",raw[2],food_guard,e[24]);}return Err({eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood}.into())}}"
new=b"5 if self.effects.as_ref().unwrap().record(word(&raw,0)as usize)?[69]!=0x11&&self.effects.as_ref().unwrap().binding(word(&raw,0)as usize)?.is_some()=>{},8=>{},1 if std::env::var(\"GW_F23_PROBE\").is_ok()=>{},_=>{if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_PROBE cellwalk kind={} food_guard={} e24={}\",raw[2],food_guard,e[24]);}return Err({eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood}.into())}}"
assert b.count(old)==1
open(p,'wb').write(b.replace(old,new))
print("ok")
