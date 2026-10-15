//! Adversary case: the acceptance's stub contract, which `tests/cli.rs` does not assert. Each of
//! the six subcommands exits 70 and says `not implemented`. Each owner story deletes its own line
//! from `STUBS` when it replaces the stub.

use std::process::Command;

const STUBS: [&str; 6] = ["sample", "eval", "show", "compare", "rollup", "flaps"];

#[test]
fn every_stub_subcommand_exits_70_saying_not_implemented() {
    for name in STUBS {
        let out = Command::new(env!("CARGO_BIN_EXE_thalamus"))
            .arg(name)
            .output()
            .expect("the thalamus binary runs");
        assert_eq!(out.status.code(), Some(70), "thalamus {name}");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            text.contains("not implemented"),
            "thalamus {name} printed {text:?}"
        );
    }
}
