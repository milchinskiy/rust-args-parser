use rust_args_parser as ap;

#[derive(Default)]
struct App {
    count: u32,
    limit: Option<u32>,
}

fn main() {
    let mut app = App::default();
    let env = ap::Env::new("flags-values").version("0.1.0").auto_help(true).auto_color();
    let root = ap::CmdSpec::new(None, Some(|_, _| Ok(())))
        .desc("Flags, required and optional values; short clusters like -vvv, -n10")
        .opts([
            ap::OptSpec::new("verbose", |_, u: &mut App| {
                u.count += 1;
                Ok(())
            })
            .short('v')
            .help("Increase verbosity (can repeat)")
            .flag(),
            ap::OptSpec::new("n", |v, _u: &mut App| {
                let n: u32 = v.unwrap().parse().map_err(|_| ap::Error::User("bad -n"))?;
                println!("n = {n}");
                Ok(())
            })
            .short('n')
            .metavar("N")
            .help("Required number")
            .numeric()
            .required(),
            ap::OptSpec::new("limit", |v, u: &mut App| {
                u.limit = v.map(|s| s.parse::<u32>().unwrap());
                Ok(())
            })
            .short('l')
            .metavar("N")
            .help("Optional limit")
            .optional(),
        ])
        .pos([ap::PosSpec::new("PATH").desc("Path to process").one()]);

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args_cli: Vec<&str> = argv.iter().map(String::as_str).collect();
    match ap::dispatch(&env, &root, &args_cli, &mut app) {
        Ok(()) => println!("verbosity={}, limit={:?}", app.count, app.limit),
        Err(ap::Error::Exit(code)) => std::process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2)
        }
    }
}
