# Adversarial breach fixtures (pass 2)

The instance of `tests/adv_sample_breach_pass2.rs`, built like `tests/fixtures/breach`: synthetic
free-space floors in bytes, `lower_is_worse`, `crit: {x: "1"}`.

- `platform.disk.atfloor` reads a value exactly at its floor.
- `platform.disk.mixed` is graded against `offset_1d` alone (the loader refuses `fixed` beside any other baseline), so it is not graded at sample time.
- `platform.host.free` reads `host.disk_free` of `/` on the machine running the test, under a
  floor of 10^18 bytes that no disk reaches.
