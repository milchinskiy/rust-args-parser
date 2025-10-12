use rust_args_parser as ap;

#[derive(Default)]
struct App {
    threads: u32,
    out: Option<String>,
}

fn main() {
    let mut app = App::default();
    let env = ap::Env::new("env-defaults")
        .version("0.2.0")
        .author("Jane Dev <jane@example.com>")
        .auto_help(true)
        .auto_color()
        .wrap_cols(100);

    let root = ap::CmdSpec::new(None, Some(|_, _| Ok(())))
        .desc(
            "Demonstrates environment and defaults; built-ins -h/--help, -V/--version, -A/--author",
        )
        .opts([
            ap::OptSpec::new("threads", |v, u: &mut App| {
                u.threads = v.unwrap().parse().unwrap();
                Ok(())
            })
            .short('j')
            .metavar("N")
            .help("Number of worker threads (default from env THREADS or 4)")
            .numeric()
            .env("THREADS")
            .default("4")
            .required(),
            ap::OptSpec::new("output", |v, u: &mut App| {
                u.out = v.map(Into::into);
                Ok(())
            })
            .short('o')
            .metavar("FILE")
            .help("Write results to FILE")
            .optional(),
        ])
        .pos([]);

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args_cli: Vec<&str> = argv.iter().map(String::as_str).collect();
    match ap::dispatch(&env, &root, &args_cli, &mut app) {
        Ok(()) => println!("threads={}, out={:?}", app.threads, app.out),
        Err(ap::Error::Exit(code)) => std::process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2)
        }
    }
}
