//! Hash-bound access to the sixteen animation databases loaded by the actor
//! resource table at candidate VA 0x104074.
//!
//! The table key order is `4 * style + category`; this module validates the
//! pinned key, decoded bytes, and ANB metadata before exposing bounded frame
//! lookup. It does not select an animation codec, palette, timing rule, or
//! actor draw policy. See `research/ghidra/blueprint-init-evidence.json` and
//! `research/integration/actor-resource-catalog.tsv`.

use genewars_assets::{AnbAnimationRecord, AnbFrameRecord, AnbSections, AnbUiMetadata, Error as AssetError, SpriteTable};
use std::fmt;

use crate::{actor_initialization::ActorResourceTableKey, identity::sha256_hex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorResourceSpec {
    pub key: ActorResourceTableKey,
    pub descriptor_va: u32,
    pub database_slot_va: u32,
    pub animation_slot_va: u32,
    pub source_name: &'static str,
    pub entry_id: &'static str,
    pub source_sha256: &'static str,
    pub decoded_sha256: &'static str,
    pub decoded_bytes: usize,
    pub animation_count: usize,
}

const fn actor_spec(style: u8, category: u8, flat_index: u8, source_name: &'static str,
                    entry_id: &'static str, sha256: &'static str, decoded_bytes: usize,
                    animation_count: usize) -> ActorResourceSpec {
    ActorResourceSpec {
        key: ActorResourceTableKey { style, category, flat_index },
        descriptor_va: 0x104074 + 44 * flat_index as u32,
        database_slot_va: 0x1168F4 + 4 * flat_index as u32,
        animation_slot_va: 0x116934 + 4 * flat_index as u32,
        source_name, entry_id, source_sha256: sha256, decoded_sha256: sha256,
        decoded_bytes, animation_count,
    }
}

pub const ACTOR_RESOURCE_SPECS: [ActorResourceSpec; 16] = [
    actor_spec(0,0,0,"data/spec/harch.anb","root/2/disc/361","629a6471e4deabec794f0cffa34b513eea15013ee95927e67b02a4a7e6157b6a",35497,18),
    actor_spec(0,1,1,"data/spec/hfarm.anb","root/2/disc/362","4421b2f6570face0e323f0444e3d10b972f0f7705fe128bac54963e9e0911563",26921,16),
    actor_spec(0,2,2,"data/spec/hscie.anb","root/2/disc/363","93e5aeb581d83ebd33ad55b8068adbe427fcb9a606e88a5f8bab079ea75be7cb",38076,23),
    actor_spec(0,3,3,"data/spec/hshep.anb","root/2/disc/364","38d32a18a8b94bf2411a184506dbcbf76c344292fae8ef81971d2c02f91c4929",35797,30),
    actor_spec(1,0,4,"data/spec/carch.anb","root/2/disc/357","4a232c5debfef3fbef5a54647e6312cc325265fe996b00841b35f1511340bd73",40742,18),
    actor_spec(1,1,5,"data/spec/cfarm.anb","root/2/disc/358","8946c2d3dfd8fe059fb8c72a74a71d0eaab48d43015a28d563beec095e38023b",28983,16),
    actor_spec(1,2,6,"data/spec/cscie.anb","root/2/disc/359","63a3623d75f761aa5e09737074e5f5ab4a9cc40bf85334d69863536241874184",41317,23),
    actor_spec(1,3,7,"data/spec/cshep.anb","root/2/disc/360","6576a2ffc66893d8eacb21a44165ac11118dd7256f74237f6992210148dee847",35249,31),
    actor_spec(2,0,8,"data/spec/march.anb","root/2/disc/365","03c147fb87ad0d30a2de2f4e91ee755575b07a2d63078c9c2bb5611ded2b040f",39778,18),
    actor_spec(2,1,9,"data/spec/mfarm.anb","root/2/disc/366","73cdf337d6ff63cc4d120bdd9c3e0b0dcc59c000a4193e40f8a98c3cf72d03d9",32231,16),
    actor_spec(2,2,10,"data/spec/mscie.anb","root/2/disc/367","b4eb05c610c7d135dc9de8876608302032b0428b0afdcb052fc174b9bd2eea7a",38301,23),
    actor_spec(2,3,11,"data/spec/mshep.anb","root/2/disc/368","b51ef5a703a89dcf00c2465ddc7666fe842a9ce726363c2a36b2f2f0ac1d6277",35744,30),
    actor_spec(3,0,12,"data/spec/yarch.anb","root/2/disc/369","aef9ba878097884f64674c3273ae8de8139530c31155fbbf6d4db1db11e329c0",37700,18),
    actor_spec(3,1,13,"data/spec/yfarm.anb","root/2/disc/370","662b6358b87eff43624009881701ff9eec243d05800a79ed7478fc6736b724b0",27293,16),
    actor_spec(3,2,14,"data/spec/yscie.anb","root/2/disc/371","9f4ef097c5db8ad10c41b8992345bb2b26b6b4045a30c96e03c867f2ef968f0d",34865,23),
    actor_spec(3,3,15,"data/spec/yshep.anb","root/2/disc/372","6e6a81c03e5b8ec50706ca85c77161190183a9d44202dd155a9fc249043df559",32699,30),
];

#[derive(Debug, Default, Clone, Copy)]
pub struct ActorResourceCatalog;

impl ActorResourceCatalog {
    pub fn specs(&self) -> &'static [ActorResourceSpec; 16] { &ACTOR_RESOURCE_SPECS }
    pub fn spec(&self, key: ActorResourceTableKey) -> Result<&'static ActorResourceSpec, ActorResourceError> {
        actor_resource_spec(key)
    }
    pub fn load(&self, key: ActorResourceTableKey, decoded_anb: Vec<u8>) -> Result<ActorAnimationDatabase, ActorResourceError> {
        ActorAnimationDatabase::load(key, decoded_anb)
    }
    pub fn load_borrowed<'a>(&self, key: ActorResourceTableKey, decoded_anb: &'a [u8]) -> Result<ActorAnimationDatabase<&'a [u8]>, ActorResourceError> {
        ActorAnimationDatabase::load_borrowed(key, decoded_anb)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorResourceError {
    InvalidKey(ActorResourceTableKey),
    UnknownKey(u8),
    WrongLength { key: u8, expected: usize, actual: usize },
    WrongHash { key: u8, expected: &'static str, actual: String },
    Anb(AssetError),
    AnimationCount { key: u8, expected: usize, actual: usize },
    AnimationOutOfBounds { animation: usize, count: usize },
    FrameOutOfBounds { animation: usize, frame: usize, count: usize },
    FrameRangeInvalid { animation: usize, first: u32, last: u32 },
}

impl fmt::Display for ActorResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for ActorResourceError {}

pub fn actor_resource_spec(key: ActorResourceTableKey) -> Result<&'static ActorResourceSpec, ActorResourceError> {
    if key.style > 3 || key.category > 3 || key.flat_index != key.style * 4 + key.category {
        return Err(ActorResourceError::InvalidKey(key));
    }
    ACTOR_RESOURCE_SPECS.get(usize::from(key.flat_index)).ok_or(ActorResourceError::UnknownKey(key.flat_index))
}

