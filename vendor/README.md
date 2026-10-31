# sa-token-core security source

`sa-token-core/` is the crates.io `sa-token-core` 0.2.1 archive, SHA-256
`331bce2589545d0c535864066be9ebf63cfdea909f4cfb24294d151cbe385910`.
Its manifest declares `MIT OR Apache-2.0`; the published archive contains no
separate license file. Original `Cargo.toml.orig`, attribution, and README are
kept with the source. The archive's nested Cargo.lock and registry cache marker
are omitted so audits use the workspace lockfile that actually builds.

The only source changes are in the normalized `Cargo.toml`:

- `jsonwebtoken` 10.4 uses `aws_lc_rs` instead of `rust_crypto` to avoid the
  vulnerable `crypto-bigint` 0.5.x and unpatched `rsa` 0.9.x dependencies.
- `rand_core` 0.6 explicitly enables `getrandom`. The upstream source uses
  `argon2::password_hash::rand_core::OsRng`; previously it relied on a
  transitive feature from the removed crypto backend. Without this line the
  crate fails to compile as a standalone dependency.

`thymeleaf-sa-token` uses a direct path dependency so consumers of this Git
checkout resolve the reviewed source. Cargo rewrites path dependencies to
registry dependencies when packaging. Therefore this source does **not** make
an eventual crates.io release safe by itself. Release requires a published
compatible sa-token-core version or a separately reviewed distribution plan.
