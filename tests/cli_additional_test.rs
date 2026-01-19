use clap::Parser;
use gwatch::cli::Args;

#[test]
fn test_long_verbose() {
    let args = Args::parse_from(["gwatch", "--verbose", "--verbose"]);
    assert_eq!(args.verbose, 2);
}

#[test]
fn test_path_with_equals() {
    let args = Args::parse_from(["gwatch", "--path=/custom/path"]);
    assert_eq!(args.path, "/custom/path");
}
