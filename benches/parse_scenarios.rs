use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn argv_cases() -> Vec<Vec<&'static str>> {
    vec![
        vec!["--verbose", "--output", "bin/a", "-j10", "file1"],
        vec!["-v", "-v", "-o", "bin/a", "--jobs", "-1", "file1", "file2"],
        vec!["-v", "--output=bin/a", "--", "-x", "--y"],
    ]
}

// ---------------------- rust-args-parser ----------------------
fn rap_parse_only(c: &mut Criterion) {
    use rust_args_parser::{dispatch_to, CmdSpec, Env, OptSpec, PosSpec};
    #[derive(Default)]
    struct U {
        v: usize,
        o: Option<String>,
        j: Option<String>,
        files: Vec<String>,
    }
    let env = Env::new("prog").wrap_cols(0).color(false);
    let cmd = CmdSpec::new(
        None,
        Some(|args, u: &mut U| {
            u.files = args.iter().map(|s| (*s).to_string()).collect();
            Ok(())
        }),
    )
    .opts([
        OptSpec::new("verbose", |v, _| {
            assert!(v.is_none());
            Ok(())
        })
        .short('v')
        .flag(),
        OptSpec::new("output", |v, u: &mut U| {
            u.o = v.map(std::string::ToString::to_string);
            Ok(())
        })
        .short('o')
        .required()
        .metavar("FILE"),
        OptSpec::new("jobs", |v, u: &mut U| {
            u.j = v.map(std::string::ToString::to_string);
            Ok(())
        })
        .short('j')
        .optional()
        .numeric()
        .metavar("N"),
    ])
    .pos([PosSpec::new("FILE").range(0, 3)]);

    c.bench_function("rust-args-parser_parse_only", |b| {
        b.iter(|| {
            for argv in argv_cases() {
                let mut u = U::default();
                let mut sink = std::io::sink();
                let _ = dispatch_to(&env, &cmd, &argv, &mut u, &mut sink);
                black_box((&u.v, u.o.as_deref(), u.j.as_deref(), u.files.len()));
            }
        });
    });
}

fn rap_build_and_parse(c: &mut Criterion) {
    use rust_args_parser::{dispatch_to, CmdSpec, Env, OptSpec, PosSpec};
    #[derive(Default)]
    struct U {
        v: usize,
        o: Option<String>,
        j: Option<String>,
        files: Vec<String>,
    }
    c.bench_function("rust-args-parser_build_and_parse", |b| {
        b.iter(|| {
            for argv in argv_cases() {
                let env = Env::new("prog").wrap_cols(0).color(false);
                let cmd = CmdSpec::new(
                    None,
                    Some(|args, u: &mut U| {
                        u.files = args.iter().map(|s| (*s).to_string()).collect();
                        Ok(())
                    }),
                )
                .opts([
                    OptSpec::new("verbose", |v, _| {
                        assert!(v.is_none());
                        Ok(())
                    })
                    .short('v')
                    .flag(),
                    OptSpec::new("output", |v, u: &mut U| {
                        u.o = v.map(std::string::ToString::to_string);
                        Ok(())
                    })
                    .short('o')
                    .required()
                    .metavar("FILE"),
                    OptSpec::new("jobs", |v, u: &mut U| {
                        u.j = v.map(std::string::ToString::to_string);
                        Ok(())
                    })
                    .short('j')
                    .optional()
                    .numeric()
                    .metavar("N"),
                ])
                .pos([PosSpec::new("FILE").range(0, 3)]);
                let mut u = U::default();
                let mut sink = std::io::sink();
                let _ = dispatch_to(&env, &cmd, &argv, &mut u, &mut sink);
                black_box((&u.v, u.o.as_deref(), u.j.as_deref(), u.files.len()));
            }
        });
    });
}

// ---------------------- pico-args ----------------------
fn pico_parse_only(c: &mut Criterion) {
    use pico_args::Arguments;
    use std::ffi::OsString;
    c.bench_function("pico-args_parse_only", |b| {
        b.iter(|| {
            for argv in argv_cases() {
                let vec: Vec<OsString> = argv.iter().copied().map(OsString::from).collect();
                let mut parg = Arguments::from_vec(vec);
                let mut v = 0usize;
                while parg.contains("-v") {
                    v += 1;
                }
                let o: String = parg.value_from_str(["-o", "--output"]).unwrap_or_default();
                let j: Option<String> = parg.opt_value_from_str(["-j", "--jobs"]).ok().flatten();
                black_box((v, o, j, parg.finish().len()));
            }
        });
    });
}

// pico-args has no builder cost; reuse parse_only

// ---------------------- lexopt ----------------------
fn lexopt_parse_only(crit: &mut Criterion) {
    use lexopt::{Parser, ValueExt};
    crit.bench_function("lexopt_parse_only", |b| {
        b.iter(|| {
            for argv in argv_cases() {
                let mut parser = Parser::from_args(argv);
                let (mut v, mut o, mut j) = (0usize, None::<String>, None::<String>);
                let mut files: Vec<String> = Vec::new();
                loop {
                    use lexopt::Arg::{Long, Short, Value};
                    match parser.next() {
                        Ok(Some(Short('v'))) => {
                            v += 1;
                        }
                        Ok(Some(Short('o') | Long("output"))) => {
                            o = Some(parser.value().unwrap().string().unwrap());
                        }
                        Ok(Some(Short('j') | Long("jobs"))) => {
                            if let Ok(val) = parser.value() {
                                j = Some(val.string().unwrap());
                            }
                        }
                        Ok(Some(Value(x))) => files.push(x.to_string_lossy().into_owned()),
                        Ok(None) | Err(_) => break,
                        _ => {}
                    }
                }
                black_box((v, o, j, files.len()));
            }
        });
    });
}

// ---------------------- clap (builder) ----------------------
fn clap_parse_only(c: &mut Criterion) {
    use clap::{Arg, Command};
    let cmd = Command::new("prog")
        .disable_help_flag(false)
        .arg(Arg::new("verbose").short('v').action(clap::ArgAction::Count))
        .arg(Arg::new("output").short('o').long("output").required(true).num_args(1))
        .arg(Arg::new("jobs").short('j').long("jobs").num_args(0..=1))
        .arg(Arg::new("FILE").num_args(0..=3));

    c.bench_function("clap_parse_only", |b| {
        b.iter(|| {
            for argv in argv_cases() {
                let _m = cmd
                    .clone()
                    .try_get_matches_from(std::iter::once("prog").chain(argv.iter().copied()))
                    .ok();
            }
        });
    });
}

fn clap_build_and_parse(c: &mut Criterion) {
    use clap::{Arg, Command};
    c.bench_function("clap_build_and_parse", |b| {
        b.iter(|| {
            for argv in argv_cases() {
                let cmd = Command::new("prog")
                    .disable_help_flag(false)
                    .arg(Arg::new("verbose").short('v').action(clap::ArgAction::Count))
                    .arg(Arg::new("output").short('o').long("output").required(true).num_args(1))
                    .arg(Arg::new("jobs").short('j').long("jobs").num_args(0..=1))
                    .arg(Arg::new("FILE").num_args(0..=3));
                let _m = cmd
                    .try_get_matches_from(std::iter::once("prog").chain(argv.iter().copied()))
                    .ok();
            }
        });
    });
}

criterion_group!(
    benches,
    rap_parse_only,
    rap_build_and_parse,
    pico_parse_only,
    lexopt_parse_only,
    clap_parse_only,
    clap_build_and_parse,
);
criterion_main!(benches);
