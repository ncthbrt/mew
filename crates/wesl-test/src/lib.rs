#![cfg_attr(not(test), allow(dead_code, unused_imports))]

use std::path::PathBuf;

use wesl_bundle::{file_system::PhysicalFilesystem, BundleContext, Bundler, BundlerError};

pub mod resolve;

#[test]
fn webgpu_samples() {
    let dir = std::fs::read_dir("webgpu-samples").expect("missing webgpu-samples");
    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path = entry.path();
        if path.extension().unwrap() == "wgsl" {
            println!("testing sample `{}`", path.display());
            let source = std::fs::read_to_string(path).expect("failed to read file");
            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");
            let disp = format!("{source_module}");
            let disp_module = wesl_parse::Parser::parse_str(&disp)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");
            assert_eq!(source_module, disp_module);
        }
    }
}

#[test]
fn wesl_samples() {
    let dir = std::fs::read_dir("wesl-samples").expect("missing wesl-samples");
    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());
            let source = std::fs::read_to_string(path).expect("failed to read file");
            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");
            let disp = format!("{source_module}");
            let disp_module = wesl_parse::Parser::parse_str(&disp)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");

            // println!("{:?}", source_module);
            assert_eq!(source_module, disp_module);
        }
    }
}

#[tokio::test]
async fn bundle_wesl_samples() -> Result<(), BundlerError<std::io::Error>> {
    let dir = std::fs::read_dir("wesl-samples").expect("missing wesl-samples");
    let mut entrypoints: Vec<PathBuf> = vec![];
    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            entrypoints.push(PathBuf::from("./").join(path.file_name().unwrap()));
        }
    }

    let bundler = Bundler {
        file_system: PhysicalFilesystem {
            entry_point: std::env::current_dir().unwrap().join("wesl-samples"),
        },
    };

    let result_with_root_module = bundler
        .bundle(&BundleContext {
            entry_points: entrypoints.clone(),
            enclosing_module_name: Some("MyLib".to_owned()),
        })
        .await?;

    let result_without_root_module = bundler
        .bundle(&BundleContext {
            entry_points: entrypoints,
            enclosing_module_name: None,
        })
        .await?;

    let expected_output_with_root_module_location: PathBuf = std::env::current_dir()
        .unwrap()
        .join("expected-bundle-outputs")
        .join("bundled-with-root-module.wesl");

    let expected_output_without_root_module_location: PathBuf = std::env::current_dir()
        .unwrap()
        .join("expected-bundle-outputs")
        .join("bundled-without-root-module.wesl");

    //// Uncomment to output results to expected outputs folder
    // let disp: String = format!("{result_with_root_module}");
    // let _ =
    //     std::fs::write(expected_output_with_root_module_location.clone(), disp).expect("Written");
    // let disp: String = format!("{result_without_root_module}");
    // let _ = std::fs::write(expected_output_without_root_module_location.clone(), disp)
    //     .expect("Written");

    let expected_output_module = wesl_parse::Parser::parse_str(
        &std::fs::read_to_string(expected_output_with_root_module_location.clone()).expect("READ"),
    )
    .inspect_err(|err| eprintln!("{err}"))
    .expect("parse error");
    assert_eq!(result_with_root_module, expected_output_module);

    let expected_output_module = wesl_parse::Parser::parse_str(
        &std::fs::read_to_string(expected_output_without_root_module_location.clone())
            .expect("READ"),
    )
    .inspect_err(|err| eprintln!("{err}"))
    .expect("parse error");
    assert_eq!(result_without_root_module, expected_output_module);

    Ok(())
}

#[test]
fn resolve_wesl_samples() -> Result<(), BundlerError<std::io::Error>> {
    let dir =
        std::fs::read_dir("expected-bundle-outputs").expect("missing expected-bundle-outputs");

    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());

            let resolver = wesl_resolver::Resolver {
                ..Default::default()
            };

            let source = std::fs::read_to_string(path.clone()).expect("failed to read file");
            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");

            let result = resolver.resolve(&source_module).unwrap();

            let expected_output_location: PathBuf = std::env::current_dir()
                .unwrap()
                .join("expected-mangle-outputs")
                .join(path.file_name().unwrap());

            // Uncomment to output results to expected outputs folder
            // let disp: String = format!("{result}");
            // let _ = std::fs::write(expected_output_location, disp).expect("Written");

            let expected_output_module = wesl_parse::Parser::parse_str(
                &std::fs::read_to_string(expected_output_location.clone()).expect("READ"),
            )
            .inspect_err(|err| eprintln!("{err}"))
            .expect("parse error");
            assert_eq!(result, expected_output_module);
        }
    }
    Ok(())
}
