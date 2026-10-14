//! Hash-bound CD physics constants used by the shared landing/effect update.
//! Runtime water and gravity remain separate state, not guessed constants.

pub const ANGLE_TABLE_SHA256: &str = "1c26f16f88a3e0cedc9c879837b3849567ff70514c033eb6249bd95a264debce";
pub const BOUNCE_TABLE_SHA256: &str = "c62a747206c89218e01102645341f151f5308fe1259b177f9d0418a67575e5e5";

pub const CORPSE_FRICTION_TABLE_SHA256: &str = "d56e59eae4da3dcb6d5cd9451d25e0ca49259a3a3004f43a40b47c40ae63a707";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicsTableError {
    Length { name: &'static str, expected: usize, actual: usize },
    Hash { name: &'static str, actual: String },
    TerrainClass(u8),
    MissingTable(&'static str),
}
impl core::fmt::Display for PhysicsTableError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f,"{self:?}") }
}
impl std::error::Error for PhysicsTableError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicsTables { angles: [u16;257], bounce: [i16;8], corpse_friction: Option<[i16;8]> }
impl PhysicsTables {
    pub fn from_mapped_bytes(angles: &[u8], bounce: &[u8]) -> Result<Self,PhysicsTableError> {
        for (name,bytes,expected,hash) in [("angle",angles,514,ANGLE_TABLE_SHA256),("bounce",bounce,16,BOUNCE_TABLE_SHA256)] {
            if bytes.len()!=expected { return Err(PhysicsTableError::Length { name,expected,actual:bytes.len() }); }
            let actual=crate::identity::sha256_hex(bytes);
            if actual!=hash { return Err(PhysicsTableError::Hash { name,actual }); }
        }
        Ok(Self {
            angles: std::array::from_fn(|i|u16::from_le_bytes([angles[2*i],angles[2*i+1]])),
            bounce: std::array::from_fn(|i|i16::from_le_bytes([bounce[2*i],bounce[2*i+1]])),
            corpse_friction: None,
        })
    }
    /// Source104520 is independent of the104530 bounce table. This optional
    /// binding is required only when a moving corpse enters its friction path.
    pub fn with_corpse_friction(mut self, bytes:&[u8])->Result<Self,PhysicsTableError> {
        if bytes.len()!=16 { return Err(PhysicsTableError::Length{name:"corpse-friction",expected:16,actual:bytes.len()}); }
        let actual=crate::identity::sha256_hex(bytes);
        if actual!=CORPSE_FRICTION_TABLE_SHA256 { return Err(PhysicsTableError::Hash{name:"corpse-friction",actual}); }
        self.corpse_friction=Some(std::array::from_fn(|i|i16::from_le_bytes([bytes[2*i],bytes[2*i+1]])));
        Ok(self)
    }
    pub fn friction(&self, terrain_class:u8)->Result<i16,PhysicsTableError> {
        if terrain_class>=8 { return Err(PhysicsTableError::TerrainClass(terrain_class)); }
        self.corpse_friction.as_ref().ok_or(PhysicsTableError::MissingTable("corpse-friction"))?
            .get(usize::from(terrain_class)).copied().ok_or(PhysicsTableError::TerrainClass(terrain_class))
    }
    pub fn angles(&self)->&[u16;257] { &self.angles }
    pub fn bounce(&self, terrain_class:u8)->Result<i16,PhysicsTableError> {
        self.bounce.get(usize::from(terrain_class)).copied().ok_or(PhysicsTableError::TerrainClass(terrain_class))
    }
    #[cfg(test)]
    pub(crate) fn fixture()->Self { Self { angles:[0;257],bounce:[32;8],corpse_friction:None } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_corpse_friction_rejects_absent_corrupt_and_truncated_inputs() {
        assert_eq!(PhysicsTables::fixture().friction(0),Err(PhysicsTableError::MissingTable("corpse-friction")));
        assert_eq!(PhysicsTables::fixture().friction(8),Err(PhysicsTableError::TerrainClass(8)));
        assert!(matches!(PhysicsTables::fixture().with_corpse_friction(&[0;16]),Err(PhysicsTableError::Hash{name:"corpse-friction",..})));
        assert!(matches!(PhysicsTables::fixture().with_corpse_friction(&[0;15]),Err(PhysicsTableError::Length{name:"corpse-friction",..})));
    }
    #[test]
    fn rejects_unbound_and_truncated_physics_tables() {
        assert!(matches!(PhysicsTables::from_mapped_bytes(&[0;514],&[0;16]),Err(PhysicsTableError::Hash { name:"angle",.. })));
        assert!(matches!(PhysicsTables::from_mapped_bytes(&[0;512],&[0;16]),Err(PhysicsTableError::Length { name:"angle",.. })));
    }
}
