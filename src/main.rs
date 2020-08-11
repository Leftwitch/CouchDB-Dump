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
        CouchImport::execute(matches);
    }

    if let Some(matches) = matches.subcommand_matches("export") {
        CouchExport::execute(matches);
    }
    // Same as previous examples...
}
