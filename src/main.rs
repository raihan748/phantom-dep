use clap::Parser;
use phantom_dep::cli::{CliArgs, EcosystemArg, OutputFormat};
use phantom_dep::model::Ecosystem;
use phantom_dep::reporter::{render_json, render_table, ScanReport};
use phantom_dep::scan_path;
use std::process;

fn main() {
    let args = CliArgs::parse();

    let eco_filter = match args.ecosystem {
        EcosystemArg::All => None,
        EcosystemArg::Npm => Some(Ecosystem::Npm),
        EcosystemArg::Pypi => Some(Ecosystem::PyPi),
        EcosystemArg::Cargo => Some(Ecosystem::Cargo),
        EcosystemArg::Go => Some(Ecosystem::Go),
    };

    let scan_results = match scan_path(&args.path, eco_filter, args.min_confidence) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Error scanning target: {}", err);
            process::exit(1);
        }
    };

    let report = ScanReport::new(&args.path, scan_results, args.strict);

    match args.format {
        OutputFormat::Table => {
            print!("{}", render_table(&report));
        }
        OutputFormat::Json => match render_json(&report) {
            Ok(json_str) => println!("{}", json_str),
            Err(e) => {
                eprintln!("Failed to serialize report to JSON: {}", e);
                process::exit(1);
            }
        },
    }

    if report.strict_triggered {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
