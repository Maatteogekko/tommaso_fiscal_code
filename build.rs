use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Location {
    country_code: String,
    country_name: String,
    city: Option<String>,
    state: Option<String>,
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("codegen.rs");
    let mut file = BufWriter::new(File::create(dest_path).unwrap());

    let input = File::open("codat.json").unwrap();
    let reader = BufReader::new(input);
    let data: HashMap<String, Location> = serde_json::from_reader(reader).unwrap();

    writeln!(
        &mut file,
        "\
#[derive(Debug)]
struct Location<'a> {{
    country_code: &'a str,
    country_name: &'a str,
    city: Option<&'a str>,
    state: Option<&'a str>,
}}"
    )
    .unwrap();

    let mut map = phf_codegen::Map::new();
    write!(
        &mut file,
        "static BIRTH_TOWNS: phf::Map<&'static str, &'static Location> = {}",
        {
            for (key, value) in data {
                map.entry(
                    key,
                    &format!(
                        "&Location {{
                    country_code: \"{}\",
                    country_name: \"{}\",
                    city: {},
                    state: {},
                }}",
                        value.country_code,
                        value.country_name,
                        match value.city {
                            Some(v) => format!("Some(\"{}\")", v),
                            None => "None".to_string(),
                        },
                        match value.state {
                            Some(v) => format!("Some(\"{}\")", v),
                            None => "None".to_string(),
                        },
                    ),
                );
            }
            map.build()
        }
    )
    .unwrap();
    writeln!(&mut file, ";").unwrap();

    println!("cargo:rerun-if-changed=codat.json");
}
