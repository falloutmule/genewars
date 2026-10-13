//! Source 13780's ordered factory arguments when the mission timer expires.
//! This planner does not create family8 records or consume presentation work.

pub const PLAYER_BYTES: usize = 1805;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountdownTransition {
    pub before: u16,
    pub after: u16,
    pub started: bool,
    pub expired: bool,
}

/// Numeric prefix of13780 before factories and the ascending live-slot pass.
/// A live selector0 prevents restart only while the countdown is already zero.
pub fn plan_countdown_transition(count: u16, period: u16, progress: i8,
    player_byte607: u8, runtime_flags: u8, active_selector0: bool) -> CountdownTransition {
    let special = matches!(progress, 14 | 20);
    if count == 0 {
        if active_selector0 { return CountdownTransition { before: count, after: 0, started: false, expired: false }; }
        let after = if special { 3000 } else if runtime_flags & 0xc0 != 0 { u16::MAX } else { period };
        return CountdownTransition { before: count, after, started: true, expired: false };
    }
    let decrement = if special { player_byte607 != 0 } else { count != u16::MAX };
    let after = if decrement { count - 1 } else { count };
    CountdownTransition { before: count, after, started: false, expired: decrement && after == 0 }
}

/// Pending source presentation operations. These are requests, not audio or
/// UI execution. Descriptor offsets belong to the hash-bound11680C table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountdownPresentation {
    StartNarration { selected_player: u8 },
    ExpiryEffect3a { selected_player: u8 },
    ExpiryNarration { selected_player: u8 },
    ResetUiByte185ec1,
    AdvanceUiAnimation { descriptor_offset: u16, state_address: u32 },
}

/// Prefix notices in source order. Animation updates follow all callbacks;
/// callers must append `animation_requests` only after that pass succeeds.
pub fn transition_requests(plan: CountdownTransition, selected_player: u8) -> Vec<CountdownPresentation> {
    use CountdownPresentation::*;
    if plan.started { vec![StartNarration { selected_player }, ResetUiByte185ec1] }
    else if plan.expired { vec![ExpiryEffect3a { selected_player }, ExpiryEffect3a { selected_player }, ExpiryNarration { selected_player }] }
    else { Vec::new() }
}

pub fn animation_requests() -> [CountdownPresentation; 8] {
    core::array::from_fn(|i| if i == 0 {
        CountdownPresentation::AdvanceUiAnimation { descriptor_offset: 0x15e, state_address: 0x184d04 }
    } else {
        CountdownPresentation::AdvanceUiAnimation { descriptor_offset: 0x118 + (i as u16 - 1) * 10, state_address: 0x184d0c + (i as u32 - 1) * 5 }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpirySpawnRequest {
    pub owner: u8,
    pub selector: u8,
    pub position: [i16; 2],
    pub initial_height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpiryRequestError<E> {
    TooManyPlayers,
    HeightSample(E),
}

/// Enter only after the caller proves the countdown transitioned to zero.
/// Source byte6E3 gates each player; signed building-count byte4A2 chooses
/// the position at5FC when positive, otherwise the position at9. Factory
/// capacity checks and the same-frame family8 callbacks are separate work.
pub fn plan_expiry_requests<E>(
    players: &[[u8; PLAYER_BYTES]],
    mut sample_height: impl FnMut([i16; 2]) -> Result<i32, E>,
) -> Result<Vec<ExpirySpawnRequest>, ExpiryRequestError<E>> {
    if players.len() > 4 {
        return Err(ExpiryRequestError::TooManyPlayers);
    }
    let mut requests = Vec::new();
    for (owner, player) in players.iter().enumerate() {
        if player[0x6e3] != 0 {
            continue;
        }
        let at = if player[0x4a2] as i8 > 0 { 0x5fc } else { 9 };
        let position = [
            i16::from_le_bytes([player[at], player[at + 1]]),
            i16::from_le_bytes([player[at + 2], player[at + 3]]),
        ];
        let height = sample_height(position).map_err(ExpiryRequestError::HeightSample)?;
        requests.push(ExpirySpawnRequest {
            owner: owner as u8,
            selector: 0,
            position,
            initial_height: height.wrapping_add(0x6400),
        });
    }
    Ok(requests)
}
