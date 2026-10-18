//! Hash-bound CD physics constants used by the shared landing/effect update.
//! Runtime water and gravity remain separate state, not guessed constants.

pub const ANGLE_TABLE_SHA256: &str = "1c26f16f88a3e0cedc9c879837b3849567ff70514c033eb6249bd95a264debce";
pub const BOUNCE_TABLE_SHA256: &str = "c62a747206c89218e01102645341f151f5308fe1259b177f9d0418a67575e5e5";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicsTableError {
    Length { name: &'static str, expected: usize, actual: usize },
    Hash { name: &'static str, actual: String },
    TerrainClass(u8),
}
impl core::fmt::Display for PhysicsTableError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f,"{self:?}") }
}
impl std::error::Error for PhysicsTableError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicsTables { angles: [u16;257], bounce: [i16;8] }
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
        })
    }
    pub fn angles(&self)->&[u16;257] { &self.angles }
    pub fn bounce(&self, terrain_class:u8)->Result<i16,PhysicsTableError> {
        self.bounce.get(usize::from(terrain_class)).copied().ok_or(PhysicsTableError::TerrainClass(terrain_class))
    }
    #[cfg(test)]
    pub(crate) fn fixture()->Self { Self { angles:[0;257],bounce:[32;8] } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unbound_and_truncated_physics_tables() {
        assert!(matches!(PhysicsTables::from_mapped_bytes(&[0;514],&[0;16]),Err(PhysicsTableError::Hash { name:"angle",.. })));
        assert!(matches!(PhysicsTables::from_mapped_bytes(&[0;512],&[0;16]),Err(PhysicsTableError::Length { name:"angle",.. })));
    }
}
