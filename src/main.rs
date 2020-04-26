extern crate clap;
extern crate env_logger;
extern crate flexbuffers;
extern crate lmdb;
extern crate log;
extern crate serde_json;
use lmdb::Transaction;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

mod deepcopy;
mod embed;

// Build with --features=logging_allocator and run with env RUST_LOG=trace to log allocations.
#[cfg(feature = "logging_allocator")]
extern crate logging_allocator;
#[cfg(feature = "logging_allocator")]
#[global_allocator]
static ALLOC: logging_allocator::LoggingAllocator = logging_allocator::LoggingAllocator::new();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    #[cfg(feature = "logging_allocator")]
    ALLOC.enable_logging();
    let matches = clap::App::new("Embed")
        .arg_from_usage("-D, --datadir <DATADIR> 'LMDB database directory'")
        .arg_from_usage("--mapsize [MB] 'LMDB map size in MB'")
        .arg_from_usage("--frame [FRAME] 'Frame'")
        .arg_from_usage("--out [FILE] 'output file'")
        .arg_from_usage("--json 'output json'")
        .arg_from_usage("<ROOT>.. 'Root path'")
        .get_matches();

    let datadir = Path::new(matches.value_of("datadir").unwrap());
    let map_size: usize = matches
        .value_of("mapsize")
        .map_or(4096, |s| s.parse().unwrap())
        * (1024 * 1024 * 1024);
    let output_json = matches.is_present("json");
    let frame = matches.value_of("frame").unwrap();
    let root_keys: Vec<&str> = matches.values_of("ROOT").unwrap().collect();
    let env = lmdb::Environment::new()
        .set_map_size(map_size)
        .set_max_dbs(2)
        .open(datadir)
        .unwrap();
    let db = env.open_db(None).unwrap();
    let framesdb = env.open_db(Some("frames")).unwrap();

    let writer: Box<dyn Write> = match matches.value_of("out") {
        Some(path) if path != "-" => Box::new(fs::File::create(path).unwrap()),
        _ => Box::new(io::stdout()),
    };
    let mut out = io::BufWriter::new(writer);

    let txn1 = env.begin_ro_txn().unwrap();
    let frame = txn1.get(framesdb, &frame).to_owned().unwrap().to_vec();
    let mut inactive = txn1.reset();

    for root in root_keys {
        log::trace!("begin_txn");
        let begin_txn = Instant::now();
        let txn = inactive.renew().unwrap();

        log::trace!("begin_builder");
        // If we change Builder to use Vec::with_capacity we can get this to just 3 allocations.
        let mut builder = flexbuffers::Builder::default();
        log::trace!("end_builder");

        log::trace!("begin_embed");
        let begin_embed = Instant::now();
        embed::embed(&frame, &root, &mut builder, |key| {
            txn.get(db, &key).map_err(|e| {
                log::error!("in:load error:{:?} key:{} root:{} ", e, key, root);
                e
            })
        })?;
        let end_embed = Instant::now();
        log::trace!("end_embed");

        log::trace!("begin_output");
        let begin_output = Instant::now();
        let data = builder.view();
        if output_json {
            // This conversion triggers a huge number of allocations.
            // Could probably write directly to output.
            log::trace!("begin_from_vec");
            let value: serde_json::Value = flexbuffers::from_slice(data).unwrap();
            log::trace!("end_from_vec");
            log::trace!("begin_to_vec");
            let mut json_data = serde_json::to_vec(&value)?;
            log::trace!("end_to_vec");
            json_data.push('\n' as u8);
            out.write_all(&json_data)?;
        //serde_json::to_writer(out, &value)?;
        } else {
            out.write_all(data)?;
        }
        let end_output = Instant::now();
        log::trace!("end_output");
        inactive = txn.reset();
        let end_txn = Instant::now();
        log::trace!("end_txn");
        log::info!(
            "txn:{:?} embed:{:?} output:{:?} root:{}",
            end_txn.duration_since(begin_txn),
            end_embed.duration_since(begin_embed),
            end_output.duration_since(begin_output),
            root,
        );
    }
    #[cfg(feature = "logging_allocator")]
    ALLOC.disable_logging();
    Ok(())
}
