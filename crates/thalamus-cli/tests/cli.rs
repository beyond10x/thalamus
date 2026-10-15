//! The command-line contract every subcommand shares: the six subcommands are listed, and a usage
//! error exits 64, never a verdict ladder code (0 to 3) and never clap's default 2.

use std::process::{Command, Output};

const SUBCOMMANDS: [&str; 6] = ["sample", "eval", "show", "compare", "rollup", "flaps"];

fn thalamus(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_thalamus"))
        .args(args)
        .output()
        .expect("the thalamus binary runs")
}

fn listed_subcommands(help: &str) -> Vec<String> {
    help.lines()
        .skip_while(|line| line.trim() != "Commands:")
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next().map(str::to_owned))
        .filter(|name| name != "help")
        .collect()
}

#[test]
fn help_lists_the_six_subcommands_and_exits_0() {
    let out = thalamus(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let help = String::from_utf8(out.stdout).expect("help is UTF-8");
    let mut listed = listed_subcommands(&help);
    listed.sort();
    let mut expected: Vec<String> = SUBCOMMANDS.iter().map(|s| (*s).to_owned()).collect();
    expected.sort();
    assert_eq!(listed, expected, "help was:\n{help}");
}

#[test]
fn an_unknown_flag_exits_64() {
    let out = thalamus(&["--no-such-flag"]);
    assert_eq!(out.status.code(), Some(64));
    assert!(
        !out.stderr.is_empty(),
        "a usage error explains itself on stderr"
    );
}

#[test]
fn an_unknown_subcommand_exits_64() {
    assert_eq!(thalamus(&["no-such-subcommand"]).status.code(), Some(64));
}

#[test]
fn a_missing_subcommand_exits_64() {
    assert_eq!(thalamus(&[]).status.code(), Some(64));
}

#[test]
fn an_unknown_flag_on_every_subcommand_exits_64() {
    for name in SUBCOMMANDS {
        let out = thalamus(&[name, "--no-such-flag"]);
        assert_eq!(
            out.status.code(),
            Some(64),
            "thalamus {name} --no-such-flag"
        );
    }
}

#[test]
fn every_subcommand_answers_help_with_0() {
    for name in SUBCOMMANDS {
        let out = thalamus(&[name, "--help"]);
        assert_eq!(out.status.code(), Some(0), "thalamus {name} --help");
    }
}
