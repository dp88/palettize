# Releasing

Release the library before the CLI: the CLI package depends on `palettize` by
version and must be able to resolve that version from crates.io.

1. Run the full local validation suite.
2. Check the library archive and registry upload without publishing it:
   `cargo package --list --package palettize` and
   `cargo publish --dry-run --package palettize`.
3. Publish the library with `cargo publish --package palettize`.
4. Wait until the new library version is available from crates.io.
5. Check the CLI archive and dry run:
   `cargo package --list --package palettize-cli` and
   `cargo publish --dry-run --package palettize-cli`.
6. Publish the CLI with `cargo publish --package palettize-cli`.

Only tag a release and create a GitHub release after both crates have been
published successfully.
