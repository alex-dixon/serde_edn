extern crate serde_edn;
extern crate serde_json;
extern crate criterion;
extern crate core;

use criterion::*;
use serde_edn::{from_str, from_reader, Value, from_slice,Serializer};
use std::fs::File;
use std::io::{BufReader, Read};
use std::str;
use std::path::Path;
use std::string::ToString;

const CANADA_PATH: &'static str = "benches/canada.edn";
const CANADA_JSON_PATH: &'static str = "benches/canada.json";
const TWITTER_JSON_PATH: &'static str = "benches/twitter.json";
const TWITTER_PATH: &'static str = "benches/twitter.edn";
const TWITTER_KW_PATH: &'static str = "benches/twitter-kw.edn";

fn deserialize_slice_from_file(c: &mut Criterion, filepath: &str) {
    let path = Path::new(filepath);
    let filename = path.file_name().unwrap().to_str().unwrap();
    let mut f = File::open(path).unwrap();
    let mut bytes = vec![];
    f.read_to_end(&mut bytes).unwrap();

    c.bench(
        filename,
        ParameterizedBenchmark::new(
            "deserialize",
            |b, elems| b.iter(|| {
                let v: Value = from_slice(elems).unwrap();
                v
            }),
            vec![bytes],
        ).throughput(|elems| Throughput::Elements(elems.len() as u32)),
    );
}

fn serde_json_deserialize_slice_from_file(c: &mut Criterion, filepath: &str) {
    let path = Path::new(filepath);
    let filename = path.file_name().unwrap().to_str().unwrap();
    let mut f = File::open(path).unwrap();
    let mut bytes = vec![];
    f.read_to_end(&mut bytes).unwrap();

    c.bench(
        filename,
        ParameterizedBenchmark::new(
            "deserialize",
            |b, elems| b.iter(|| {
                let v: serde_json::Value = serde_json::from_slice(elems).unwrap();
                v
            }),
            vec![bytes],
        ).throughput(|elems| Throughput::Elements(elems.len() as u32)),
    );
}

fn serialize_slice_from_file(c: &mut Criterion, filepath: &str) {
    let path = Path::new(filepath);
    let filename = path.file_name().unwrap().to_str().unwrap();
    let mut f = File::open(path).unwrap();
    let mut bytes = vec![];
    f.read_to_end(&mut bytes).unwrap();
    let values: Value = from_slice(&bytes).unwrap();
    c.bench(
        filename,
        ParameterizedBenchmark::new(
            "serialize",
            |b, elems| b.iter(|| {
                let s =  Value::to_string(&elems);
                s
            }),
            vec![values],
        ).throughput(move|elems| Throughput::Elements(bytes.len() as u32)),
    );
}

fn serde_json_serialize_slice_from_file(c: &mut Criterion, filepath: &str) {
    let path = Path::new(filepath);
    let filename = path.file_name().unwrap().to_str().unwrap();
    let mut f = File::open(path).unwrap();
    let mut bytes = vec![];
    f.read_to_end(&mut bytes).unwrap();
    let values: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    c.bench(
        filename,
        ParameterizedBenchmark::new(
            "serialize",
            |b, elems| b.iter(|| {
                let s =  serde_json::Value::to_string(&elems);
                s
            }),
            vec![values],
        ).throughput(move|elems| Throughput::Elements(bytes.len() as u32)),
    );
}

fn bench(c: &mut Criterion) {
    deserialize_slice_from_file(c, CANADA_PATH);
    serde_json_deserialize_slice_from_file(c, CANADA_JSON_PATH);
    deserialize_slice_from_file(c, TWITTER_PATH);
    deserialize_slice_from_file(c, TWITTER_KW_PATH);
    serde_json_deserialize_slice_from_file(c, TWITTER_JSON_PATH);
    serialize_slice_from_file(c, TWITTER_PATH);
    serde_json_serialize_slice_from_file(c, TWITTER_JSON_PATH);
}

criterion_group!(benches, bench);
criterion_main!(benches);
