extern crate flexbuffers;
use flexbuffers::FlexBufferType::*;
use flexbuffers::{Builder, MapBuilder, Reader, VectorBuilder};

pub fn deepcopy<'de>(reader: &Reader<'de>, builder: &mut Builder) {
    let fxbt = reader.flexbuffer_type();
    match fxbt {
        Null => builder.build_singleton(()),
        Bool => builder.build_singleton(reader.as_bool()),
        Int | IndirectInt => builder.build_singleton(reader.as_i64()),
        UInt | IndirectUInt => builder.build_singleton(reader.as_u64()),
        Float | IndirectFloat => builder.build_singleton(reader.as_f64()),
        String | Key => builder.build_singleton(reader.as_str()),
        Blob => builder.build_singleton(reader.as_blob()),
        Map => {
            let mapreader = reader.as_map();
            let mut mapbuilder = builder.start_map();
            for (key, value) in mapreader.iter_keys().zip(mapreader.iter_values()) {
                deepcopy_map(&key, &value, &mut mapbuilder);
            }
            mapbuilder.end_map()
        }
        _ if fxbt.is_vector() => {
            let vecreader = reader.as_vector();
            let mut vecbuilder = builder.start_vector();
            for value in vecreader.iter() {
                deepcopy_vec(&value, &mut vecbuilder);
            }
            vecbuilder.end_vector()
        }
        _ => panic!("unsupported type"),
    }
}

pub fn deepcopy_vec<'de>(reader: &Reader<'de>, builder: &mut VectorBuilder) {
    let fxbt = reader.flexbuffer_type();
    match fxbt {
        Null => builder.push(()),
        Bool => builder.push(reader.as_bool()),
        Int | IndirectInt => builder.push(reader.as_i64()),
        UInt | IndirectUInt => builder.push(reader.as_u64()),
        Float | IndirectFloat => builder.push(reader.as_f64()),
        String | Key => builder.push(reader.as_str()),
        Blob => builder.push(reader.as_blob()),
        Map => {
            let mapreader = reader.as_map();
            let mut mapbuilder = builder.start_map();
            for (key, value) in mapreader.iter_keys().zip(mapreader.iter_values()) {
                deepcopy_map(&key, &value, &mut mapbuilder);
            }
            mapbuilder.end_map()
        }
        _ if fxbt.is_vector() => {
            let vecreader = reader.as_vector();
            let mut vecbuilder = builder.start_vector();
            for value in vecreader.iter() {
                deepcopy_vec(&value, &mut vecbuilder);
            }
            vecbuilder.end_vector()
        }
        _ => panic!("unsupported type"),
    }
}

pub fn deepcopy_map<'de>(key: &str, reader: &Reader<'de>, builder: &mut MapBuilder) {
    let fxbt = reader.flexbuffer_type();
    match fxbt {
        Null => builder.push(&key, ()),
        Bool => builder.push(&key, reader.as_bool()),
        Int | IndirectInt => builder.push(&key, reader.as_i64()),
        UInt | IndirectUInt => builder.push(&key, reader.as_u64()),
        Float | IndirectFloat => builder.push(&key, reader.as_f64()),
        String | Key => builder.push(&key, reader.as_str()),
        Blob => builder.push(&key, reader.as_blob()),
        Map => {
            let mapreader = reader.as_map();
            let mut mapbuilder = builder.start_map(&key);
            for (key, value) in mapreader.iter_keys().zip(mapreader.iter_values()) {
                deepcopy_map(&key, &value, &mut mapbuilder);
            }
            mapbuilder.end_map()
        }
        _ if fxbt.is_vector() => {
            let vecreader = reader.as_vector();
            let mut vecbuilder = builder.start_vector(&key);
            for value in vecreader.iter() {
                deepcopy_vec(&value, &mut vecbuilder);
            }
            vecbuilder.end_vector()
        }
        _ => panic!("unsupported type"),
    }
}
