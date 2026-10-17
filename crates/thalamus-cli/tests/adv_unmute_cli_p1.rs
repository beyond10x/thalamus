//! Adversary pass 1 on story:m2-auto-unmute: `thalamus sample` on a definition whose mute has
//! expired, against the fixture instance and the Connectors double. D19: "a `Muted` definition
//! whose expiry has passed is sampled as `Active`"; the service does so, the CLI refuses it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use thalamus_model::definition::SignalId;
use thalamus_store::eventlog::{EventlogStore, Locator};

const T1: &str = "2026-10-14T22:57:03Z";
const MUTED: &str = "service.batch.duration";

fn double() -> PathBuf {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT
        .get_or_init(|| {
            let status = Command::new(env!("CARGO"))
                .args([
                    "build",
                    "--locked",
                    "--quiet",
                    "-p",
                    "thalamus-sources",
                    "--bin",
                    "connectors-double",
                ])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .status()
                .expect("cargo runs");
            assert!(status.success(), "the Connectors double builds");
            Path::new(env!("CARGO_BIN_EXE_thalamus")).with_file_name("connectors-double")
        })
        .clone()
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create a directory");
    for entry in std::fs::read_dir(from).expect("the fixture exists") {
        let path = entry.expect("a fixture entry").path();
        let target = to.join(path.file_name().expect("a file name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("copy a fixture file");
        }
    }
}

fn thalamus(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_thalamus"))
        .arg("--instance")
        .arg(root.join("instance"))
        .arg("--connectors")
        .arg(double())
        .args(args)
        .env_remove("THALAMUS_INSTANCE")
        .env_remove("THALAMUS_STORE")
        .env_remove("THALAMUS_CONNECTORS")
        .env("CONNECTORS_DOUBLE_FIXTURES", root.join("connectors"))
        .env("CONNECTORS_DOUBLE_LOG", root.join("calls.jsonl"))
        .output()
        .expect("the thalamus binary runs")
}

/// The fixture's muted definition with its mute ended on 2026-10-10, before `--at` and before
/// today's wall clock.
fn world(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("cli_adv_unmute_p1")
        .join(name);
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("clear the test directory");
    }
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/instance"),
        &root.join("instance"),
    );
    std::fs::create_dir_all(root.join("connectors")).expect("recordings directory");
    let file = root
        .join("instance/definitions")
        .join(format!("{MUTED}.yaml"));
    let text = std::fs::read_to_string(&file).expect("the muted fixture");
    let expired = text.replace(
        "mute_until: 2026-11-01T00:00:00Z",
        "mute_until: 2026-10-10T00:00:00Z",
    );
    assert_ne!(text, expired, "the fixture's mute_until was replaced");
    std::fs::write(&file, expired).expect("expire the mute");
    root
}

#[test]
fn adv_cli_sample_of_an_expired_mute_is_not_refused() {
    let root = world("adv_cli_sample_of_an_expired_mute_is_not_refused");
    let out = thalamus(&root, &["sample", MUTED, "--at", T1]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("not-active"),
        "the service samples this definition as Active; the CLI refused it (exit {:?}): {stderr}",
        out.status.code()
    );
    // The unmute is the service's to record (actor Service); the CLI run records none.
    let locator = Locator::Sqlite(root.join("instance/.thalamus/store.sqlite"));
    let events = EventlogStore::open(&locator, "fixture")
        .expect("the store opens")
        .definition_events(&SignalId(MUTED.to_owned()));
    assert!(
        events.is_empty(),
        "the CLI run recorded an unmute: {events:?}"
    );
}
