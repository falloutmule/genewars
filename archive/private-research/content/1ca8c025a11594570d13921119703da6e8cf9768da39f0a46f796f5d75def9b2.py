p=r"rust/crates/genewars-runtime/src/actor_world/owned_mule_updates.rs"
b=open(p,'rb').read()
pat=b"if hostile.is_some(){return Err({eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood}.into());}hostile=Some(other);"
tagA=b"if hostile.is_some(){if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_DUP branchA slot={} 5e={} 60={:02x} 24={}\",word(&other,15)as u16,other[0x5e],other[0x60],other[24]);}return Err(CreatureError::UnsupportedNeighborhood.into());}if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_SET branchA slot={}\",word(&other,15)as u16);}hostile=Some(other);"
tagB=b"if hostile.is_some(){if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_DUP branchB slot={} 5e={} 60={:02x} 24={}\",word(&other,15)as u16,other[0x5e],other[0x60],other[24]);}return Err(CreatureError::UnsupportedNeighborhood.into());}if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_SET branchB slot={} 5e={}\",word(&other,15)as u16,other[0x5e]);}hostile=Some(other);"
assert b.count(pat)==2
b=b.replace(pat,tagA,1)
assert b.count(pat)==1
b=b.replace(pat,tagB,1)
# also the kind-1 walk probe listing each entity visited
old1=b"1=>{let pool=self.creatures.as_ref().ok_or({eprintln!(\"GW_F23_SITE {}:{}\",file!(),line!());CreatureError::UnsupportedNeighborhood})?;let other=*pool.record(word(&raw,0)as usize)?;"
assert b.count(old1)==1
new1=b"1=>{let pool=self.creatures.as_ref().ok_or(CreatureError::UnsupportedNeighborhood)?;let other=*pool.record(word(&raw,0)as usize)?;if std::env::var(\"GW_F23_PROBE\").is_ok(){eprintln!(\"GW_F23_KIND1 slot={} 5e={} 60={:02x} 61={:02x} 24={} 7a={} 6a={:04x} 49={:04x}\",word(&other,15)as u16,other[0x5e],other[0x60],other[0x61],other[24],other[0x7a],word(&other,0x6a)as u16,word(&other,0x49)as u16);}"
b=b.replace(old1,new1)
open(p,'wb').write(b)
print("ok")
