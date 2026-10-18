p=r"rust/crates/genewars-runtime/src/actor_world/owned_mule_updates.rs"
b=open(p,'rb').read()
old=b"if let Some(enemy)=world.owned_mule_single_hostile(&r).map_err(|_|{eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood})? {"
new=b"if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_PREFIX 5e={} 60={:02x} 61={:02x} 6c={:04x} 78={} 24={}\",r[0x5e],r[0x60],r[0x61],word(&r,0x6c)as u16,r[0x78],r[24]);}if let Some(enemy)=world.owned_mule_single_hostile(&r).map_err(|_|{eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood})? {"
assert b.count(old)==1, b.count(old)
b=b.replace(old,new)
old2=b"if hostile.is_some(){return Err({eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood}.into());}hostile=Some(other);"
assert b.count(old2)==1, b.count(old2)
new2=b"if hostile.is_some(){if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_SECOND hostile-slot {} 5e={} 60={:02x} 61={:02x} 7a={} 24={}\",word(&other,15)as u16,other[0x5e],other[0x60],other[0x61],other[0x7a],other[24]);}return Err(CreatureError::UnsupportedNeighborhood.into());}if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_FIRST hostile-slot {} 5e={} 60={:02x} 7a={} 24={}\",word(&other,15)as u16,other[0x5e],other[0x60],other[0x7a],other[24]);}hostile=Some(other);"
b=b.replace(old2,new2)
open(p,'wb').write(b)
print("ok")
