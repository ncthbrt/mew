//! The Command-line interface for the `wgsl-tools` suite.
//!
//! Very much a work in progress.

use clap::{Args, Parser, Subcommand};
use std::{fs, path::PathBuf};
use wesl_bundle::{file_system::PhysicalFilesystem, BundleContext, Bundler as WeslBundler};
use wesl_parse::Parser as WeslParser;

#[derive(Parser)]
#[command(version, author, about)]
#[command(propagate_version = true)]
struct Cli {
    /// main command
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// check correctness of the source file
    Check(CommonArgs),
    /// parse the source and convert it back to code from the syntax tree.
    Parse(CommonArgs),
    /// output the syntax tree to stdout
    Dump(CommonArgs),
    Bundle(BundleArgs),
}

#[derive(Args)]
struct CommonArgs {
    /// wgsl file entry-point
    input: PathBuf,
}

#[derive(Args)]
struct BundleArgs {
    /// optional module name to enclose output in
    #[arg(short, long)]
    module_name: Option<String>,
    /// wgsl file entry-point
    input: PathBuf,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let args = match &cli.command {
        Command::Check(args) => args,
        Command::Parse(args) => args,
        Command::Dump(args) => args,
        Command::Bundle(args) => &CommonArgs {
            input: args.input.clone(),
        },
    };

    let source = fs::read_to_string(&args.input).expect("could not open input file");

    match &cli.command {
        Command::Check(_) => {
            print!("{} -- ", args.input.display());
            match WeslParser::parse_str(&source) {
                Ok(_) => println!("OK"),
                Err(err) => eprintln!("{err}"),
            };
        }
        Command::Parse(_) => {
            match WeslParser::parse_str(&source) {
                Ok(ast) => {
                    println!("{ast}")
                }
                Err(err) => eprintln!("{err}"),
            };
        }
        Command::Dump(_) => {
            match WeslParser::parse_str(&source) {
                Ok(ast) => println!("{ast:?}"),
                Err(err) => eprintln!("{err}"),
            };
        }
        Command::Bundle(bundle_args) => {
            let bundler = WeslBundler {
                file_system: PhysicalFilesystem {
                    entry_point: args.input.parent().unwrap().to_path_buf(),
                },
            };
            match bundler
                .bundle(&BundleContext {
                    entry_points: vec![PathBuf::from(bundle_args.input.file_name().unwrap())],
                    enclosing_module_name: bundle_args.module_name.clone(),
                })
                .await
            {
                Ok(ast) => println!("\n{ast}"),
                Err(err) => eprintln!("{err:?}"),
            };
        }
    }
}
