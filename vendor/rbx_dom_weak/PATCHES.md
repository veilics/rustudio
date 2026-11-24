# rbx_dom_weak 4.2.0, patched for rustudio

Vendored from crates.io (`rbx_dom_weak` 4.2.0, part of
[rbx-dom](https://github.com/rojo-rbx/rbx-dom), MIT; `LICENSE.txt` is
upstream's licence). Used through `[patch.crates-io]` in the root
`Cargo.toml`.

Changes, all in `src/dom.rs` and marked `rustudio:`:

- `WeakDom::detach_children(referent)`: detaches every child of an
  instance at once (each child's parent becomes none) and returns them in
  order, in O(children).
- `WeakDom::detach_descendants(referent)`: detaches a whole subtree,
  returning every descendant (pre-order) with its former parent.
- `remove_child`: the places that took one child out of its parent's child
  list (`retain` over the whole list) now search from the end and remove
  just that entry.

Why: `Instance:Destroy()` and `ClearAllChildren()` on large models
(tens of thousands of parts) removed children one at a time, scanning the
parent's child list for each, which is quadratic. The new calls let
rustudio detach a subtree in one pass, and newest-first removal (what
ClearAllChildren does, as in Studio) no longer shifts the whole list.
Nothing else in the crate changed.

## 2026-11-24: big places (a 44 MB place of a million instances)

- `PropMap::own_keys`, `PropMap::has_refs` (a template remembers whether
  any shared value is a Ref), and `props::share_maps` (exported): what
  `share_properties` does, for maps not in a dom yet. rbx_binary's reader
  shares each class's values as soon as its chunks end.
- `InstanceBuilder::take_properties` / `set_prop_map`: `with_prop_map` in
  place, for that reader.
- `WeakDom::len` / `is_empty`.
- `clone_multiple_into_external` counts the subtree first and sizes the
  destination's tables once; its builders are made directly (no class-name
  String per instance); the reference fix-up skips maps without
  references; `inner_insert` interns "UniqueId" once. A whole place's
  copy for Play: 4.4 s → 3.6 s for a million instances.