/// Runtime-owned, source-bound resource bytes plus a validated sprite table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorAnimationDatabase<B = Vec<u8>> {
    key: ActorResourceTableKey,
    bytes: B,
    sprites: SpriteTable,
    animation_count: usize,
}

impl ActorAnimationDatabase<Vec<u8>> {
    pub fn load(key: ActorResourceTableKey, decoded_anb: Vec<u8>) -> Result<Self, ActorResourceError> {
        Self::validate(key, decoded_anb)
    }
}

impl<'a> ActorAnimationDatabase<&'a [u8]> {
    pub fn load_borrowed(key: ActorResourceTableKey, decoded_anb: &'a [u8]) -> Result<Self, ActorResourceError> {
        Self::validate(key, decoded_anb)
    }
}

impl<B: AsRef<[u8]>> ActorAnimationDatabase<B> {
    fn validate(key: ActorResourceTableKey, decoded_anb: B) -> Result<Self, ActorResourceError> {
        let spec = actor_resource_spec(key)?;
        let source_bytes = decoded_anb.as_ref();
        if source_bytes.len() != spec.decoded_bytes {
            return Err(ActorResourceError::WrongLength { key: key.flat_index, expected: spec.decoded_bytes, actual: source_bytes.len() });
        }
        let actual_hash = sha256_hex(source_bytes);
        if actual_hash != spec.decoded_sha256 {
            return Err(ActorResourceError::WrongHash { key: key.flat_index, expected: spec.decoded_sha256, actual: actual_hash });
        }
        let sections = AnbSections::parse(source_bytes).map_err(ActorResourceError::Anb)?;
        let metadata = sections.ui_metadata().map_err(ActorResourceError::Anb)?;
        if metadata.animation_count() != spec.animation_count {
            return Err(ActorResourceError::AnimationCount { key: key.flat_index, expected: spec.animation_count, actual: metadata.animation_count() });
        }
        // Validate every declared animation and every frame in its inclusive
        // first/last range before admitting the database.
        for animation in 0..metadata.animation_count() {
            let record = metadata.animation(animation).map_err(ActorResourceError::Anb)?;
            let count = animation_frame_count(animation, record)?;
            for frame in 0..count {
                let offset = record.first_frame_offset as usize + frame * 14;
                let frame_record = metadata.frame_at(offset as u32).map_err(ActorResourceError::Anb)?;
                for component in 0..usize::from(frame_record.component_count) {
                    metadata.frame_component(frame_record, component).map_err(ActorResourceError::Anb)?;
                }
            }
        }
        let sprites = SpriteTable::from_anb(source_bytes).map_err(ActorResourceError::Anb)?;
        Ok(Self { key, bytes: decoded_anb, sprites, animation_count: spec.animation_count })
    }

