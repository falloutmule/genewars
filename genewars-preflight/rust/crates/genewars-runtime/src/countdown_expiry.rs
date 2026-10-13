//! Source 13780's ordered factory arguments when the mission timer expires.
//! This planner does not create family8 records or consume presentation work.

pub const PLAYER_BYTES: usize = 1805;

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
