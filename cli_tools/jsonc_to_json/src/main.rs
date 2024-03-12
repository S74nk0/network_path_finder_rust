use std::collections::{HashMap};
use std::path::PathBuf;
use std::string::String;
use serde_json::{Value};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Ok};
use std::io::{BufReader, BufWriter};
use std::fs::{File};
use clap::Parser;
use json_comments::StripComments;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input path
    #[clap(short, long, value_parser)]
    file: PathBuf,

    /// Otput path for saving transformation
    #[clap(short, long, value_parser)]
    output: Option<PathBuf>,
}

fn parse_jsonc(jsonc_file_path: PathBuf, jsonc_stripped_out_file_path: PathBuf) -> Result<Value> {
    let f = File::open(jsonc_file_path)?;
    let reader = BufReader::new(f);
    let stripped = StripComments::new(reader);
    let v: Value = serde_json::from_reader(stripped)?;
    let out_file = File::create(jsonc_stripped_out_file_path)?;
    let writer = BufWriter::new(out_file);
    serde_json::to_writer_pretty(writer, &v)?;
    Ok(v)
}

// fn strip_jsonc(jsonc_file_path: &str, json_vars: &str) -> Result<()> {

    

//     let maybe_jsonc_array: Value = parse_jsonc(jsonc_file_path)?;
//     let maybe_vars: Value = parse_jsonc(json_vars)?;
//     // println!("maybe_vars {}", maybe_vars);
//     // println!("");

//     let mut vars: HashMap<String, String> = HashMap::new();
//     if let Value::Object(jvars) = maybe_vars {
//         for (k,v) in jvars.into_iter() {
            
//             vars.insert(k, v.as_str().unwrap().to_owned());
//         }
//     }
//     let vars = vars;

//     if let Value::Array(elements) = maybe_jsonc_array {
//         for e in elements.into_iter() {
//             let e_obj = e.as_object().unwrap();
//             // sort by key insertion to be human friendly
//             let mut sorted_e: IndexMap<&str, &Value> = IndexMap::new();
//             let rest_keys: Vec<_> = e_obj.keys().filter(|p| p.ne(&"id") || p.ne(&"tag")).sorted().collect();
//             if let Some(v) = e_obj.get("id") {
//                 sorted_e.insert("id", v);
//             }
//             if let Some(v) = e_obj.get("tag") {
//                 sorted_e.insert("tag", v);
//             }
//             for k in rest_keys.iter() {
//                 sorted_e.insert(k, &e_obj.get(*k).unwrap());
//             }

//             // sorted keys
//             let mut e_str = serde_json::to_string(&sorted_e).unwrap();
//             for (k,v) in vars.iter() {
//                 // println!("{} {}", k, v);
//                 e_str = e_str.replace(k, v);
            
//             }
//             println!("{}", e_str);
//         }
//     }

//     Ok(())
// }

fn main() -> Result<()> {

    let args = Args::parse();

    let in_file = args.file;
    let mut out_file = in_file.clone();
    out_file.set_extension("stripped.json");

    parse_jsonc(in_file, out_file)?;

    Ok(())
}