    pub fn key(&self) -> ActorResourceTableKey { self.key }
    pub fn bytes(&self) -> &[u8] { self.bytes.as_ref() }
    pub fn sprite_record_count(&self) -> usize { self.sprites.records().len() }
    pub fn sprite_descriptors(&self) -> &[genewars_assets::SpriteDescriptor] { self.sprites.records() }
    pub fn animation_count(&self) -> usize { self.animation_count }

    pub fn animation(&self, index: usize) -> Result<AnbAnimationRecord, ActorResourceError> {
        let metadata = self.metadata()?;
        metadata.animation(index).map_err(|error| match error {
            AssetError::AnbAnimationOutOfBounds { animation, count } => ActorResourceError::AnimationOutOfBounds { animation, count },
            other => ActorResourceError::Anb(other),
        })
    }

    pub fn frame_count(&self, animation: usize) -> Result<usize, ActorResourceError> {
        animation_frame_count(animation, self.animation(animation)?)
    }

    /// Read one frame inside the declared inclusive first/last range. The
    /// resource bytes do not establish frame timing or loop behavior.
    pub fn frame(&self, animation: usize, frame_index: usize) -> Result<AnbFrameRecord, ActorResourceError> {
        let record = self.animation(animation)?;
        let count = animation_frame_count(animation, record)?;
        if frame_index >= count {
            return Err(ActorResourceError::FrameOutOfBounds { animation, frame: frame_index, count });
        }
        let offset = usize::try_from(record.first_frame_offset).map_err(|_| ActorResourceError::FrameRangeInvalid {
            animation, first: record.first_frame_offset, last: record.last_frame_offset,
        })?.checked_add(frame_index.checked_mul(14).ok_or(ActorResourceError::FrameRangeInvalid {
            animation, first: record.first_frame_offset, last: record.last_frame_offset,
        })?).ok_or(ActorResourceError::FrameRangeInvalid { animation, first: record.first_frame_offset, last: record.last_frame_offset })?;
        self.metadata()?.frame_at(u32::try_from(offset).map_err(|_| ActorResourceError::FrameRangeInvalid {
            animation, first: record.first_frame_offset, last: record.last_frame_offset,
        })?).map_err(ActorResourceError::Anb)
    }

    pub fn component(&self, animation: usize, frame: usize, component: usize) -> Result<genewars_assets::AnbComponentRecord, ActorResourceError> {
        let metadata = self.metadata()?;
        let frame_record = self.frame(animation, frame)?;
        metadata.frame_component(frame_record, component).map_err(ActorResourceError::Anb)
    }

    /// The caller explicitly chooses the already-proved Packed4 codec. This
    /// method does not claim the actor draw path uses Packed4; it only decodes
    /// the referenced record with the supplied base.
    pub fn decode_component_packed4(&self, animation: usize, frame: usize, component: usize, base: u8) -> Result<genewars_assets::IndexedSprite, ActorResourceError> {
        let record = self.component(animation, frame, component)?;
        let sprite_record = usize::try_from(record.sprite_offset / 6).map_err(|_| ActorResourceError::Anb(AssetError::LengthOverflow))?;
        self.sprites.decode_packed4(sprite_record, self.bytes.as_ref(), base).map_err(ActorResourceError::Anb)
    }

    /// Decode the bounded indexed8 form recorded by the existing corpus. This
    /// is an asset-format capability; it does not by itself prove that the
    /// candidate actor draw path selects this codec.
    pub fn decode_component_indexed8(&self, animation: usize, frame: usize, component: usize) -> Result<genewars_assets::IndexedSprite, ActorResourceError> {
        let record = self.component(animation, frame, component)?;
        let sprite_record = usize::try_from(record.sprite_offset / 6).map_err(|_| ActorResourceError::Anb(AssetError::LengthOverflow))?;
        self.sprites.decode(sprite_record, self.bytes.as_ref()).map_err(ActorResourceError::Anb)
    }

