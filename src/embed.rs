extern crate flexbuffers;
use crate::deepcopy::*;
use flexbuffers::{Builder, FlexBufferType, MapBuilder, Reader, VectorBuilder};

pub fn embed<'b, F, E>(frame: &[u8], root: &str, builder: &mut Builder, load: F) -> Result<(), E>
where
    E: std::error::Error,
    F: Fn(&str) -> Result<&'b [u8], E>,
{
    let frame_reader = Reader::get_root(&frame).unwrap();
    let root_buf = load(&root)?;
    let reader = &Reader::get_root(&root_buf).unwrap();

    let fxbt = frame_reader.flexbuffer_type();
    match fxbt {
        FlexBufferType::Null => deepcopy(reader, builder),
        FlexBufferType::Map => {
            let tmpreader;
            let derefreader = if reader.flexbuffer_type().is_string() {
                let key = reader.as_str();
                let b = load(&key)?;
                tmpreader = Reader::get_root(&b).unwrap();
                &tmpreader
            } else {
                reader
            };
            if !derefreader.flexbuffer_type().is_map() {
                return Ok(());
            }
            let mapreader = derefreader.as_map();
            let mapframe = frame_reader.as_map();
            let mut mapbuilder = builder.start_map();
            // TODO: with two arrays of sorted keys sometimes it will be faster to iterate than binary search per key.
            // Worst case for binary search: len(smaller) + (len(smaller) * log2(len(larger)))
            // Worst case for simultaneous iteration: len(smaller) + len(larger)
            // Binary search is O(log2(N)) so faster to iterate < log2(N) keys to neighbour?
            for (k, child) in mapframe.iter_keys().zip(mapframe.iter_values()) {
                match mapreader.index_key(k) {
                    Some(idx) => {
                        embed_map(k, &child, &mapreader.idx(idx), &mut mapbuilder, &load)?;
                    }
                    None => {}
                }
            }
            mapbuilder.end_map()
        }
        _ if fxbt.is_vector() => {
            if !reader.flexbuffer_type().is_vector() {
                return Ok(());
            }
            let child = frame_reader.as_vector().idx(0);
            let mut vecbuilder = builder.start_vector();
            let vecreader = reader.as_vector();
            for value in vecreader.iter() {
                embed_vec(&child, &value, &mut vecbuilder, &load)?;
            }
            vecbuilder.end_vector()
        }
        _ => panic!("unsupported type"),
    }
    Ok(())
}

// Avoid type variable for load Fn below to prevent overflow evaluating requirements.
// https://stackoverflow.com/a/31197781
fn embed_vec<'a, 'b, E>(
    frame_reader: &Reader<'a>,
    reader: &Reader<'a>,
    builder: &mut VectorBuilder,
    load: &dyn Fn(&str) -> Result<&'b [u8], E>,
) -> Result<(), E>
where
    E: std::error::Error,
{
    let fxbt = frame_reader.flexbuffer_type();
    match fxbt {
        FlexBufferType::Null => deepcopy_vec(reader, builder),
        FlexBufferType::Map => {
            let tmpreader;
            let derefreader = if reader.flexbuffer_type().is_string() {
                let key = reader.as_str();
                let b = load(&key)?;
                tmpreader = Reader::get_root(&b).unwrap();
                &tmpreader
            } else {
                reader
            };
            if !derefreader.flexbuffer_type().is_map() {
                return Ok(());
            }
            let mapreader = derefreader.as_map();
            let mapframe = frame_reader.as_map();
            let mut mapbuilder = builder.start_map();
            for (k, child) in mapframe.iter_keys().zip(mapframe.iter_values()) {
                match mapreader.index_key(k) {
                    Some(idx) => {
                        embed_map(k, &child, &mapreader.idx(idx), &mut mapbuilder, &load)?;
                    }
                    None => {}
                }
            }
            mapbuilder.end_map()
        }
        _ if fxbt.is_vector() => {
            if !reader.flexbuffer_type().is_vector() {
                return Ok(());
            }
            let child = frame_reader.as_vector().idx(0);
            let mut vecbuilder = builder.start_vector();
            let vecreader = reader.as_vector();
            for value in vecreader.iter() {
                embed_vec(&child, &value, &mut vecbuilder, &load)?;
            }
            vecbuilder.end_vector()
        }
        _ => panic!("unsupported type"),
    };
    Ok(())
}

fn embed_map<'a, 'b, E>(
    key: &str,
    frame_reader: &Reader<'a>,
    reader: &Reader<'a>,
    builder: &mut MapBuilder,
    load: &dyn Fn(&str) -> Result<&'b [u8], E>,
) -> Result<(), E>
where
    E: std::error::Error,
{
    let fxbt = frame_reader.flexbuffer_type();
    match fxbt {
        FlexBufferType::Null => deepcopy_map(key, reader, builder),
        FlexBufferType::Map => {
            let tmpreader;
            let derefreader = if reader.flexbuffer_type().is_string() {
                let key = reader.as_str();
                let b: &[u8] = load(&key)?;
                tmpreader = Reader::get_root(&b).unwrap();
                &tmpreader
            } else {
                reader
            };
            if !derefreader.flexbuffer_type().is_map() {
                return Ok(());
            }
            let mapreader = derefreader.as_map();
            let mapframe = frame_reader.as_map();
            let mut mapbuilder = builder.start_map(key);
            for (k, child) in mapframe.iter_keys().zip(mapframe.iter_values()) {
                match mapreader.index_key(k) {
                    Some(idx) => {
                        embed_map(k, &child, &mapreader.idx(idx), &mut mapbuilder, &load)?;
                    }
                    None => {}
                }
            }
            mapbuilder.end_map()
        }
        _ if fxbt.is_vector() => {
            if !reader.flexbuffer_type().is_vector() {
                return Ok(());
            }
            let child = frame_reader.as_vector().idx(0);
            let mut vecbuilder = builder.start_vector(key);
            let vecreader = reader.as_vector();
            for value in vecreader.iter() {
                embed_vec(&child, &value, &mut vecbuilder, &load)?;
            }
            vecbuilder.end_vector()
        }
        _ => panic!("unsupported type"),
    };
    Ok(())
}
