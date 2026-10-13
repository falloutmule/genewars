//! Pure stage-three construction debit from CD VA 0x53A33.
//! See research/ghidra/construction-cost-bridge.md for scope and prerequisites.

const BASE_COSTS: [i32; 14] = [600, 750, 400, 400, 300, 600, 250, 1000, 300, 500, 900, 650, 600, 300];

/// A safely admitted selector for the CD cost table; building names are unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstructionSelector(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidConstructionSelector(pub i8);

impl TryFrom<i8> for ConstructionSelector {
    type Error = InvalidConstructionSelector;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        if (0..14).contains(&value) {
            Ok(Self(value as u8))
        } else {
            Err(InvalidConstructionSelector(value))
        }
    }
}

impl ConstructionSelector {
    /// Signed-byte adjustment from profile+0x17. No gameplay range is assumed.
    pub fn adjusted_cost(self, skill: i8) -> i32 {
        BASE_COSTS[usize::from(self.0)].wrapping_sub(i32::from(skill).wrapping_mul(2))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstructionPools {
    pub primary: i32,
    pub secondary: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructionDebitStatus {
    NotReady,
    Insufficient,
    Paid,
}

/// Final values of the numeric debit only, excluding other entity side effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstructionDebit {
    pub status: ConstructionDebitStatus,
    /// The original does not evaluate cost when its callback returns zero.
    pub cost: Option<i32>,
    pub pools: ConstructionPools,
    pub stage: u16,
}

/// Apply only the debit branch for an entity already in mode0, stage3.
///
/// `ready` represents the nonzero AL result from vtable callback+0xAC.
/// `refill` represents the independently selected nonzero team flag; its
/// gameplay meaning and relationship to entity owner are not established.
/// Signed comparisons and wrapping32 arithmetic preserve the machine contract,
/// including negative costs. NotReady/Insufficient retain the supplied pools;
/// their original update/cancellation helpers are deliberately outside this API.
pub fn construction_stage3_debit(
    selector: ConstructionSelector,
    skill: i8,
    pools: ConstructionPools,
    refill: bool,
    ready: bool,
) -> ConstructionDebit {
    if !ready {
        return ConstructionDebit { status: ConstructionDebitStatus::NotReady, cost: None, pools, stage: 3 };
    }
    let cost = selector.adjusted_cost(skill);
    if pools.primary.wrapping_add(pools.secondary) < cost {
        return ConstructionDebit { status: ConstructionDebitStatus::Insufficient, cost: Some(cost), pools, stage: 3 };
    }
    let remainder = cost.wrapping_sub(pools.primary);
    let mut updated = if remainder <= 0 {
        ConstructionPools { primary: remainder.wrapping_neg(), secondary: pools.secondary }
    } else {
        ConstructionPools { primary: 0, secondary: pools.secondary.wrapping_sub(remainder) }
    };
    if refill && updated.primary.wrapping_add(updated.secondary) <= 1000 {
        updated.secondary = 1000;
    }
    ConstructionDebit { status: ConstructionDebitStatus::Paid, cost: Some(cost), pools: updated, stage: 4 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pay(primary: i32, secondary: i32, refill: bool) -> ConstructionDebit {
        construction_stage3_debit(ConstructionSelector::try_from(0).unwrap(), 0,
                                 ConstructionPools { primary, secondary }, refill, true)
    }

    #[test]
    fn selector_admission_rejects_negative_and_past_table() {
        for value in i8::MIN..=i8::MAX {
            assert_eq!(ConstructionSelector::try_from(value).is_ok(), (0..14).contains(&value));
        }
    }

    #[test]
    fn exact_affordability_boundary_and_primary_first() {
        assert_eq!(pay(100, 499, false).status, ConstructionDebitStatus::Insufficient);
        let exact = pay(100, 500, false);
        assert_eq!(exact.pools, ConstructionPools { primary: 0, secondary: 0 });
        assert_eq!((exact.status, exact.stage), (ConstructionDebitStatus::Paid, 4));
        assert_eq!(pay(650, 70, false).pools, ConstructionPools { primary: 50, secondary: 70 });
    }

    #[test]
    fn refill_assigns_secondary_not_combined_total() {
        assert_eq!(pay(700, 0, true).pools, ConstructionPools { primary: 100, secondary: 1000 });
        assert_eq!(pay(1600, 0, true).pools, ConstructionPools { primary: 1000, secondary: 1000 });
        assert_eq!(pay(1601, 0, true).pools, ConstructionPools { primary: 1001, secondary: 0 });
    }

    #[test]
    fn preserves_signed_overflow_and_negative_cost() {
        assert_eq!(pay(i32::MAX, 1, false).status, ConstructionDebitStatus::Insufficient);
        let result = construction_stage3_debit(ConstructionSelector::try_from(6).unwrap(), 127,
            ConstructionPools { primary: 0, secondary: 0 }, false, true);
        assert_eq!(result.cost, Some(-4));
        assert_eq!(result.pools, ConstructionPools { primary: 4, secondary: 0 });
    }

    #[test]
    fn callback_prerequisite_preserves_pools_without_evaluating_cost() {
        let pools = ConstructionPools { primary: 100, secondary: 500 };
        let result = construction_stage3_debit(ConstructionSelector::try_from(0).unwrap(), 0, pools, true, false);
        assert_eq!(result, ConstructionDebit { status: ConstructionDebitStatus::NotReady, cost: None, pools, stage: 3 });
    }
}
