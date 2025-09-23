use rust_args_parser as ap;

#[derive(Default)]
struct App;

fn main() {
    let mut app = App;
    let env = ap::Env::new("sc").auto_help(true).auto_color().wrap_cols(80);

    let remote_add = ap::CmdSpec::new(
        Some("add"),
        Some(|pos, _u: &mut App| {
            println!("remote add url={}", pos[0]);
            Ok(())
        }),
    )
    .desc("Add a remote")
    .pos([ap::PosSpec::new("URL").one()]);

    let remote = ap::CmdSpec::new(Some("remote"), None)
        .desc("Remote management")
        .aliases(["r"])
        .subs([remote_add]);

    let root = ap::CmdSpec::new(None, None).desc("Tool with a subcommand").subs([remote]);

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args_cli: Vec<&str> = argv.iter().map(String::as_str).collect();
    match ap::dispatch(&env, &root, &args_cli, &mut app) {
        Ok(()) => {}
        Err(ap::Error::Exit(code)) => std::process::exit(code),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2)
        }
    }
}
