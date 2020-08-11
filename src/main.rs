mod couch_action;
mod models;

use clap::{load_yaml, App};
use couch_action::couch_export::CouchExport;
use couch_action::couch_import::CouchImport;
use couch_action::CouchAction;

fn main() {
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("./cli.yaml");
    let matches = App::from(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("import") {
        let file = matches.value_of("file").unwrap();
        let host = matches.value_of("host").unwrap();
        let user = matches.value_of("user").unwrap();
        let password = matches.value_of("password").unwrap();
        let database = matches.value_of("database").unwrap();
        let protocol = matches.value_of("protocol").unwrap_or("http");
        let create = matches.is_present("create");
        let import = CouchImport {
            host: host.to_string(),
            user: user.to_string(),
            password: password.to_string(),
            database: database.to_string(),
            protocol: protocol.to_string(),
            file: file.to_string(),
            create: create,
        };
        import.execute();
    }

    if let Some(matches) = matches.subcommand_matches("export") {
        let file = matches.value_of("file").unwrap();
        let host = matches.value_of("host").unwrap();
        let user = matches.value_of("user").unwrap();
        let password = matches.value_of("password").unwrap();
        let database = matches.value_of("database").unwrap();
        let protocol = matches.value_of("protocol").unwrap_or("http");
        let export = CouchExport {
            host: host.to_string(),
            user: user.to_string(),
            password: password.to_string(),
            database: database.to_string(),
            protocol: protocol.to_string(),
            file: file.to_string(),
        };
        export.execute();
    }
    // Same as previous examples...
}
