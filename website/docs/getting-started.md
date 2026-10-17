---
title: Getting started
sidebar_position: 2
description: Build the thalamus workspace from source, run its gate, and see what the command line does today.
---

# Getting started

thalamus has no release yet. What exists on `main` is the specification, the Rust model generated
from it, and a `thalamus` binary whose subcommands are declared but not implemented. This page
builds all three and runs the gate; every output below is pasted from a run.

## Requirements

- Rust and Cargo (edition 2024).
- [Task](https://taskfile.dev/), the task runner the gate is written for.
- The `ess` command line at the version `spec/ess-inputs.yaml` requires (`ess 0.60.0`), from
  [ESS](https://beyond10x.github.io/ess/) ([GitHub](https://github.com/beyond10x/ess)).

## 1. Check the specification

```console
$ git clone https://github.com/beyond10x/thalamus.git
$ cd thalamus
$ ess specify validate --path spec --strict-requires
thalamus v1 — 7 file(s), valid
```

The five domains it validates are on the [ESS specification](/docs/reference/ess) pages.

## 2. Run the gate

```console
$ task check
```

`task check` validates the specification, fails when `generated/` or `spec/suite.json` differ from
what `spec/` generates, then runs `cargo fmt --check`, `cargo clippy -D warnings`, every test of
the workspace, and `task docs-check`, which fails when this site's generated pages have fallen
behind the specification. It ends:

```text
     Running `target/debug/thalamus-docs generate --check --ess ess`
website/docs/reference/ess: 7 generated files current
website/data/ess: 7 generated files current
website/static/schemas: 6 generated files current
website/data: 1 generated files current
website/static/img: 1 generated files current
website/docs: no raw admonition titles
```

## 3. The command line

```console
$ cargo build --locked -p thalamus-cli
$ ./target/debug/thalamus --help
Samples signals, grades each sample into a verdict and shows the verdicts

Usage: thalamus [OPTIONS] <COMMAND>

Commands:
  sample   Read a definition's source and record one sample
  eval     Grade samples into evaluations
  show     Show the current verdicts
  compare  Compare a series with its own past
  rollup   Build daily rollups
  flaps    Measure how often verdicts flap
  help     Print this message or the help of the given subcommand(s)

Options:
      --instance <DIR>    The instance directory: `dimensions.yaml`, `definitions/*.yaml` and the registry files [env: THALAMUS_INSTANCE]
      --store <LOCATOR>   The instance's store: `sqlite:<path>` or a loopback `postgres://` URL without a password. Default: `sqlite:<instance>/.thalamus/store.sqlite` [env: THALAMUS_STORE]
      --connectors <BIN>  The `connectors` binary sources are read through (else `THALAMUS_CONNECTORS`, else `connectors` on `PATH`)
  -h, --help              Print help
  -V, --version           Print version
```

A usage error exits 64, never one of the verdict ladder's exit codes (0 to 3):

```console
$ ./target/debug/thalamus show --bogus
error: unexpected argument '--bogus' found

  tip: to pass '--bogus' as a value, use '-- --bogus'

Usage: thalamus show [OPTIONS] [SIGNAL_ID]

For more information, try '--help'.
$ echo $?
64
```

What each subcommand will do, and when, is on the [status page](/docs/status).

## Next

- [Signals](./concepts/signals.md): what a signal definition holds.
- [The verdict ladder](./concepts/verdicts.md): how a sample becomes a verdict.
- `task website` builds this site locally into `website/build`.
