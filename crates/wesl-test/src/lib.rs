#![cfg_attr(not(test), allow(dead_code, unused_imports))]

use std::{collections::HashMap, fs, path::PathBuf};
use wesl_api::{ModuleDescriptor, Path, WeslError};
use wesl_bundle::Bundler;
use wesl_parse::syntax::TranslationUnit;
use wesl_types::{CompilerPass, CompilerPassError};

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

            assert_eq!(format!("{}", source_module), format!("{}", disp_module));
        }
    }
}

#[tokio::test]
async fn bundle_wesl_samples() -> Result<(), CompilerPassError> {
    let dir = std::fs::read_dir("wesl-samples").expect("missing wesl-samples");
    let mut entrypoints: Vec<String> = vec![];
    let mut dir_contents = dir.into_iter().collect::<Vec<_>>();

    dir_contents.sort_by_cached_key(|x| x.as_ref().unwrap().file_name());

    for entry in dir_contents {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            entrypoints.push(fs::read_to_string(path).unwrap());
        }
    }

    let mut bundler = Bundler {
        sources: entrypoints.iter().map(|x| x.as_str()).collect(),
        enclosing_module_name: Some("MyLib".to_owned()),
    };

    let translation_unit = TranslationUnit::default();

    let result_with_root_module = bundler.apply(&translation_unit)?;

    let mut bundler = Bundler {
        sources: entrypoints.iter().map(|x| x.as_str()).collect(),
        enclosing_module_name: None,
    };

    let result_without_root_module = bundler.apply(&translation_unit)?;

    let expected_output_with_root_module_location: PathBuf = std::env::current_dir()
        .unwrap()
        .join("expected-bundle-outputs")
        .join("bundled-with-root-module.wesl");

    let expected_output_without_root_module_location: PathBuf = std::env::current_dir()
        .unwrap()
        .join("expected-bundle-outputs")
        .join("bundled-without-root-module.wesl");

    #[cfg(feature = "update_expected_output")]
    {
        let disp: String = format!("{result_with_root_module}");
        let _ = std::fs::write(expected_output_with_root_module_location.clone(), disp)
            .expect("Written");
        let disp: String = format!("{result_without_root_module}");
        let _ = std::fs::write(expected_output_without_root_module_location.clone(), disp)
            .expect("Written");
    }

    let expected_output_module = wesl_parse::Parser::parse_str(
        &std::fs::read_to_string(expected_output_with_root_module_location.clone()).expect("READ"),
    )
    .inspect_err(|err| eprintln!("{err}"))
    .expect("parse error");
    assert_eq!(
        format!("{}", result_with_root_module),
        format!("{}", expected_output_module)
    );

    let expected_output_module = wesl_parse::Parser::parse_str(
        &std::fs::read_to_string(expected_output_without_root_module_location.clone())
            .expect("READ"),
    )
    .inspect_err(|err| eprintln!("{err}"))
    .expect("parse error");
    assert_eq!(
        format!("{}", result_without_root_module),
        format!("{}", expected_output_module)
    );

    Ok(())
}

#[test]
fn resolve_wesl_samples() -> Result<(), CompilerPassError> {
    let dir =
        std::fs::read_dir("expected-bundle-outputs").expect("missing expected-bundle-outputs");

    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());

            let mut resolver = wesl_resolve::Resolver;

            let source = std::fs::read_to_string(path.clone()).expect("failed to read file");
            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");

            let result = resolver.apply(&source_module).unwrap();

            let expected_output_location: PathBuf = std::env::current_dir()
                .unwrap()
                .join("expected-resolver-outputs")
                .join(path.file_name().unwrap());

            #[cfg(feature = "update_expected_output")]
            {
                let disp: String = format!("{result}");
                let _ = std::fs::write(expected_output_location.clone(), disp).expect("Written");
            }

            let expected_output_module = wesl_parse::Parser::parse_str(
                &std::fs::read_to_string(expected_output_location.clone()).expect("READ"),
            )
            .inspect_err(|err| eprintln!("{err}"))
            .expect("parse error");
            assert_eq!(format!("{}", result), format!("{}", expected_output_module));
        }
    }
    Ok(())
}

#[test]
fn mangle_wesl_samples() -> Result<(), CompilerPassError> {
    let dir =
        std::fs::read_dir("expected-resolver-outputs").expect("missing expected-resolver-outputs");

    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());

            let mut mangler = wesl_mangle::Mangler;

            let source = std::fs::read_to_string(path.clone()).expect("failed to read file");
            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");

            let result = mangler.apply(&source_module)?;

            let expected_output_location: PathBuf = std::env::current_dir()
                .unwrap()
                .join("expected-mangler-outputs")
                .join(path.file_name().unwrap());

            #[cfg(feature = "update_expected_output")]
            {
                let disp: String = format!("{result}");
                let _ = std::fs::write(expected_output_location.clone(), disp).expect("Written");
            }

            let expected_output_module = wesl_parse::Parser::parse_str(
                &std::fs::read_to_string(expected_output_location.clone()).expect("READ"),
            )
            .inspect_err(|err| eprintln!("{err}"))
            .expect("parse error");
            assert_eq!(format!("{}", result), format!("{}", expected_output_module));
        }
    }
    Ok(())
}

