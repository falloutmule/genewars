p=r"rust/crates/genewars-runtime/src/actor_world/owned_mule_updates.rs"
b=open(p,'rb').read()
old=b"if let Some(enemy)=world.owned_mule_single_hostile(&r).map_err(|_|{eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood})? {"
new=b"if std::env::var(\"GW_F23_PROBE\").is_ok(){let mut s=String::new();for i in 0..124{s.push_str(&format!(\"{:02x}\",r[i]));if i%16==15{s.push_str(\" / \");}}eprintln!(\"GW_F23_PREFIX\\n{}\",s);}if let Some(enemy)=world.owned_mule_single_hostile(&r).map_err(|_|{eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood})? {"
assert b.count(old)==1
open(p,'wb').write(b.replace(old,new))
print("ok")
