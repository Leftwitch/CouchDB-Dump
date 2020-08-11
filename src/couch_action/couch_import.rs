use super::CouchAction;
use clap::ArgMatches;
use serde_json::json;
use std::fs;
pub struct CouchImport;

impl CouchAction for CouchImport {
    fn execute(matches: &ArgMatches) {
        let file = matches.value_of("file").unwrap();
        let host = matches.value_of("host").unwrap();
        let user = matches.value_of("user").unwrap();
        let password = matches.value_of("password").unwrap();
        let database = matches.value_of("database").unwrap();
        let protocol = matches.value_of("protocol").unwrap_or("http");

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
}
