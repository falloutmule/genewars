//! Draft only: inactive packed-to-native family-3 loader.
//! Do not copy into building_pool.rs until the owning root releases the freeze.

impl BuildingPool {
    pub fn from_packed_inactive(
        packed: &[u8],
        prior: &[BuildingRecord; 256],
        bindings: [Option<BuildingAnimationBinding>; 256],
    ) -> Result<Self, BuildingPoolError> {
        if packed.len() != 256 * 59 {
            return Err(BuildingPoolError::PackedLength(packed.len()));
        }

        let mut records = *prior;
        for (slot, source) in packed.chunks_exact(59).enumerate() {
            // Source 45010 calls a virtual initializer for nonzero +6. This
            // API admits only the source-inactive path and rejects beforehand.
            if source[6] != 0 {
                return Err(BuildingPoolError::ActivePackedRecord(slot));
            }
            if bindings[slot].is_some() {
                return Err(BuildingPoolError::InvalidBinding(slot));
            }

            // 6D490 copies 31 bytes; 45010 copies the remaining 28 bytes.
            // The eight-byte native-only gap is preserved from `prior`.
            records[slot][..31].copy_from_slice(&source[..31]);
            records[slot][39..67].copy_from_slice(&source[31..59]);
            validate(slot, &records[slot], bindings[slot])?;
        }
        Ok(Self { records, bindings })
    }
}

// Proposed additions to BuildingPoolError:
//   PackedLength(usize), ActivePackedRecord(usize)
