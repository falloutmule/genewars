//! Standalone cross-check for the bounded PLGBL status overlay vectors.
//! This is proof tooling, not a production renderer or scene assembler.
use std::{env, fs, path::PathBuf};

const W: usize = 640;
const H: usize = 480;
const P: usize = 648;

#[derive(Debug)]
struct Sprite { w: usize, h: usize, px: Vec<u8>, mask: Vec<bool>, x: i16, y: i16, flags: u8 }

fn u16at(b: &[u8], i: usize) -> u16 { u16::from_le_bytes([b[i], b[i + 1]]) }
fn i16at(b: &[u8], i: usize) -> i16 { i16::from_le_bytes([b[i], b[i + 1]]) }
fn u32at(b: &[u8], i: usize) -> usize { u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize }
fn signed_byte(b: u8) -> i8 { b as i8 }

fn load_sprite(anb: &[u8], anim: usize) -> Result<Sprite, String> {
    if anb.len() < 18 { return Err("short ANB header".into()); }
    let count = u16at(anb, 0) as usize;
    let o = [u32at(anb, 2), u32at(anb, 6), u32at(anb, 10), u32at(anb, 14)];
    if count != 44 || o != [35670, 36300, 37860, 39974] || anb.len() != o[3] + count * 10 {
        return Err("PLGBL layout mismatch".into());
    }
    let ae = o[3] + anim * 10;
    let first = u32at(anb, ae);
    let last = u32at(anb, ae + 4);
    if first != last || first % 14 != 0 { return Err("not single-frame overlay".into()); }
    let fa = o[2] + first;
    let co = u32at(anb, fa);
    let n = anb[fa + 11] as usize;
    if n != 1 { return Err("overlay component count".into()); }
    let ca = o[1] + co;
    let so = u32at(anb, ca);
    let sx = i16at(anb, ca + 4) >> 1;
    let sy = i16at(anb, ca + 6) >> 1;
    let flags = anb[ca + 8];
    if flags != 0 || so % 6 != 0 { return Err("unadmitted component flags/reference".into()); }
    let sa = o[0] + so;
    let dat_off = u32at(anb, sa);
    let w = anb[sa + 4] as usize;
    let h = anb[sa + 5] as usize;
    let mut px = vec![0u8; w * h];
    let mut mask = vec![false; w * h];
    let mut cur = 0x12usize.checked_add(dat_off).ok_or("sprite offset overflow")?;
    if cur >= o[0] { return Err("sprite stream beyond database prefix".into()); }
    for row in 0..h {
        let mut x = 0usize;
        loop {
            if cur >= o[0] { return Err("truncated sprite control".into()); }
            let control = signed_byte(anb[cur]);
            cur += 1;
            if control == 0 { break; }
            let run = control.unsigned_abs() as usize;
            if control > 0 {
                if cur + run > o[0] || x + run > w { return Err("literal run overflow".into()); }
                for j in 0..run {
                    let at = row * w + x;
                    px[at] = anb[cur + j];
                    mask[at] = true;
                    x += 1;
                }
                cur += run;
            } else {
                x += run;
                if x > w { return Err("skip run overflow".into()); }
            }
        }
    }
    Ok(Sprite { w, h, px, mask, x: sx, y: sy, flags })
}

fn draw(surface: &mut [u8], sprite: &Sprite, origin: [i16; 2], gla: &[u8], flags: u8) -> Result<(usize, usize), String> {
    if surface.len() < P * (H - 1) + W { return Err("short surface".into()); }
    if flags != 0 && flags != 4 { return Err("unsupported base flags".into()); }
    if flags == 4 && gla.len() != 65536 { return Err("short GLA".into()); }
    if sprite.flags != 0 { return Err("component flags not admitted".into()); }
    let x0 = i32::from(origin[0]) + i32::from(sprite.x);
    let y0 = i32::from(origin[1]) + i32::from(sprite.y);
    let mut written = 0;
    let mut clipped = 0;
    for y in 0..sprite.h {
        for x in 0..sprite.w {
            let at = y * sprite.w + x;
            if !sprite.mask[at] { continue; }
            let dx = x0 + x as i32;
            let dy = y0 + y as i32;
            if dx < 0 || dx >= W as i32 || dy < 0 || dy >= H as i32 { clipped += 1; continue; }
            let dst = dy as usize * P + dx as usize;
            let src = sprite.px[at];
            surface[dst] = if flags == 4 { gla[((src as usize) << 8) | surface[dst] as usize] } else { src };
            written += 1;
        }
    }
    Ok((written, clipped))
}

