# rbx_binary 3.0.1, patched for rustudio

Vendored from crates.io (`rbx_binary` 3.0.1, part of
[rbx-dom](https://github.com/rojo-rbx/rbx-dom), MIT; `LICENSE.txt` is
upstream's licence). Used through `[patch.crates-io]` in the root
`Cargo.toml`. The benches (and their test files) aren't vendored.

Changes, in `src/deserializer/` and marked `rustudio`:

- `Deserializer::skip_defaults(f)`: `f(class, property)` gives a value
  not to store (None: store every value). Asked once per PROP chunk, for
  unmigrated properties; `add_property` leaves out a value equal to it.
  rustudio's `doc.rs implied_default` answers with what `Document::prop`
  falls back to for a missing property.
- With `skip_defaults`, an instance's property list starts empty instead
  of sized for every property its class has (~45-60 values of 72 bytes:
  2.2 GB for a million instances before a value was read).
- With `skip_defaults`, a class's instances share their common values
  (rbx_dom_weak `share_maps`) as soon as the file moves on from that
  class's PROP chunks (and at the end), instead of rustudio's
  `share_properties` after the whole file.
- `finish` reserves the tree's instance table once.

Why: a 44 MB place (996 380 instances, 25.6 M stored values, 69% of them
defaults) parsed to ~4 GB and crashed the editor on a phone. Now ~1.9 GB
at the parse's peak, and every property reads the same
(`rustudio-core doc.rs implied_default_tests`).
