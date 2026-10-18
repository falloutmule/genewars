use std::{env, fs, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("rnc_decode: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 4 {
        return Err("usage: rnc_decode <input.rnc> <output.bin> <output-limit-bytes>".into());
    }
    let limit: usize = args[3].to_string_lossy().parse()?;
    let input_path = &args[1];
    let input_len = fs::metadata(input_path)?.len();
    if input_len > 64 * 1024 * 1024 {
        return Err("input exceeds 64 MiB safety ceiling".into());
    }
    let input = fs::read(input_path)?;
    let header = genewars_assets::Header::parse(&input)?;
    let output = genewars_assets::decode(&input, limit)?;
    fs::write(&args[2], &output)?;
    println!("method={} packed={} unpacked={} output={}",
        header.method, header.packed_size, header.unpacked_size, output.len());
    Ok(())
}
