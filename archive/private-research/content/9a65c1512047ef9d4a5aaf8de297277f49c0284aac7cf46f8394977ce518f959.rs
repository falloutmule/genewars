use std::fs;
use std::path::Path;
mod actor_initialization { pub use genewars_runtime::actor_initialization::{ActorResourceTableKey,BlueprintRecordHandle}; }
mod identity { pub use genewars_runtime::identity::sha256_hex; }
mod actor_pool { pub use genewars_runtime::actor_pool::{ActorPool,ActorPoolError,ActorHandleBindings}; }
mod player_state { pub use genewars_runtime::player_state::{PlayerState,ActorOriginSample,PLAYER_RECORD_BYTES}; }
#[path = "C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_resources.rs"] mod actor_resources;
#[path = "C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/actor_render.rs"] mod actor_render;
use player_state::{PlayerState,ActorOriginSample};
use actor_resources::{ActorResourceCatalog,ActorResourceError};
fn main() {
  let root = std::env::args().nth(1).expect("input dir");
  let catalog = ActorResourceCatalog;
  for spec in catalog.specs() {
    let path = Path::new(&root).join(format!("key-{:02}.anb", spec.key.flat_index));
    let bytes = fs::read(path).expect("read staged ANB");
    let db = catalog.load(spec.key, bytes).expect("strict catalog load");
    let borrowed = catalog.load_borrowed(spec.key, db.bytes()).expect("borrowed strict catalog load");
    assert_eq!(borrowed.bytes(), db.bytes());
    assert!(matches!(db.animation(db.animation_count()), Err(ActorResourceError::AnimationOutOfBounds{..})));
    assert!(matches!(db.frame(0, usize::MAX), Err(ActorResourceError::FrameOutOfBounds{..})));
    assert!(matches!(db.frame(0, db.frame_count(0).unwrap()), Err(ActorResourceError::FrameOutOfBounds{..})));
    assert!(db.component(0, 0, usize::MAX).is_err());
    assert!(db.decode_sprite_indexed8(db.sprite_record_count()).is_err());
    assert!(matches!(catalog.load(spec.key, db.bytes()[..db.bytes().len()-1].to_vec()), Err(ActorResourceError::WrongLength{..})));
    let mut altered = db.bytes().to_vec(); altered[0] ^= 1;
    assert!(matches!(catalog.load(spec.key, altered), Err(ActorResourceError::WrongHash{..})));
    assert!(matches!(catalog.spec(actor_initialization::ActorResourceTableKey{style:spec.key.style,category:spec.key.category,flat_index:spec.key.flat_index ^ 1}), Err(ActorResourceError::InvalidKey(_))));
    println!("P\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", spec.key.flat_index, spec.key.style, spec.key.category, spec.descriptor_va, spec.database_slot_va, spec.animation_slot_va, spec.source_name, spec.entry_id, spec.source_sha256, spec.decoded_sha256, spec.decoded_bytes, spec.animation_count);
    println!("R\t{}\t{}\t{}\t{}", spec.key.flat_index, db.animation_count(), db.sprite_record_count(), identity::sha256_hex(db.bytes()));
    for animation in 0..db.animation_count() {
      let record = db.animation(animation).expect("animation");
      let frame_count = db.frame_count(animation).expect("frame count");
      println!("A\t{}\t{}\t{}\t{}\t{}", spec.key.flat_index, animation, record.first_frame_offset, record.last_frame_offset, frame_count);
      for frame_index in 0..frame_count {
        let frame = db.frame(animation, frame_index).expect("frame");
        println!("F\t{}\t{}\t{}\t{}\t{}\t{}\t{}", spec.key.flat_index, animation, frame_index, frame.width_raw, frame.height_raw, frame.component_count, frame.component_offset);
        for component in 0..usize::from(frame.component_count) {
          let c = db.component(animation, frame_index, component).expect("component");
          println!("C\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", spec.key.flat_index, animation, frame_index, component, c.sprite_offset, c.x_raw, c.y_raw, c.flags, c.extra);
        }
      }
    }
    for (record, desc) in db.sprite_descriptors().iter().enumerate() {
      let sprite = db.decode_sprite_indexed8(record).expect("indexed8 sprite decode");
      let opaque = sprite.opaque.iter().filter(|v| **v).count();
      println!("S\t{}\t{}\t{}\t{}\t{}\t{}\t{}", spec.key.flat_index, record, desc.width, desc.height, sprite.consumed_bytes, opaque, sprite.indices.len());
    }
  }
  let key0 = catalog.specs()[0].key;
  let db0 = catalog.load(key0, fs::read(Path::new(&root).join("key-00.anb")).unwrap()).unwrap();
  let packed = vec![0u8; 20 * 140];
  let native = [[0u8; 200]; 20];
  let mut pool = actor_pool::ActorPool::from_packed_inactive(&packed, &native, [actor_pool::ActorHandleBindings::empty(); 20]).unwrap();
  let mut player = PlayerState::fresh_campaign(0, [20000,12000], b"TEST\0").unwrap();
  let mut player_bytes = *player.bytes();
  player_bytes[0x16] = b'P'; player_bytes[0x16+0x15] = 0; player_bytes[0x16+0x16] = 0;
  player = PlayerState::from_record(&player_bytes).unwrap();
  let partition = pool.partition(0).unwrap();
  let prepared = player.prepare_fresh_actor_partition(&partition, 1, ActorOriginSample { position:[321,-456], height:10, shade:4 }, [777,-888]).unwrap();
  let expected_player = player.clone();
  pool.commit_prepared_partition(&mut player, &expected_player, &partition, &prepared).unwrap();
  let view = actor_render::extract_actor_render_record(&pool, 0, &catalog, &db0).unwrap();
  let decoded = actor_render::decode_actor_components_indexed8(&view, &db0).unwrap();
  assert_eq!(decoded.len(), view.components.len());
  assert!(decoded.iter().all(|c| c.sprite.indices.len() == c.sprite.opaque.len()));
  assert_eq!(view.actor_id, 0); assert_eq!(view.owner, 0); assert_eq!(view.resource_key, key0);
  assert_eq!(view.world_position, [777,-888]);
  assert_eq!(view.animation_index, 8); assert_eq!(view.frame_index, 0);
  let db1 = catalog.load(catalog.specs()[1].key, fs::read(Path::new(&root).join("key-01.anb")).unwrap()).unwrap();
  assert!(matches!(actor_render::extract_actor_render_record(&pool,0,&catalog,&db1), Err(actor_render::ActorRenderError::ResourceBindingMismatch{..})));
  println!("X\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", view.actor_id, view.owner, view.resource_key.flat_index, view.world_position[0], view.world_position[1], view.animation_index, view.frame_index, view.components.len());
}
