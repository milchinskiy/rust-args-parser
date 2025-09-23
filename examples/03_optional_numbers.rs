use rust_args_parser as ap;

#[derive(Default)]
struct App {
    threshold: Option<f64>,
}

fn main() {
    let mut app = App::default();
    let env = ap::Env::new("optional-numbers").auto_help(true).auto_color();
    let root = ap::CmdSpec::new(None, None)
        .desc("Optional numeric value consumption: --thres[=X] or --thres -0.25 etc.")
        .opts([ap::OptSpec::new("thres", |v, u: &mut App| {
            u.threshold = v.map(|s| s.parse().unwrap());
            Ok(())
        })
        .short('t')
        .metavar("X")
        .help("Optional numeric threshold")
        .numeric() // enables numeric look-ahead like -t -0.5
        .optional()])
        .pos([]);

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args_cli: Vec<&str> = argv.iter().map(String::as_str).collect();
    match ap::dispatch(&env, &root, &args_cli, &mut app) {
        Ok(()) => println!("threshold={:?}", app.threshold),
        Err(ap::Error::Exit(code)) => std::process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2)
        }
    }
}
