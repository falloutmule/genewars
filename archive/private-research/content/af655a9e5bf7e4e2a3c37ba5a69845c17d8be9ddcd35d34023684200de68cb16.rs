//! Bounded, tool-authored `GWPLANT1` snapshots for explicit plant diagnostics.
//!
//! This is a diagnostic input format, not a game resource or inferred world
//! state. Queueing and drawing remain separate caller operations.

use std::{collections::BTreeSet, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlantSnapshotRecord {
    pub id: u32,
    pub slot: u8,
    pub animation: u8,
    pub frame: u8,
    pub x: i32,
    pub y: i32,
    pub shade: u8,
    pub flags: u8,
    pub jitter: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantSceneSnapshot {
    pub active: bool,
    pub initial_count: i16,
    pub margin: u16,
    pub gate: bool,
    pub mode: u8,
    pub context_flags: u8,
    pub plants: Vec<PlantSnapshotRecord>,
}

fn number<T: FromStr>(word: &str) -> Result<T, String> {
    word.parse().map_err(|_| format!("invalid or out-of-range snapshot number: {word}"))
}

fn bit(word: &str) -> Result<bool, String> {
    match word {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err("snapshot booleans must be0 or1".into()),
    }
}

impl PlantSceneSnapshot {
    /// Parse the strict bounded text format used by explicit scene diagnostics.
    /// Empty/whitespace-only lines are ignored, matching the tools parser.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 1024 * 1024 { return Err("snapshot exceeds1 MiB diagnostic limit".into()); }
        let text = std::str::from_utf8(bytes).map_err(|_| "snapshot must be UTF-8")?;
        let mut lines = text.lines().filter(|line| !line.trim().is_empty());
        if lines.next() != Some("GWPLANT1") { return Err("snapshot must start with GWPLANT1".into()); }
        let words: Vec<_> = lines.next().ok_or("missing snapshot context")?.split_whitespace().collect();
        if words.len() != 7 || words[0] != "context" {
            return Err("expected context ACTIVE INITIAL_COUNT MARGIN GATE MODE CONTEXT_FLAGS".into());
        }
        let active = bit(words[1])?;
        let initial_count: i16 = number(words[2])?;
        let margin: u16 = number(words[3])?;
        let gate = bit(words[4])?;
        let mode = number(words[5])?;
        let context_flags = number(words[6])?;
        if !(0..=256).contains(&initial_count) || margin != 128 {
            return Err("scene snapshot requires count0..256 and margin128".into());
        }

        let mut plants = Vec::new();
        let mut ids = BTreeSet::new();
        for line in lines {
            if plants.len() == 4096 { return Err("snapshot exceeds4096 diagnostic candidates".into()); }
            let words: Vec<_> = line.split_whitespace().collect();
            if words.len() != 10 || words[0] != "plant" {
                return Err("expected plant ID SLOT ANIM FRAME X Y SHADE FLAGS JITTER".into());
            }
            let id = number(words[1])?;
            if !ids.insert(id) { return Err("duplicate snapshot ID".into()); }
            let slot: u8 = number(words[2])?;
            if slot >= 12 { return Err("snapshot slot must be0..11".into()); }
            plants.push(PlantSnapshotRecord {
                id,
                slot,
                animation: number(words[3])?,
                frame: number(words[4])?,
                x: number(words[5])?,
                y: number(words[6])?,
                shade: number(words[7])?,
                flags: number(words[8])?,
                jitter: number(words[9])?,
            });
        }
        Ok(Self { active, initial_count, margin, gate, mode, context_flags, plants })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_explicit_snapshot_and_preserves_fields() {
        let input = b"GWPLANT1\ncontext 1 2 128 1 0 4\nplant 17 3 1 2 -40 479 29 8 0\n";
        let parsed = PlantSceneSnapshot::parse(input).unwrap();
        assert_eq!(parsed.active, true);
        assert_eq!(parsed.initial_count, 2);
        assert_eq!(parsed.margin, 128);
        assert_eq!(parsed.gate, true);
        assert_eq!(parsed.mode, 0);
        assert_eq!(parsed.context_flags, 4);
        assert_eq!(parsed.plants, [PlantSnapshotRecord { id: 17, slot: 3, animation: 1,
            frame: 2, x: -40, y: 479, shade: 29, flags: 8, jitter: 0 }]);
    }

    #[test]
    fn malformed_or_ambiguous_snapshots_are_rejected() {
        for input in [
            b"GWPLANT1\ncontext 1 257 128 1 0 0\n".as_slice(),
            b"GWPLANT1\ncontext 1 0 127 1 0 0\n",
            b"GWPLANT1\ncontext yes 0 128 1 0 0\n",
            b"GWPLANT1\ncontext 1 0 128 1 0 0\nplant 1 12 0 0 0 0 0 0 0\n",
            b"GWPLANT1\ncontext 1 0 128 1 0 0\nplant 1 0 0 0 0 0 0 0 0\nplant 1 0 0 0 0 0 0 0 0\n",
        ] { assert!(PlantSceneSnapshot::parse(input).is_err()); }
        assert!(PlantSceneSnapshot::parse(&vec![b' '; 1024 * 1024 + 1]).is_err());
        assert!(PlantSceneSnapshot::parse(&[0xff]).is_err());
    }
}
