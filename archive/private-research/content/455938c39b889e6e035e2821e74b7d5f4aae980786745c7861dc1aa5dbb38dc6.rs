//! Source874B0 conversion for the88x89 minimap control only.
//! Screen origin and map-base bytes come from the actual control context.
//! This function does not infer terrain picking or create a control layout.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapContext {
    pub screen_origin: [i16; 2],
    pub map_base: [u8; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapTarget {
    pub point: [i16; 2],
    pub cell_index: u16,
}

impl MinimapContext {
    pub fn project(self, screen: [i16; 2]) -> Option<MinimapTarget> {
        let x = i32::from(screen[0]) - i32::from(self.screen_origin[0]);
        let y = i32::from(screen[1]) - i32::from(self.screen_origin[1]);
        if !(0..88).contains(&x) || !(0..89).contains(&y) { return None; }
        let diagonal = x - 43;
        let v = y + 6 - (diagonal >> 1);
        let u = diagonal + v;
        let point = [((u + i32::from(self.map_base[0])) << 8) as i16,
                     ((v + i32::from(self.map_base[1])) << 8) as i16];
        let cell_index = ((point[1] as u16 >> 8) & 127) * 128 + ((point[0] as u16 >> 8) & 127);
        Some(MinimapTarget { point, cell_index })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minimap_rectangle_uses_signed_inputs_without_wrapping_subtraction() {
        let context = MinimapContext { screen_origin: [10,20], map_base: [0,0] };
        for screen in [[9,20],[10,19],[98,20],[10,109]] { assert_eq!(context.project(screen), None); }
        assert_eq!(context.project([53,20]), Some(MinimapTarget { point: [1536,1536], cell_index: 774 }));
        assert_eq!(context.project([10,20]).unwrap().point, [-3840,7168]);
        assert_eq!(MinimapContext { screen_origin: [32767,0], map_base: [0,0] }.project([-32768,0]), None);
    }
    #[test]
    fn minimap_coordinates_preserve_low_word_and_use_seven_bit_cell_indices() {
        let context = MinimapContext { screen_origin: [-100,-100], map_base: [255,255] };
        assert_eq!(context.project([-57,-100]).unwrap(), MinimapTarget { point: [1280,1280], cell_index: 645 });
        assert!(context.project([-13,-12]).is_some());
        assert!(context.project([-12,-12]).is_none());
    }
}
