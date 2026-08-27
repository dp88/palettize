# Releasing

Release the library before the CLI: the CLI package depends on `palettize` by
version and must be able to resolve that version from crates.io.

1. Move the `Unreleased` changelog entries under a new version heading with
   today's date, and confirm the workspace version in `Cargo.toml`.
2. Run the full local validation suite.
3. Check the library archive and registry upload without publishing it:
   `cargo package --list --package palettize` and
   `cargo publish --dry-run --package palettize`.
4. Publish the library with `cargo publish --package palettize`.
5. Wait until the new library version is available from crates.io.
6. Check the CLI archive and dry run:
   `cargo package --list --package palettize-cli` and
   `cargo publish --dry-run --package palettize-cli`.
7. Publish the CLI with `cargo publish --package palettize-cli`.
8. After both crates have published, tag the release commit with an annotated
   tag and push it: `git tag -a vX.Y.Z -m "palettize X.Y.Z"`, then
   `git push origin vX.Y.Z`. Create the GitHub release from the tag.
