<p align="center"><img src="assets/logo.svg" width="160" alt="thalamus"></p>

# thalamus

The realtime perception layer over the systems an organization runs. Thalamus reads numbers from
the instruments those systems already expose (metrics, logs, databases, orchestrators, version
control, test runs), records each reading as a typed **sample**, grades it into a **verdict**
against a baseline, and answers: *what is out of bounds right now, compared with what is normal for
it?*

**Documentation: <https://beyond10x.github.io/thalamus/>**

It pairs with [cortex](https://beyond10x.github.io/cortex/) ([GitHub](https://github.com/beyond10x/cortex)):
thalamus perceives, cortex remembers. Most samples stay in thalamus; verdict transitions and daily
rollups reach cortex as evidence-backed facts.

| | |
|---|---|
| status | specified, not yet implemented: the ESS specification and the generated model are on `main`; the libraries, the service and the CLI are planned ([status](https://beyond10x.github.io/thalamus/docs/status)) |
| specification | [five ESS domains](https://beyond10x.github.io/thalamus/docs/reference/ess), sources under [`spec/`](spec/) |
| design | [docs/design/thalamus.md](docs/design/thalamus.md) |
| language | Rust: libraries plus a service and a CLI |
| secrets | none; every source is a [Connectors](https://beyond10x.github.io/connectors/) ([GitHub](https://github.com/beyond10x/connectors)) connection |
| changes | [CHANGELOG.md](CHANGELOG.md) |
| licence | [Apache-2.0](LICENSE) |

## Build and check

Needs Rust, [Task](https://taskfile.dev/) and the `ess` CLI at the version `spec/ess-inputs.yaml`
requires.

```bash
git clone https://github.com/beyond10x/thalamus.git
cd thalamus
task check
```

[Getting started](https://beyond10x.github.io/thalamus/docs/getting-started) walks through it.
The concepts (signals, samples, the verdict ladder, baselines, the score) are on the
[site](https://beyond10x.github.io/thalamus/docs/concepts/verdicts).

Agents start at [AGENTS.md](AGENTS.md).
