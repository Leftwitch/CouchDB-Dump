use clap::{load_yaml, App};

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, File};
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct AllDocs {
    total_rows: i64,
    offset: i64,
    rows: Vec<Row>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Row {
    id: String,
    key: String,
    doc: serde_json::value::Value,
}

fn main() {
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("./cli.yaml");
    let matches = App::from(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("import") {
        let host = matches.value_of("host").unwrap();
        let user = matches.value_of("user").unwrap();
        let password = matches.value_of("password").unwrap();
        let database = matches.value_of("database").unwrap();
        let protocol = matches.value_of("protocol").unwrap_or("http");

        let file = matches.value_of("file").unwrap();

        println!(
            "IMPORT - HOST: {} USER: {} PW: {} FILE: {} ",
            host, user, password, file
        );

        let file = fs::File::open(file).expect("file should open read only");
        let json: serde_json::Value =
            serde_json::from_reader(file).expect("file should be proper JSON");
        let docs = json
            .get("docs")
            .expect("Docs key not provided")
            .as_array()
            .expect("Docs should not be null");

        println!("Documents Read: {}", docs.len());

        let client = reqwest::Client::new();
        let url = format!("{}://{}/{}/_bulk_docs", protocol, host, database);
        let mut res = client
            .post(&url)
            .basic_auth(user, Some(password))
            .json(&json!({"new_edits":false, "docs": docs }))
            .send()
            .expect("The CouchDB returned an error");

        if res.status() != 201 {
            println!(
                "Request failed with status code: {}, response: {}",
                res.status(),
                res.text().unwrap()
            )
        }

        println!("Documents Inserted: {}", docs.len());
    }

    if let Some(matches) = matches.subcommand_matches("export") {
        let host = matches.value_of("host").unwrap();
        let user = matches.value_of("user").unwrap();
        let password = matches.value_of("password").unwrap();
        let file = matches.value_of("file").unwrap();
        let database = matches.value_of("database").unwrap();
        let protocol = matches.value_of("protocol").unwrap_or("http");

        println!(
            "EXPORT - HOST: {} USER: {} PW: {} FILE: {} ",
            host, user, password, file
        );

        let client = reqwest::Client::new();
        let url = format!(
            "{}://{}/{}/_all_docs?include_docs=true",
            protocol, host, database
        );
        let mut res = client
            .get(&url)
            .basic_auth(user, Some(password))
            .send()
            .expect("The CouchDB returned an error");
        if res.status() != 200 {
            println!(
                "Request failed with status code: {}, response: {}",
                res.status(),
                res.text().unwrap()
            )
        }

        let docs: AllDocs = res
            .json()
            .expect("The Response returned by the CouchDB is not valid JSON");

        let adjusted_docs = docs
            .rows
            .iter()
            .map(|row| &row.doc)
            .collect::<Vec<&serde_json::value::Value>>();
        let final_result = json!({ "docs": adjusted_docs });
        let json = serde_json::to_string(&final_result).expect("Json Conversion Failed");

        let mut file = File::create(file).expect("Output File could not be created");
        file.write_all(json.as_bytes())
            .expect("Couldnt write to Output File");

        println!("DOCS WRITTEN: {}", docs.total_rows);
    }
    // Same as previous examples...
}
