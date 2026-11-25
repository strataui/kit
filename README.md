# Strata Kit (`strataui/kit`)

Application-layer batteries for [Strata](https://github.com/strataui/strata) apps: routing, internationalization, global state stores, and local-first CRDTs.

## Crates

- **`strata-router`** — Radix-tree client/app router with nested layouts, guards, and dynamic params.
- **`strata-i18n`** — ICU/Fluent-style locale catalog, pluralization, and RTL-aware formatting.
- **`strata-ui-store`** — Slice-based global reactive store with middleware, snapshots, and time-travel undo/redo.
- **`strata-crdt`** — Pure-Rust conflict-free replicated data types (`LwwRegister`, `PnCounter`, `OrSet`, causal `RgaSequence`) bridged to Strata reactive signals.

## License

MIT
