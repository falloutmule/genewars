p=r"rust/crates/genewars-runtime/src/actor_world/owned_mule_updates.rs"
b=open(p,'rb').read()
# find the neighborhood cellwalk arm line and print it first
i=b.find(b"for dy in -4..=4")
print(repr(b[i:i+1200]))