fn frame_pattern(mult: usize, add: usize) -> Vec<u8> {
    (0..P * H).map(|i| (i.wrapping_mul(mult).wrapping_add(add) & 0xff) as u8).collect()
}

fn make_record(owner: u8, shade: u8, flags: u16, grade: u8, grade_cmp: u8, health: i16, parity: u8) -> [u8; 67] {
    let mut r = [0u8; 67];
    r[0x18] = owner;
    r[0x17] = shade;
    r[0x2b..0x2d].copy_from_slice(&health.to_le_bytes());
    r[0x2e] = grade;
    r[0x2f] = grade_cmp;
    r[0x30] = parity;
    r[0x31..0x33].copy_from_slice(&flags.to_le_bytes());
    r
}

fn select(r: &[u8; 67], origin: [i16; 2], owner: u8, frame: u32, threshold: Option<i16>) -> Result<Option<(usize, [i16; 2])>, String> {
    if r[0x18] != owner || r[0x17] == 0 || frame & 0x3f < 0x20 { return Ok(None); }
    let flags = u16at(r, 0x31);
    let anim = if flags & 0x100 != 0 { 11 }
    else if flags & 1 == 0 { if r[0x2e] == r[0x2f] { return Ok(None); } 12 }
    else {
        let limit = threshold.ok_or("completed branch requires explicit threshold")?;
        let mut a = flags & 4 == 0;
        let mut b = i16at(r, 0x2b) < limit;
        if a && b {
            if frame & 0x7f < 0x40 { b = false; } else { a = false; }
        }
        if a { 2 } else if b { 13 } else { return Ok(None); }
    };
    let adjust = if r[0x30] > 1 { (r[0x30] & 1) as i32 } else { 0 };
    let x = (i32::from(origin[0]) - adjust) as i16;
    let y = (i32::from(origin[1]) - 10) as i16;
    Ok(Some((anim, [x, y])))
}

fn named_surface(out: &PathBuf, name: &str, surface: &[u8]) -> Result<(), String> {
    fs::write(out.join(format!("{name}.bin")), surface).map_err(|e| e.to_string())
}

fn run() -> Result<(), String> {
    let a: Vec<String> = env::args().collect();
    if a.len() != 4 { return Err("usage: harness PLGBL-decoded.bin MAIN.GLA output-dir".into()); }
    let anb = fs::read(&a[1]).map_err(|e| e.to_string())?;
    let gla = fs::read(&a[2]).map_err(|e| e.to_string())?;
    if gla.len() != 65536 { return Err("MAIN.GLA extent".into()); }
    let out = PathBuf::from(&a[3]);
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let cases: [(&str, [u8; 67], u32, Option<i16>); 7] = [
        ("high_flag", make_record(2, 1, 0x100, 0, 1, 1, 2), 0x20, Some(20)),
        ("incomplete_grade", make_record(2, 1, 0, 0, 1, 1, 2), 0x20, None),
        ("completed_flag4_clear_pre_phase", make_record(2, 1, 1, 0, 1, 5, 2), 0x20, Some(10)),
        ("completed_flag4_clear_post_phase", make_record(2, 1, 1, 0, 1, 5, 2), 0x60, Some(10)),
        ("completed_flag4_set_low_health", make_record(2, 1, 5, 0, 1, 5, 2), 0x60, Some(10)),
        ("completed_flag4_set_high_health", make_record(2, 1, 5, 0, 1, 20, 2), 0x60, Some(10)),
        ("odd_parity_origin_adjust", make_record(2, 1, 0x100, 0, 1, 1, 3), 0x20, Some(20)),
    ];
    for (name, r, frame, threshold) in cases {
        let mut surface = frame_pattern(17, 9);
        if let Some((anim, origin)) = select(&r, [123, 90], 2, frame, threshold)? {
            let spr = load_sprite(&anb, anim)?;
            draw(&mut surface, &spr, origin, &gla, 0)?;
        }
        named_surface(&out, name, &surface)?;
    }
    for flags in [0u8, 4] {
        let mut surface = frame_pattern(29, 7);
        let spr = load_sprite(&anb, 11)?;
        draw(&mut surface, &spr, [320, 230], &gla, flags)?;
        named_surface(&out, &format!("flag_path_{flags}"), &surface)?;
    }
    let mut clip = vec![0xa5; P * H];
    let spr = load_sprite(&anb, 11)?;
    let (written, clipped) = draw(&mut clip, &spr, [-6, -6], &gla, 0)?;
    if written != 10 || clipped != 399 { return Err(format!("unexpected clipping vector {written}/{clipped}")); }
    named_surface(&out, "clipping", &clip)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() { eprintln!("overlay harness: {e}"); std::process::exit(1); }
}
