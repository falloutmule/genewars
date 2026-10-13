#[path = "../rust/crates/genewars-runtime/src/family8_creation.rs"]
mod family8_creation;
#[path = "../rust/crates/genewars-runtime/src/family8_pool.rs"]
mod family8_pool;
use family8_creation::{CreationInput, plan_creation};
use family8_pool::{Family8Pool, AnimationBinding, PoolError};
fn main() {
    let mut packed = vec![0; 16 * 94];
    for (i, value) in packed.iter_mut().enumerate() { *value = (i * 73) as u8; }
    for slot in 0..16 { packed[slot * 94 + 6] = 0; }
    let prior = std::array::from_fn(|slot| std::array::from_fn(|i| (slot + i * 17) as u8));
    let mut pool = Family8Pool::from_packed_inactive(&packed, &prior, [None; 16]).unwrap();
    for slot in 0..16 {
        let actual = pool.record(slot).unwrap();
        assert_eq!(&actual[..31], &packed[slot * 94..slot * 94 + 31]);
        assert_eq!(&actual[31..39], &prior[slot][31..39]);
        assert_eq!(&actual[39..], &packed[slot * 94 + 31..(slot + 1) * 94]);
    }
    for slot in 0..16 {
        let before = *pool.record(slot).unwrap();
        let input = CreationInput { slot: slot as u16, owner: (slot % 4) as u8,
            selector: (slot % 3) as u8, position: [0, 0], ground_height: 128,
            initial_height: 25728, shade: 31, old_head: None };
        let plan = plan_creation(&before, input).unwrap();
        let mut broken = plan.clone(); broken.after[31] ^= 1;
        let unchanged = pool.clone();
        assert_eq!(pool.commit_creation(slot, &before, &broken), Err(PoolError::PointerBytesChanged(slot)));
        assert_eq!(pool, unchanged);
        pool.commit_creation(slot, &before, &plan).unwrap();
        assert_eq!(pool.binding(slot).unwrap(), Some(if slot % 3 == 1 { AnimationBinding::Base } else { AnimationBinding::Offset80 }));
        let after = *pool.record(slot).unwrap();
        assert_eq!(pool.commit_creation(slot, &before, &plan), Err(PoolError::StaleRecord(slot)));
        let mut next = after; next[3..6].copy_from_slice(&[7, 0, 2]);
        pool.replace_record_checked(slot, &after, &next).unwrap();
        let stable = pool.clone();
        let mut wrong = next; wrong[0x55] ^= 1;
        assert_eq!(pool.replace_record_checked(slot, &next, &wrong), Err(PoolError::IdentityChanged(slot)));
        assert_eq!(pool, stable);
    }
    assert_eq!(pool.records().iter().filter(|r| r[6] == 8).count(), 16);
    packed[15 * 94 + 6] = 8;
    assert_eq!(Family8Pool::from_packed_inactive(&packed, &prior, [None; 16]), Err(PoolError::ActivePackedRecord(15)));
    println!("PASS family8 pool:16 gap-preserving loads and creations, stale/identity/pointer rejection, exact bindings");
}
