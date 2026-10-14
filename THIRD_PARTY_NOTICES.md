# Third-party notices

The root ISC license covers RS Mod/Project Gielinor-origin material and the owner-confirmed newly authored Rust contributions; it is not a blanket relicensing of third parties. New Rust copyright attribution is “Copyright (c) 2026 Project Gielinor Rust contributors,” expressly confirmed by the owner for this preparation.

Project Gielinor / RS Mod: ISC, Copyright (c) 2025 RS Mod. Full preserved notice: licenses/Project-Gielinor-ISC.txt. Public project: https://github.com/Slashx124/Project-Gielinor. Source identity distinctions are in SOURCE_PROVENANCE.md.

RSProt: MIT, Copyright (c) 2024 Z-Kris. Full notice: licenses/RSProt-MIT.txt. Project: https://github.com/blurite/rsprot. Preserve this notice for adapted protocol/crypto/wire behavior material; do not attribute new Rust code wholesale to Z-Kris.

Vendored crates are an offline source subset with pre-existing reduced Cargo manifests. No vendor algorithm changes are made in this packaging pass. Original manifests and per-file notices are retained. SPDX-compatible declared expressions and file paths are in DEPENDENCIES.json:

- adler2 2.0.1: `0BSD OR MIT OR Apache-2.0`; vendor/adler2-2.0.1/LICENSE-0BSD, vendor/adler2-2.0.1/LICENSE-APACHE, vendor/adler2-2.0.1/LICENSE-MIT.
- autocfg 1.4.0: `Apache-2.0 OR MIT`; vendor/autocfg-1.4.0/LICENSE-APACHE, vendor/autocfg-1.4.0/LICENSE-MIT.
- libbz2-rs-sys 0.2.2: `bzip2-1.0.6`; vendor/libbz2-rs-sys-0.2.2/LICENSE.
- miniz_oxide 0.8.9: `MIT OR Zlib OR Apache-2.0`; vendor/miniz_oxide-0.8.9/LICENSE, vendor/miniz_oxide-0.8.9/LICENSE-APACHE.md, vendor/miniz_oxide-0.8.9/LICENSE-MIT.md, vendor/miniz_oxide-0.8.9/LICENSE-ZLIB.md.
- num-bigint 0.4.6: `MIT OR Apache-2.0`; vendor/num-bigint-0.4.6/LICENSE-APACHE, vendor/num-bigint-0.4.6/LICENSE-MIT.
- num-integer 0.1.46: `MIT OR Apache-2.0`; vendor/num-integer-0.1.46/LICENSE-APACHE, vendor/num-integer-0.1.46/LICENSE-MIT.
- num-traits 0.2.19: `MIT OR Apache-2.0`; vendor/num-traits-0.2.19/LICENSE-APACHE, vendor/num-traits-0.2.19/LICENSE-MIT.

libbz2-rs-sys is under bzip2-1.0.6 terms, not MIT. miniz_oxide retains its embedded miniz notices. Optional runtime Argon2 and OS libraries are not redistributed here and retain their own terms. No Kotlin reference tree, Netty/JVM jars, original cache assets or Kronos archive is included. Jagex game assets are not licensed by ISC or MIT. This unofficial project does not imply Jagex endorsement.
