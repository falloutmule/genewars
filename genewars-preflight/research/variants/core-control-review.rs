#[path = "../../rust/crates/genewars-core/src/mouse_packet.rs"] mod mouse_packet;
#[path = "../../rust/crates/genewars-core/src/custom_list.rs"] mod custom_list;
use custom_list::{custom_list_name, CustomListNameError as E};

fn main() {
    let mut packet_cases = 0u64;
    // Independent byte-mask transcription of left-button producer branches.
    for old in 0u8..=255 {
        for latch in [false,true] {
            for physical in [false,true] {
                let bits = if latch {
                    (if old & 4 != 0 { old | 2 } else { old & !2 }) | 5
                } else if physical {
                    (old & !7) | 4
                } else {
                    (if old & 4 != 0 { old | 2 } else { old & !2 }) & !5
                };
                let got = mouse_packet::mode2_button_packet(old & 4 != 0,latch,physical);
                assert_eq!((got.pressed,got.released,got.held),(bits&1!=0,bits&2!=0,bits&4!=0));
                packet_cases += 1;
            }
        }
    }
    let mut storage = vec![0x7f;2250];
    for slot in 0..10 { storage[slot*225+14] = b'A'+slot as u8; storage[slot*225+15] = 0; }
    let mut selection_cases = 0u64;
    for count in 0..=255u8 {
        for selected in 0..=255u8 {
            let got = custom_list_name(&storage,count,selected);
            if count > 10 { assert_eq!(got,Err(E::CountExceedsCapacity)); }
            else if selected >= count { assert_eq!(got,Err(E::SelectionOutsideCount)); }
            else { assert_eq!(got,Ok(&storage[selected as usize*225+14..selected as usize*225+15])); }
            selection_cases += 1;
        }
    }
    for length in 0..2250 {
        assert_eq!(custom_list_name(&storage[..length],10,9),Err(E::StorageTruncated));
    }
    for slot in 0..10usize {
        let start = slot*225;
        for nul in 0..34usize {
            storage[start+14..start+48].fill(b'Q');
            storage[start+14+nul] = 0;
            assert_eq!(custom_list_name(&storage,10,slot as u8).unwrap().len(),nul);
        }
        storage[start+14..start+48].fill(b'Q');
        storage[start+48] = 0;
        assert_eq!(custom_list_name(&storage,10,slot as u8),Err(E::MissingTerminator));
    }
    println!("{{\"packet_byte_cases\":{packet_cases},\"count_selection_cases\":{selection_cases},\"truncated_lengths\":2250,\"name_nul_positions\":340,\"following_field_nuls_rejected\":10,\"status\":\"passed\"}}");
}
