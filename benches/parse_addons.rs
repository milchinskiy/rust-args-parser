use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn argv_error_cases() -> Vec<Vec<&'static str>> {
    vec![
        vec!["-o"],         // missing value for required
        vec!["--x", "--y"], // XOR conflict
    ]
}

fn argv_heavy_cases() -> Vec<Vec<&'static str>> {
    vec![
        vec![
            "-v",
            "-v",
            "--output",
            "bin/a",
            "-j10",
            "--verbose",
            "-v",
            "--jobs",
            "-1",
            "file1",
            "file2",
            "file3",
        ],
        vec!["--verbose", "--output=bin/b", "--jobs=1e-3", "--", "-x", "--y", "z"],
    ]
}

// ---------------------- rust-args-parser ----------------------
fn rap_schema<'a, U>() -> (rust_args_parser::Env<'a>, rust_args_parser::CmdSpec<'a, U>) {
    use rust_args_parser::{CmdSpec, Env, OptSpec};
    let env = Env::new("prog").wrap_cols(0).color(false);
    let cmd = CmdSpec::new(None, None)
        .opts([
            OptSpec::new("verbose", |v, _| {
                assert!(v.is_none());
                Ok(())
            })
            .short('v')
            .flag(),
            OptSpec::new("output", |_, _| Ok(())).short('o').required().metavar("FILE"),
            OptSpec::new("jobs", |_, _| Ok(())).short('j').optional().numeric().metavar("N"),
            // XOR group for conflict case
            OptSpec::new("x", |_, _| Ok(())).at_most_one(7).flag(),
            OptSpec::new("y", |_, _| Ok(())).at_most_one(7).flag(),
        ])
        .pos([rust_args_parser::PosSpec::new("FILE").range(0, 3)]);
    (env, cmd)
}

fn rap_errors(c: &mut Criterion) {
    use rust_args_parser::dispatch_to;
    let (env, cmd) = rap_schema::<()>();
    c.bench_function("rust-args-parser_errors", |b| {
        b.iter(|| {
            for argv in argv_error_cases() {
                let mut sink = std::io::sink();
                let _ = dispatch_to(&env, &cmd, &argv, &mut (), &mut sink);
            }
        });
    });
}

fn rap_heavy(c: &mut Criterion) {
    use rust_args_parser::dispatch_to;
    let (env, cmd) = rap_schema::<()>();
    c.bench_function("rust-args-parser_heavy", |b| {
        b.iter(|| {
            for argv in argv_heavy_cases() {
                let mut sink = std::io::sink();
                let _ = dispatch_to(&env, &cmd, &argv, &mut (), &mut sink);
            }
        });
    });
}

// ---------------------- pico-args ----------------------
fn pico_errors(c: &mut Criterion) {
    use pico_args::Arguments;
    use std::ffi::OsString;
    c.bench_function("pico-args_errors", |b| {
        b.iter(|| {
            for argv in argv_error_cases() {
                let vec: Vec<OsString> = argv.iter().copied().map(OsString::from).collect();
                let mut parg = Arguments::from_vec(vec);

                // missing value error for -o/--output
                let _o: Result<String, _> = parg.value_from_str(["-o", "--output"]);

                // XOR conflict: check presence of both --x and --y
                let has_x = parg.contains("--x");
                let has_y = parg.contains("--y");
                let xor_conflict = has_x && has_y;
                black_box(xor_conflict);

                // consume rest
                let _ = parg.finish();
            }
        });
    });
}

fn pico_heavy(c: &mut Criterion) {
    use pico_args::Arguments;
    use std::ffi::OsString;
    c.bench_function("pico-args_heavy", |b| {
        b.iter(|| {
            for argv in argv_heavy_cases() {
                let vec: Vec<OsString> = argv.iter().copied().map(OsString::from).collect();
                let mut parg = Arguments::from_vec(vec);
                while parg.contains("-v") {}
                let _o: String = parg.value_from_str(["-o", "--output"]).unwrap_or_default();
                let _j: Option<String> = parg.opt_value_from_str(["-j", "--jobs"]).ok().flatten();
                let _ = parg.finish();
            }
        });
    });
}

// ---------------------- lexopt ----------------------
fn lexopt_errors(c: &mut Criterion) {
    use lexopt::Parser;
    c.bench_function("lexopt_errors", |b| {
        b.iter(|| {
            for argv in argv_error_cases() {
                let mut p = Parser::from_args(argv);
                let (mut has_x, mut has_y) = (false, false);
                loop {
                    use lexopt::Arg::{Long, Short};
                    match p.next() {
                        Ok(Some(Short('o') | Long("output"))) => {
                            // try to read a value; if missing, this is the error path we want
                            let _ = p.value();
                        }
                        Ok(Some(Long("x"))) => {
                            has_x = true;
                        }
                        Ok(Some(Long("y"))) => {
                            has_y = true;
                        }
                        Ok(Some(_)) => {}
                        Ok(None) | Err(_) => break,
                    }
                }
                black_box(has_x && has_y);
            }
        });
    });
}

fn lexopt_heavy(c: &mut Criterion) {
    use lexopt::Parser;
    c.bench_function("lexopt_heavy", |b| {
        b.iter(|| {
            for argv in argv_heavy_cases() {
                let mut p = Parser::from_args(argv);
                loop {
                    use lexopt::Arg::{Long, Short, Value};
                    match p.next() {
                        Ok(Some(Short('o' | 'j') | Long("output" | "jobs"))) => {
                            let _ = p.value().and_then(lexopt::ValueExt::string);
                        }
                        Ok(Some(Value(_x))) => {}
                        Ok(None) | Err(_) => break,
                        _ => {}
                    }
                }
            }
        });
    });
}

// ---------------------- clap ----------------------
fn clap_errors(c: &mut Criterion) {
    use clap::{Arg, ArgAction, Command};
    let cmd = Command::new("prog")
        .disable_help_flag(false)
        .arg(Arg::new("verbose").short('v').action(ArgAction::Count))
        .arg(Arg::new("output").short('o').long("output").required(true).num_args(1))
        .arg(Arg::new("x").long("x").action(ArgAction::SetTrue).conflicts_with("y"))
        .arg(Arg::new("y").long("y").action(ArgAction::SetTrue).conflicts_with("x"));
    c.bench_function("clap_errors", |b| {
        b.iter(|| {
            for argv in argv_error_cases() {
                let _ = cmd
                    .clone()
                    .try_get_matches_from(std::iter::once("prog").chain(argv.iter().copied()));
            }
        });
    });
}

fn clap_heavy(c: &mut Criterion) {
    use clap::{Arg, ArgAction, Command};
    let cmd = Command::new("prog")
        .disable_help_flag(false)
        .arg(Arg::new("verbose").short('v').action(ArgAction::Count))
        .arg(Arg::new("output").short('o').long("output").required(true).num_args(1))
        .arg(Arg::new("jobs").short('j').long("jobs").num_args(0..=1))
        .arg(Arg::new("FILE").num_args(0..=3));
    c.bench_function("clap_heavy", |b| {
        b.iter(|| {
            for argv in argv_heavy_cases() {
                let _ = cmd
                    .clone()
                    .try_get_matches_from(std::iter::once("prog").chain(argv.iter().copied()));
            }
        });
    });
}

criterion_group!(
    benches,
    rap_errors,
    rap_heavy,
    pico_errors,
    pico_heavy,
    lexopt_errors,
    lexopt_heavy,
    clap_errors,
    clap_heavy,
);
criterion_main!(benches);
