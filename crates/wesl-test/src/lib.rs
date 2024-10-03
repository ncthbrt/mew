#![cfg_attr(not(test), allow(dead_code, unused_imports))]

use std::path::PathBuf;
use wesl_bundle::{file_system::PhysicalFilesystem, BundleContext, Bundler, BundlerError};
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

            assert_eq!(source_module, disp_module);
        }
    }
}

#[tokio::test]
async fn bundle_wesl_samples() -> Result<(), BundlerError<std::io::Error>> {
    let dir = std::fs::read_dir("wesl-samples").expect("missing wesl-samples");
    let mut entrypoints: Vec<PathBuf> = vec![];
    let mut dir_contents = dir.into_iter().collect::<Vec<_>>();

    dir_contents.sort_by_cached_key(|x| x.as_ref().unwrap().file_name());

    for entry in dir_contents {
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
fn resolve_wesl_samples() -> Result<(), BundlerError<std::io::Error>> {
    let dir =
        std::fs::read_dir("expected-bundle-outputs").expect("missing expected-bundle-outputs");

    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());

            let mut resolver = wesl_resolve::Resolver {
                ..Default::default()
            };

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

            let mut mangler = wesl_mangle::Mangler {
                ..Default::default()
            };

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

            let mut flattener = wesl_flatten::Flattener {
                ..Default::default()
            };

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

            let mut resolver = wesl_resolve::Resolver {
                ..Default::default()
            };

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

            let mut resolver = wesl_resolve::Resolver {
                ..Default::default()
            };

            let source = std::fs::read_to_string(path.clone()).expect("failed to read file");
            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");

            let mut result = resolver.apply(&source_module)?;

            let mut dealiaser = wesl_dealias::Dealiaser {
                ..Default::default()
            };

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
fn template_specialize_wesl_samples() -> Result<(), CompilerPassError> {
    let dir =
        std::fs::read_dir("template-specialize-inputs").expect("missing expected-test-inputs");

    for entry in dir {
        let entry = entry.expect("error reading entry");
        let path: std::path::PathBuf = entry.path();
        if path.extension().unwrap() == "wgsl" || path.extension().unwrap() == "wesl" {
            println!("testing sample `{}`", path.display());

            let source = std::fs::read_to_string(path.clone()).expect("failed to read file");

            let source_module = wesl_parse::Parser::parse_str(&source)
                .inspect_err(|err| eprintln!("{err}"))
                .expect("parse error");

            let mut resolver = wesl_resolve::Resolver::default();
            let mut result = resolver.apply(&source_module)?;

            let mut normalizer = wesl_template_normalize::TemplateNormalizer {
                ..Default::default()
            };

            normalizer.apply_mut(&mut result)?;

            let mut specializer = wesl_specialize::Specializer::default();

            specializer.apply_mut(&mut result)?;

            let mut dealiaser = wesl_dealias::Dealiaser {
                ..Default::default()
            };

            dealiaser.apply_mut(&mut result)?;

            let mut mangler = wesl_mangle::Mangler {
                ..Default::default()
            };

            mangler.apply_mut(&mut result)?;

            let mut flattener = wesl_flatten::Flattener::default();
            flattener.apply_mut(&mut result)?;

            let expected_output_location: PathBuf = std::env::current_dir()
                .unwrap()
                .join("expected-template-specialize-outputs")
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
