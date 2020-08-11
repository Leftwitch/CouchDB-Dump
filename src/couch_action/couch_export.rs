use super::CouchAction;
use crate::models::*;
use clap::ArgMatches;
use serde_json::json;
use std::{fs::File, io::Write};

pub struct CouchExport;

impl CouchAction for CouchExport {
    fn execute(matches: &ArgMatches) {
        let file = matches.value_of("file").unwrap();
        let host = matches.value_of("host").unwrap();
        let user = matches.value_of("user").unwrap();
        let password = matches.value_of("password").unwrap();
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
}
