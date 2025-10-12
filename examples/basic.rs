use rust_args_parser as ap;

#[derive(Default)]
struct App {
    verbose: bool,
    n: i32,
    limit: Option<String>,
}

const DESC: &str = "Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";

fn main() {
    let mut app = App::default();
    let env = ap::Env::new("demo").version("0.1.2").author("John Doe").auto_help(true).auto_color();
    let root = ap::CmdSpec::new(
        None,
        Some(|pos, u: &mut App| {
            println!("pos: {pos:?}");
            println!("cfg: v={} n={} lim={:?}", u.verbose, u.n, u.limit);
            Ok(())
        }),
    )
    .desc(DESC)
    .subs([ap::CmdSpec::new(Some("sub"), Some(|_, _| Ok(()))).desc("subcommand description")])
    .opts([
        ap::OptSpec::new("verbose", |_, u: &mut App| {
            u.verbose = true;
            Ok(())
        })
        .short('v')
        .env("VERBOSE")
        .help("Verbose output")
        .flag(),
        ap::OptSpec::new("jobs", |v, u: &mut App| {
            u.n = v.unwrap().parse().unwrap();
            Ok(())
        })
        .short('j')
        .metavar("N")
        .help("Worker threads")
        .default("4")
        .required(),
        ap::OptSpec::new("limit", |v, u: &mut App| {
            u.limit = v.map(std::convert::Into::into);
            Ok(())
        })
        .short('l')
        .metavar("N")
        .help("Optional limit")
        .optional(),
    ])
    .pos([ap::PosSpec::new("FILE").range(1, 10)]);
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let cli_args: Vec<&str> = argv.iter().map(String::as_str).collect();
    if let Err(error) = ap::dispatch(&env, &root, &cli_args, &mut app) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
