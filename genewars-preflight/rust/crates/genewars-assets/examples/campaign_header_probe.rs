use std::{env, fs::File, io::Read, path::PathBuf, process};

use genewars_assets::CampaignHeader;

fn main() {
    if let Err(error) = run() {
        eprintln!("campaign_header_probe: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.is_empty() || args.len() > 64 {
        return Err("usage: campaign_header_probe <explicit-699-byte-header>... (max 64 files)".into());
    }

    for path in args.into_iter().map(PathBuf::from) {
        let metadata = std::fs::metadata(&path)?;
        if metadata.len() != 699 {
            return Err(format!("input must be exactly 699 bytes: {}", path.display()).into());
        }
        let mut bytes = Vec::with_capacity(699);
        File::open(&path)?.take(700).read_to_end(&mut bytes)?;
        let header = CampaignHeader::parse(&bytes)?;
        println!("file={} bytes={} bounded-string-length={}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            header.bytes().len(),
            header.title_bytes().len());
    }
    Ok(())
}
