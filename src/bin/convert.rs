extern crate flexbuffers;
extern crate serde_json;
use std::env;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => {
            let format_in = &args[1];
            let format_out = &args[2];
            let mut buf_in = Vec::new();
            io::stdin().lock().read_to_end(&mut buf_in)?;

            let value: serde_json::Value = match &format_in[..] {
                "json" => serde_json::from_slice(&buf_in)?,
                "flexbuffer" => flexbuffers::from_slice(&buf_in).unwrap(),
                _ => panic!("invalid arg format_in"),
            };

            let buf_out = match &format_out[..] {
                "json" => serde_json::to_vec(&value)?,
                "flexbuffer" => flexbuffers::to_vec(&value).unwrap(),
                _ => panic!("invalid arg format_out"),
            };

            io::stdout().lock().write_all(&buf_out)?;
            Ok(())
        }
        _ => {
            eprintln!(
                "usage: {} <format_in> <format_out>\nformats: json, flexbuffer",
                args[0]
            );
            panic!("invalid args");
        }
    }
}