#[test]
fn flatten_wesl_samples() -> Result<(), CompilerPassError> {
    let dir =
        std::fs::read_dir("expected-mangler-outputs").expect("missing expected-mangler-outputs");

    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());

            let mut flattener = wesl_flatten::Flattener;

            let source = std::fs::read_to_string(path.clone()).expect("failed to read file");
            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");

            let result = flattener.apply(&source_module)?;

            let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
            let expected_output_location: PathBuf = std::env::current_dir()
                .unwrap()
                .join("expected-flattener-outputs")
                .join(format!("{}.wgsl", stem));

            #[cfg(feature = "update_expected_output")]
            {
                let disp: String = format!("{result}");
                let _ = std::fs::write(expected_output_location.clone(), disp).expect("Written");
            }

            let expected_output_module = wesl_parse::Parser::parse_str(
                &std::fs::read_to_string(expected_output_location.clone()).expect("READ"),
            )
            .inspect_err(|err| eprintln!("{err}"))
            .expect("parse error");
            assert_eq!(format!("{}", result), format!("{}", expected_output_module));
        }
    }
    Ok(())
}

#[test]
fn extend_wesl_samples() -> Result<(), CompilerPassError> {
    let dir = std::fs::read_dir("extend-inputs").expect("missing expected-test-inputs");

    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());

            let mut resolver = wesl_resolve::Resolver;

            let source = std::fs::read_to_string(path.clone()).expect("failed to read file");
            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");

            let result = resolver.apply(&source_module)?;

            let expected_output_location: PathBuf = std::env::current_dir()
                .unwrap()
                .join("expected-extend-outputs")
                .join(path.file_name().unwrap());

            #[cfg(feature = "update_expected_output")]
            {
                let disp: String = format!("{result}");
                let _ = std::fs::write(expected_output_location.clone(), disp).expect("Written");
            }

            let expected_output_module = wesl_parse::Parser::parse_str(
                &std::fs::read_to_string(expected_output_location.clone()).expect("READ"),
            )
            .inspect_err(|err| eprintln!("{err}"))
            .expect("parse error");
            assert_eq!(format!("{}", result), format!("{}", expected_output_module));
        }
    }
    Ok(())
}

#[test]
fn dealias_wesl_samples() -> Result<(), CompilerPassError> {
    let dir = std::fs::read_dir("dealias-inputs").expect("missing expected-test-inputs");

    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());

            let mut resolver = wesl_resolve::Resolver;

            let source = std::fs::read_to_string(path.clone()).expect("failed to read file");
            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");

            let mut result = resolver.apply(&source_module)?;

            let mut dealiaser = wesl_dealias::Dealiaser;

            dealiaser.apply_mut(&mut result)?;

            let expected_output_location: PathBuf = std::env::current_dir()
                .unwrap()
                .join("expected-dealias-outputs")
                .join(path.file_name().unwrap());

            #[cfg(feature = "update_expected_output")]
            {
                let disp: String = format!("{result}");
                let _ = std::fs::write(expected_output_location.clone(), disp).expect("Written");
            }

            let expected_output_module = wesl_parse::Parser::parse_str(
                &std::fs::read_to_string(expected_output_location.clone()).expect("READ"),
            )
            .inspect_err(|err| eprintln!("{err}"))
            .expect("parse error");
            assert_eq!(format!("{}", result), format!("{}", expected_output_module));
        }
    }
    Ok(())
}

#[test]
fn template_specialize_wesl_samples() -> Result<(), WeslError> {
    let dir =
        std::fs::read_dir("template-specialize-inputs").expect("missing expected-test-inputs");

    let entrypoints = HashMap::from([
        ("test_1", "test_1::main"),
        ("test_2", "test_2::main"),
        ("test_3", "test_3::main"),
        ("test_4", "test_4::main"),
        ("test_5", "test_5::main"),
    ]);

    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());

            let source = std::fs::read_to_string(path.clone()).expect("failed to read file");

            let mut api = wesl_api::WeslApi::default();

            let module_name = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .replace('-', "_");
            api.add_module(ModuleDescriptor {
                module_name: module_name.as_str(),
                source: wesl_api::Source::Text(&source),
            })?;

            let result = api.compile(&Path::Text(
                entrypoints.get(module_name.as_str()).unwrap().to_string(),
            ))?;

            let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
            let expected_output_location: PathBuf = std::env::current_dir()
                .unwrap()
                .join("expected-template-specialize-outputs")
                .join(format!("{}.wgsl", stem));

            #[cfg(feature = "update_expected_output")]
            {
                let disp: String = format!("{result}");
                let _ = std::fs::write(expected_output_location.clone(), disp).expect("Written");
            }

            let expected_output_module = wesl_parse::Parser::parse_str(
                &std::fs::read_to_string(expected_output_location.clone()).expect("READ"),
            )
            .inspect_err(|err| eprintln!("{err}"))
            .expect("parse error");
            assert_eq!(result, format!("{}", expected_output_module));
        }
    }
    Ok(())
}
