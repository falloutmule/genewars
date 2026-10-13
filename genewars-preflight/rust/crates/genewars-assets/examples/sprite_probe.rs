use std::{env, fs, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("sprite_probe: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 3 {
        return Err("usage: sprite_probe <anb-database> <sprite-table.tab>".into());
    }
    let database_path = &args[1];
    let table_path = &args[2];
    if fs::metadata(database_path)?.len() > 64 * 1024 * 1024
        || fs::metadata(table_path)?.len() > 64 * 1024 * 1024
    {
        return Err("input exceeds 64 MiB safety ceiling".into());
    }
    let database = fs::read(database_path)?;
    let table_bytes = fs::read(table_path)?;
    let table = genewars_assets::SpriteTable::parse(&table_bytes, &database)?;
    if table.records().len() > 4096 {
        return Err("sprite table exceeds 4096 records".into());
    }
    let mut decoded_records = 0usize;
    let mut decoded_pixels = 0usize;
    let mut consumed_bytes = 0usize;
    for index in 0..table.records().len() {
        let sprite = table.decode(index, &database)?;
        decoded_pixels = decoded_pixels.checked_add(sprite.indices.len()).ok_or("pixel count overflow")?;
        if decoded_pixels > 64 * 1024 * 1024 {
            return Err("decoded sprite area exceeds 64 MiB safety ceiling".into());
        }
        consumed_bytes = consumed_bytes.checked_add(sprite.consumed_bytes).ok_or("byte count overflow")?;
        if !table.records()[index].is_empty() {
            decoded_records += 1;
        }
    }
    println!("records={} decoded={} pixels={} rle-bytes={}",
        table.records().len(), decoded_records, decoded_pixels, consumed_bytes);
    Ok(())
}
