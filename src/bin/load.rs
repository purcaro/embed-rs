extern crate clap;
extern crate flexbuffers;
extern crate lmdb;
extern crate serde_json;
use lmdb::Transaction;
use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let matches = clap::App::new("Load flexbuffers into LMDB")
        .arg_from_usage("-D. --datadir <DATADIR> 'LMDB database directory'")
        .arg_from_usage("-d, --dbname [DBNAME] 'LMDB database name'")
        .arg_from_usage("-i, --in [FILE] 'input json(lines) file'")
        .arg_from_usage("-k, --key [KEY] 'Fixed key to use'")
        .arg_from_usage("--mapsize [MB] 'LMDB map size in MB'")
        .arg(
            clap::Arg::from_usage("-a, --attr [ATTR] 'id attribute'")
                .required_unless("key")
                .conflicts_with("key"),
        )
        .get_matches();

    let datadir = Path::new(matches.value_of("datadir").unwrap());
    let dbname = matches.value_of("dbname");
    let map_size: usize = matches
        .value_of("mapsize")
        .map_or(4096, |s| s.parse().unwrap())
        * (1024 * 1024 * 1024);
    let key = matches.value_of("key");
    let attr = matches.value_of("attr");

    let reader: Box<dyn io::Read> = match matches.value_of("in") {
        Some(path) if path != "-" => Box::new(fs::File::open(path).unwrap()),
        _ => Box::new(io::stdin()),
    };
    let buffered = io::BufReader::new(reader);

    let deserializer = serde_json::Deserializer::from_reader(buffered);
    let iterator = deserializer.into_iter::<serde_json::Value>();

    fs::create_dir_all(datadir)?;
    let env = lmdb::Environment::new()
        .set_map_size(map_size)
        .set_max_dbs(2)
        .open(datadir)
        .unwrap();

    let db = env
        .open_db(dbname)
        .unwrap_or_else(|_| env.create_db(dbname, lmdb::DatabaseFlags::empty()).unwrap());
    let mut txn = env.begin_rw_txn().unwrap();

    for item in iterator {
        let value = item.unwrap();
        let id = key.unwrap_or_else(|| value[attr.unwrap()].as_str().unwrap());
        let buf = flexbuffers::to_vec(&value).unwrap();
        txn.put(db, &id, &buf, lmdb::WriteFlags::empty()).unwrap();
    }
    txn.commit().unwrap();
    Ok(())
}
