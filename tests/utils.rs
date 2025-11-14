use rust_args_parser as ap;

#[test]
fn looks_like_number_token_examples() {
    use ap::util::looks_like_number_token as is_num;
    assert!(is_num("-1"));
    assert!(is_num("-.5"));
    assert!(is_num("+3.14"));
    assert!(is_num("1e3"));
    assert!(is_num("-1.2e-3"));
    assert!(!is_num("-e10"));
    assert!(!is_num("--1"));
}

#[test]
fn strip_ansi_len_ignores_codes() {
    use ap::util::strip_ansi_len as sl;
    let s = "\x1b[1mHello\x1b[0m world"; // bold Hello
    assert_eq!(sl(s), "Hello world".len());
}
