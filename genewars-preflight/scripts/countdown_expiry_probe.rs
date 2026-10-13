//! Independently authored caller-boundary probe; all fixture bytes stay private.
#[path = "../rust/crates/genewars-runtime/src/countdown_expiry.rs"]
mod countdown_expiry;
use countdown_expiry::{plan_expiry_requests, ExpiryRequestError, PLAYER_BYTES};
use std::{fs, path::Path};

fn main() {
    let argument = std::env::args().nth(1).unwrap();
    let folder = Path::new(&argument);
    let input = fs::read(folder.join("input.bin")).unwrap();
    let map = fs::read(folder.join("map.bin")).unwrap();
    let mut output = Vec::new();
    for case in input.chunks_exact(1 + 4 * PLAYER_BYTES) {
        let players: Vec<[u8; PLAYER_BYTES]> = case[1..]
            .chunks_exact(PLAYER_BYTES).map(|r| r.try_into().unwrap()).collect();
        let requests = plan_expiry_requests(&players[..usize::from(case[0])], |point| {
            genewars_core::cd_candidate::interpolate_height(&map, point[0], point[1])
        }).unwrap();
        output.push(requests.len() as u8);
        for request in requests {
            output.extend([request.owner, request.selector]);
            for word in request.position { output.extend(word.to_le_bytes()); }
            output.extend(request.initial_height.to_le_bytes());
        }
    }
    assert_eq!(plan_expiry_requests(&[[0; PLAYER_BYTES]; 5], |_| -> Result<i32, ()> {
        panic!("invalid extent must reject before queries")
    }), Err(ExpiryRequestError::TooManyPlayers));
    let mut calls = 0;
    assert_eq!(plan_expiry_requests(&[[0; PLAYER_BYTES]; 4], |_| {
        calls += 1;
        if calls == 2 { Err("blocked height sample") } else { Ok(i32::MAX) }
    }), Err(ExpiryRequestError::HeightSample("blocked height sample")));
    assert_eq!(calls, 2);
    let wrapped = plan_expiry_requests(&[[0; PLAYER_BYTES]; 1], |_| Ok::<_, ()>(i32::MAX)).unwrap();
    assert_eq!(wrapped[0].initial_height, i32::MAX.wrapping_add(0x6400));
    fs::write(folder.join("actual.bin"), output).unwrap();
}
