use rust_args_parser as ap;

#[derive(Default)]
struct App {
    mode: &'static str,
}

fn main() {
    let mut app = App::default();
    let env = ap::Env::new("groups").auto_help(true).auto_color();
    let root = ap::CmdSpec::new(None, None)
        .desc("Mutually-exclusive and required-one groups")
        .opts([
            ap::OptSpec::new("color", |_, _u: &mut App| Ok(())).help("Force color").at_most_one(1),
            ap::OptSpec::new("no-color", |_, _u: &mut App| Ok(()))
                .help("Disable color")
                .at_most_one(1),
            ap::OptSpec::new("mode-a", |_, u: &mut App| {
                u.mode = "A";
                Ok(())
            })
            .help("Mode A")
            .at_least_one(2),
            ap::OptSpec::new("mode-b", |_, u: &mut App| {
                u.mode = "B";
                Ok(())
            })
            .help("Mode B")
            .at_least_one(2),
        ])
        .pos([]);

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args_cli: Vec<&str> = argv.iter().map(String::as_str).collect();
    match ap::dispatch(&env, &root, &args_cli, &mut app) {
        Ok(()) => println!("mode={}", app.mode),
        Err(ap::Error::Exit(code)) => std::process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2)
        }
    }
}