    /// Decode any table record using the existing indexed8 RLE reader. Codec
    /// selection remains explicit at the call site.
    pub fn decode_sprite_indexed8(&self, sprite_record: usize) -> Result<genewars_assets::IndexedSprite, ActorResourceError> {
        self.sprites.decode(sprite_record, self.bytes.as_ref()).map_err(ActorResourceError::Anb)
    }

    /// Decode any table record using the existing caller-selected Packed4
    /// reader and explicit base. This is not automatic actor draw admission.
    pub fn decode_sprite_packed4(&self, sprite_record: usize, base: u8) -> Result<genewars_assets::IndexedSprite, ActorResourceError> {
        self.sprites.decode_packed4(sprite_record, self.bytes.as_ref(), base).map_err(ActorResourceError::Anb)
    }

    fn metadata(&self) -> Result<AnbUiMetadata<'_>, ActorResourceError> {
        AnbSections::parse(self.bytes.as_ref()).and_then(|s| s.ui_metadata()).map_err(ActorResourceError::Anb)
    }
}

fn animation_frame_count(animation: usize, record: AnbAnimationRecord) -> Result<usize, ActorResourceError> {
    if record.last_frame_offset < record.first_frame_offset
        || (record.last_frame_offset - record.first_frame_offset) % 14 != 0
    {
        return Err(ActorResourceError::FrameRangeInvalid { animation, first: record.first_frame_offset, last: record.last_frame_offset });
    }
    Ok(((record.last_frame_offset - record.first_frame_offset) / 14 + 1) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_exact_flattened_style_category_pairs() {
        let catalog = ActorResourceCatalog;
        assert_eq!(catalog.specs().len(), 16);
        for (index, spec) in catalog.specs().iter().enumerate() {
            assert_eq!(usize::from(spec.key.flat_index), index);
            assert_eq!(spec.key.flat_index, spec.key.style * 4 + spec.key.category);
            assert_eq!(spec.descriptor_va, 0x104074 + 44 * index as u32);
            assert_eq!(spec.database_slot_va, 0x1168F4 + 4 * index as u32);
            assert_eq!(spec.animation_slot_va, 0x116934 + 4 * index as u32);
            assert_eq!(actor_resource_spec(spec.key).unwrap(), spec);
        }
        assert!(matches!(actor_resource_spec(ActorResourceTableKey { style:1, category:2, flat_index:5 }), Err(ActorResourceError::InvalidKey(_))));
        assert!(matches!(actor_resource_spec(ActorResourceTableKey { style:4, category:0, flat_index:16 }), Err(ActorResourceError::InvalidKey(_))));
    }

    #[test]
    fn frame_ranges_are_inclusive_and_must_be_aligned() {
        assert_eq!(animation_frame_count(0, AnbAnimationRecord { first_frame_offset: 0, last_frame_offset: 0, trailing_word: 0 }).unwrap(), 1);
        assert_eq!(animation_frame_count(1, AnbAnimationRecord { first_frame_offset: 14, last_frame_offset: 42, trailing_word: 0 }).unwrap(), 3);
        assert!(matches!(animation_frame_count(2, AnbAnimationRecord { first_frame_offset: 42, last_frame_offset: 14, trailing_word: 0 }), Err(ActorResourceError::FrameRangeInvalid { .. })));
        assert!(matches!(animation_frame_count(3, AnbAnimationRecord { first_frame_offset: 0, last_frame_offset: 15, trailing_word: 0 }), Err(ActorResourceError::FrameRangeInvalid { .. })));
    }

    #[test]
    fn wrong_length_is_rejected_before_hash_or_container_parse() {
        let spec = &ACTOR_RESOURCE_SPECS[0];
        assert!(matches!(ActorAnimationDatabase::load(spec.key, Vec::new()), Err(ActorResourceError::WrongLength { key: 0, expected: 35497, actual: 0 })));
    }

    #[test]
    fn same_length_wrong_resource_is_rejected_by_hash() {
        let spec = &ACTOR_RESOURCE_SPECS[0];
        assert!(matches!(ActorAnimationDatabase::load(spec.key, vec![0; spec.decoded_bytes]), Err(ActorResourceError::WrongHash { key: 0, .. })));
    }
}
