# bundled-pm-change-1.96.1

## Pattern

`bundled_pm_change` — targets behavior changes introduced by the
Cargo version bundled with a specific Rust toolchain release.

## Feature exercised

Rust 1.96.1 ships with a patched Cargo that includes:

1. **libssh2 security patch** — affects SSH-based git source fetching.
   Even for HTTPS git sources, the patched libssh2 is compiled in and
   participates in TLS credential resolution on some platforms. This
   probe includes a git-sourced dependency
   (`tinytemplate` at a specific commit SHA) to exercise this path.

2. **Network timeout and retry fixes** — affects registry-source
   dependency fetching. This probe includes a realistic set of
   registry-sourced crates (including transitive deps through
   `env_logger` and `serde`) to exercise the registry fetch path
   under the fixed retry logic.

3. **Workspace dependency inheritance** (`[workspace.dependencies]`,
   Cargo 1.64+) — both workspace members inherit shared deps from the
   root manifest; Mend's resolver must follow inheritance to see the
   correct versions and feature sets.

## Categories

- `bundled_pm_change`
- `git_source`
- `registry_source`
- `workspace`

## PM version under test

Rust / Cargo `1.96.1` (Cargo PM version tracks Rust toolchain version
one-to-one; the `rust-toolchain.toml` file pins both).

## Workspace layout

```
Cargo.toml          workspace root (no [package]; resolver = "2")
Cargo.lock
rust-toolchain.toml
.whitesource
app/
  Cargo.toml        binary crate — probe-app
  src/main.rs
core/
  Cargo.toml        library crate — probe-core
  src/lib.rs
```

## Dependency sources exercised

### git source (libssh2 / fetch path)

`tinytemplate` is pulled from GitHub at tag `v1.2.1` (pinned to
commit `d0f45f0c3df21db7d82b1e1c3bfe3ca9e14f11d1`). This forces
Cargo to invoke its git-clone/fetch machinery (and the bundled
libssh2) at scan time if the UA runs `cargo metadata` without a
pre-populated cache.

### registry sources (timeout/retry path)

| Crate          | Version   | Why chosen                              |
|----------------|-----------|-----------------------------------------|
| `serde`        | 1.0.219   | Ubiquitous; proc-macro transitive chain |
| `log`          | 0.4.22    | Stable, no transitive deps              |
| `thiserror`    | 2.0.12    | Proc-macro dep chain (syn, quote, ...)  |
| `once_cell`    | 1.20.3    | No deps; simple registry entry          |
| `env_logger`   | 0.11.5    | Pulls in regex, anstream, humantime     |
| `serde_json`   | 1.0.140   | Transitive via tinytemplate             |

### local (path) source

`probe-core` is declared as `{ path = "../core" }` from `probe-app`,
making it a workspace-internal path dependency. Mend must detect its
source as `local` and resolve its own transitive deps.

## Expected dependency tree

All packages in `Cargo.lock` must appear in Mend's output. Key
expectations:

- `tinytemplate 1.2.1` — `source: git`, URL
  `https://github.com/bheisler/TinyTemplate`, commit SHA
  `d0f45f0c3df21db7d82b1e1c3bfe3ca9e14f11d1`.
- `serde 1.0.219` — `source: registry` (crates.io),
  `dependencies: ["serde_derive"]`.
- `serde_derive 1.0.219` — `source: registry`,
  `dependencies: ["proc-macro2", "quote", "syn"]`.
- `env_logger 0.11.5` — `source: registry`,
  `dependencies: ["anstream", "anstyle", "env_filter", "humantime", "log"]`.
- `probe-core 0.1.0` — `source: local` (inter-member path dep from
  `probe-app`).
- All 30+ transitive registry crates (regex, aho-corasick, memchr,
  windows-sys family, etc.) must be present.
- No dep must appear twice at the same version; no cross-version
  duplication for this dep set.
- `group` for all production deps should be `main` (no dev or build
  deps in this probe).
- Workspace-inherited versions (`serde 1.0.219`, `log 0.4.22`, etc.)
  must be consistent across both `probe-app` and `probe-core`.

## Mend failure modes to watch

1. **git dep missing or misidentified** — `tinytemplate` reported as
   a registry dep, or absent entirely.
2. **git commit SHA lost** — only the tag/rev `v1.2.1` reported, not
   the resolved SHA.
3. **workspace inheritance not followed** — `serde` or `log` shown as
   missing from one workspace member because the resolver didn't
   expand `workspace = true`.
4. **transitive deps truncated** — the regex/anstream chain from
   `env_logger` partially or fully absent.
5. **inter-member path dep misclassified** — `probe-core` reported as
   a registry crate instead of `source: local`.
6. **version drift under retry** — if the UA's new retry logic selects
   a different resolved version on a slow network than the lockfile
   records. Detection: Mend tree differs from `Cargo.lock`.

## Resolver knowledge note

The upstream Mend UA resolver knowledge file for Cargo
(`resolvers/cargo.md`) was not accessible from this environment at
probe generation time (HTTP 404 on the GitHub raw URL). This probe
is therefore partially **exploratory** with respect to:

- Whether the UA reads `Cargo.lock` directly (lockfile-driven) or
  shells out to `cargo metadata` (resolver-driven).
- Whether `rust.runPreStep` / `rust.resolveDependencies` toggles
  exist in the UA config.
- Whether the UA follows git deps into sub-dependencies (e.g.
  `tinytemplate` → `serde_json`).

Until the resolver file can be confirmed, downstream comparators
should treat any divergence involving `tinytemplate` or
workspace-inheritance as exploratory rather than a confirmed
regression.

## Mend config

Bucket B — `.whitesource` emitted with `scanSettings.versioning`
pinning `rust: "1.96.1"`. Cargo is a Bucket B PM (dynamic detection
is "limited" — the `edition` field and `rust-toolchain.toml` give
partial version information, but they do not uniquely identify a
patch release). Because this probe specifically targets Rust 1.96.1
behavior (libssh2 patch, timeout/retry fixes), the version must be
pinned exactly via `install-tool` to reproduce the conditions.

`rust-toolchain.toml` is also included so that any developer running
`cargo` locally gets the same toolchain. Mend UA reads
`rust-toolchain.toml` if present (as of the editions documented in
the knowledge files) and may use it to influence toolchain selection;
the `.whitesource` pin is the authoritative override for Mend scans.

`configMode` is `"AUTO"` (no companion `whitesource.config` in this
probe — standard UA resolution behavior is sufficient).
