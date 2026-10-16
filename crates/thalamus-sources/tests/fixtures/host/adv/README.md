# Adversarial host fixtures

`plain/` is a root read through the collector's own `Rooted` reader, so `statvfs` is the real
call: `plain/data` is a directory and `plain/file.txt` a regular file, and neither is a mount
point. Used by `tests/adv_host_pass1.rs`.
