mod couch_action;
mod models;
mod progress_style;

use clap::{load_yaml, App, ArgMatches};
use couch_action::couch_export::CouchExport;
use couch_action::couch_import::CouchImport;
use couch_action::CouchAction;

fn main() {
    let yaml = load_yaml!("./cli.yaml");
    let matches = App::from(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("import") {
        let (protocol, host, port, user, password, database, file) = get_base_arguments(matches);
        let create = matches.is_present("create");

        let import = CouchImport {
            host: host.to_string(),
            user: user.to_string(),
            password: password.to_string(),
            database: database.to_string(),
            protocol: protocol.to_string(),
            port: port.to_string(),
            file: file.to_string(),
            create: create,
        };
        import.execute();
    }

    if let Some(matches) = matches.subcommand_matches("export") {
        let (protocol, host, port, user, password, database, file) = get_base_arguments(matches);

        let export = CouchExport {
            host: host.to_string(),
            user: user.to_string(),
            password: password.to_string(),
            database: database.to_string(),
            protocol: protocol.to_string(),
            port: port.to_string(),
            file: file.to_string(),
        };
        export.execute();
    }
    // Same as previous examples...
}

fn get_base_arguments(matches: &ArgMatches) -> (&str, &str, &str, &str, &str, &str, &str) {
    let file = matches.value_of("file").unwrap();
    let host = matches.value_of("host").unwrap();
    let user = matches.value_of("user").unwrap();
    let password = matches.value_of("password").unwrap();
    let database = matches.value_of("database").unwrap();
    let port = matches.value_of("port").unwrap_or("5984");
    let mut protocol = matches.value_of("protocol").unwrap_or("http");
    if port == "443" {
        protocol = "https";
    }
    (protocol, host, port, user, password, database, file)
}
