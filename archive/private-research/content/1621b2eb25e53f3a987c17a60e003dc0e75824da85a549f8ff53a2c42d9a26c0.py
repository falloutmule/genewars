p=r"rust/crates/genewars-runtime/src/actor_world/owned_mule_updates.rs"
b=open(p,'rb').read()
old=b"if r[24]!=0||!(word(r,0x6c)==0&&r[0x5e]==8&&r[0x60]==8||attack_query)||r[0x61]!=0||r[0x78]!=7{return Ok(None);}"
new=b"if r[24]!=0||!(word(r,0x6c)==0&&r[0x5e]==8&&r[0x60]==8||attack_query)||r[0x61]!=0||r[0x78]!=7||std::env::var(\"GW_F23_PROBE\").is_ok()&&r[0x57]==0&&r[0x52]==0{return Ok(None);}"
assert b.count(old)==1
open(p,'wb').write(b.replace(old,new))
print("ok")
