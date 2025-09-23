use rust_args_parser as ap;

#[derive(Default)]
struct App {
    verbose: bool,
}

fn main() {
    let mut app = App::default();
    let env = ap::Env::new("minimal").auto_help(true).auto_color();
    let root = ap::CmdSpec::new(None, None)
        .desc("Minimal example: one flag and one positional")
        .opts([ap::OptSpec::new("verbose", |_, u: &mut App| {
            u.verbose = true;
            Ok(())
        })
        .short('v')
        .help("Enable verbose output")
        .flag()])
        .pos([ap::PosSpec::new("FILE").one().desc("Input file")]);

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args_cli: Vec<&str> = argv.iter().map(String::as_str).collect();

    match ap::dispatch(&env, &root, &args_cli, &mut app) {
        Ok(()) => {
            println!("verbose={}, file={:?}", app.verbose, args_cli.last());
        }
        Err(ap::Error::Exit(code)) => std::process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2)
        }
    }
}
