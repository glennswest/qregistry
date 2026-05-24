# CLAUDE.md — qregistry

## What this project is

A **container registry appliance**. One scratch image built from
[stormdbase](https://github.com/glennswest/stormd), bundling
[rspace_registry](https://github.com/glennswest/rspace_registry) (OCI head)
and [rspacefs](https://github.com/glennswest/rspacefs) (per-repo
storage), plus a small admin UI that registers itself as a stormd UI
plugin so it surfaces in stormd's dashboard nav.

First deploy target: **MikroTik Rose** (ARM64). Same image runs on x86.

## Cross-project rules

Same rules as every project under `/Volumes/minihome/gwest/projects/`:

1. **All changes are approved.** Do not ask for confirmation.
2. **Commit and push after every logical unit of work.** No uncommitted state.
3. **Maintain `CHANGELOG.md`.**
4. **Docs stay current with code.**
5. **No claude attribution in commits.**
6. **Never build or deploy sibling projects** (rspace_registry, rspacefs, stormd). Only build/deploy this one. If a sibling change is required, write a spec at `../<sibling>/enhancements/`.
7. **Always use `podman`, NOT docker.**
8. **No sensitive data in commits.** Scan diffs before pushing.
9. **Semantic versioning** — pre-1.0 minor may include breaking changes.

## Build & Deploy

### Local dev (macOS, no FUSE)

```bash
cargo build --workspace --release
cargo run --release -p qregistry -- --config config/qregistry.toml
# UI on http://127.0.0.1:8081/
```

### Appliance image (per `/Volumes/minihome/gwest/CLAUDE.md` MikroTik Rose rules)

```bash
make image
# 1. cross-compile aarch64-unknown-linux-musl static binary
# 2. podman build --platform linux/arm64 -f deploy/Containerfile -t qregistry
# 3. podman push to registry.gt.lo:5000  (mkube syncs to GHCR + rolls the appliance)
```

The container is `FROM registry.gt.lo:5000/stormdbase:latest`. stormd is
PID 1. stormd's config (`config/stormd.toml`) declares three classes of
process:

- `qregistry-ui` — admin UI, port 8081, registered as stormd plugin
- `rspace-registry` — OCI registry head, port 5000
- `rspacefs-mount.<tenant>` — one daemon per repo, mounting rspacefs at `/var/lib/qregistry/repos/<tenant>/`

## Architecture

| Crate | Purpose |
|---|---|
| `crates/qregistry/` | binary; CLI entry point; loads TOML config, starts UI server |
| `crates/qregistry-core/` | lib; config types (`AppConfig`, `Tenant`, `User`), persistence |
| `crates/qregistry-ui/` | lib; axum HTTP server, HTML pages, JSON `/api/*` endpoints |

The UI renders **without its own nav chrome** because stormd embeds it
in an iframe inside stormd's nav. CSS palette matches stormd
(`#0f0f1a` bg, `#e94560` accent, `#50fa7b` green, `#8be9fd` cyan) so it
feels native.

## How qregistry shows up in stormd's nav

stormd reads `[process.ui] { label, proxy }` for any `[[process]]` in its
config and exposes that proxy URL as `/ui/ext/<process-name>` in its
dashboard nav. So `config/stormd.toml` declares:

```toml
[[process]]
name = "qregistry"
command = "/qregistry"
args = ["--config", "/etc/qregistry/qregistry.toml"]
on_failure = "restart"
on_exit = "restart"

[process.ui]
label = "Registry"
proxy = "http://127.0.0.1:8081"
```

That's the entire wiring. stormd handles routing, iframe chrome, and nav.

## Per-repo rspacefs

Each tenant repo gets a directory under `/var/lib/qregistry/repos/<name>/`
that is **the mount point of its own rspacefs filesystem**. stormd
supervises one `rspacefs-mount` process per repo. rspace-registry's
`FsStorage` is pointed at the mount point; the storage trait is
oblivious to whether the dir is FUSE or plain — that's stormd's problem.

For v0.1, the tenant list is **static** (read from `config/qregistry.toml`
at startup, and the matching rspacefs-mount processes are declared in
`config/stormd.toml`). Dynamic tenant CRUD (UI rewrites stormd config and
triggers reload) lands in v0.2.

## Work Plan

### v0.1.0 — Scaffold (current)

- [x] Workspace skeleton (Cargo.toml, three crates, README/CLAUDE/CHANGELOG/LICENSE)
- [x] `qregistry-core`: `AppConfig`, `Tenant`, `User` types + TOML loader
- [x] `qregistry-ui`: axum server, dark stormd-style HTML, tenant list page, users page, system status page
- [x] `qregistry` binary: clap CLI, loads config, starts UI server
- [x] `config/qregistry.toml` sample config
- [x] `config/stormd.toml` sample supervisor config
- [x] `deploy/Containerfile` (FROM stormdbase, COPY binary + configs)
- [x] `deploy/Makefile` cross-compile + podman build + push to registry.gt.lo:5000
- [x] GitHub repo created, initial commit pushed

### v0.2.0 — Dynamic tenants

- [ ] UI add-tenant form → writes tenant entry → reloads stormd via `POST /api/v1/processes/...`
- [ ] qregistry rewrites `config/stormd.toml` to add/remove `rspacefs-mount.<name>` and per-tenant `rspace-registry` blocks
- [ ] Tenant deletion: stop registry process, unmount rspacefs, archive data dir
- [ ] htpasswd file per tenant (or unified) — UI manages user CRUD

### v0.3.0 — Offline mirror

- [ ] Pull-through cache mode for a tenant: configurable upstream registry
- [ ] Scheduled sync to a removable rspacefs partition (USB / SD)
- [ ] UI shows mirror status, manual trigger

### v1.0 — Quay parity backlog

Orgs, robot accounts, scope-based RBAC, cosign signing, vulnerability
scanning hooks, audit log to local journal, tag immutability / retention.

## Cross-references

- **Sibling repos**: `../rspace_registry/`, `../rspacefs/`, `../stormd/`
- **stormd UI plugin contract**: `../stormd/crates/stormd/src/web.rs` (`build_plugin`, `nav_html`)
- **stormd process+UI config**: `../stormd/crates/stormd/src/config.rs` (`ProcessConfig`, `ProcessUiConfig`)
- **MikroTik Rose build rules**: `/Volumes/minihome/gwest/CLAUDE.md`
