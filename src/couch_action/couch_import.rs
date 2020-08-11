use super::CouchAction;
use clap::ArgMatches;
use serde_json::json;
use std::fs;
pub struct CouchImport {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub protocol: String,
    pub file: String,
}

impl CouchAction for CouchImport {
    fn execute(&self) {
        println!(
            "IMPORT - HOST: {} USER: {} PW: {} FILE: {} ",
            self.host, self.user, self.password, self.file
        );

        let file = fs::File::open(&self.file).expect("file should open read only");
        let json: serde_json::Value =
            serde_json::from_reader(file).expect("file should be proper JSON");
        let docs = json
            .get("docs")
            .expect("Docs key not provided")
            .as_array()
            .expect("Docs should not be null");

        println!("Documents Read: {}", docs.len());

        let client = reqwest::Client::new();
        let url = format!(
            "{}://{}/{}/_bulk_docs",
            self.protocol, self.host, self.database
        );
        let mut res = client
            .post(&url)
            .basic_auth(&self.user, Some(&self.password))
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