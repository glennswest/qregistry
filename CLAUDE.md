# CLAUDE.md — qregistry

## What this project is

A **container registry appliance** delivered as a **Proxmox LXC** on a
**Fedora base**, supervised by **systemd**. It bundles
[rspace_registry](https://github.com/glennswest/rspace_registry) (OCI head,
one instance per storage tier) and [rspacefs](https://github.com/glennswest/rspacefs)
(per-repo storage, v0.2), plus a small admin UI (this repo) as the front door.

First deploy target: **CT 118 on `pve.g8.lo`** — `qregistry.g8.lo`,
192.168.8.50, vmbr0/g8. Built and deployed by
[forcicd](https://github.com/glennswest/forcicd) (local Forgejo Actions).

### Key decisions (locked)

- **No stormd on Fedora** — systemd is init + supervisor. (stormd and its
  `stormbase`/`stormpull` dep are private repos; dropping it keeps CI to
  public repos only.)
- **Tiered "fast and slow registry"** — two `rspace-registry` instances on
  separate physical drives: fast=:5000 NVMe (`production-lvm`),
  archive=:5001 ZFS HDD (`impulse1`).
- **Data survives rootfs rebuilds** — tier data on separate Proxmox mount-
  point volumes (mp0 fast 500G, mp1 archive 2T); `provision-ct.sh` never
  recreates an existing volume.
- **Ship four artifact types** — rpm, deb, container image, and the LXC.

## Cross-project rules

Same rules as every project under `/Volumes/minihome/gwest/projects/`:

1. **All changes are approved.** Do not ask for confirmation (but confirm
   shared-infra coordinates like CT IDs / IPs / storage before creating).
2. **Commit and push after every logical unit of work.**
3. **Maintain `CHANGELOG.md`.**
4. **Docs stay current with code.**
5. **No claude attribution in commits.**
6. **Don't modify sibling source** (rspace_registry, rspacefs). Building
   their binaries read-only in CI is fine; source changes go via a spec at
   `../<sibling>/enhancements/`.
7. **Always use `podman`, NOT docker.**
8. **No sensitive data in commits.** Scan diffs before pushing.
9. **Semantic versioning** — pre-1.0 minor may include breaking changes.

## Build & Deploy

### Local dev (just the UI)

```bash
cargo run -p qregistry -- --config config/qregistry.toml   # UI on :8081
```

### CI/CD (forcicd) — the real path

Push to `main` → forcicd mirrors the repo (1-min poll) → `.github/workflows/ci.yml`:

1. **build** — `cargo build --release` for qregistry; clone + build the two
   public siblings (`rspace-registry`, `rspacefs-mount`) on the Linux
   runner (FUSE/`fuser` compiles natively there — it cannot cross-compile
   from macOS). Upload binaries artifact.
2. **package** — `nfpm` → rpm + deb.
3. **container** — `podman build` the Fedora-minimal appliance image, push
   to `forcicd.g8.lo:5000`.
4. **deploy** — scp binaries+config+deploy to `pve.g8.lo`, run
   `deploy/provision-ct.sh` to create/update CT 118.

Required Forgejo Actions secrets:
- `GH_TOKEN` — clone sibling public repos / avoid rate limits.
- `PVE_SSH_KEY` — ed25519 private key authorized as `root@pve.g8.lo`.

### Local packaging

```bash
cd deploy && make rpm deb container     # needs nfpm + podman
```

## In-CT layout (systemd services)

| Service | Role | Port | Data |
|---|---|---|---|
| `qregistry.service` | admin UI | 8081 | — |
| `rspace-registry@fast.service` | OCI registry, fast tier | 5000 | mp0 NVMe |
| `rspace-registry@archive.service` | OCI registry, archive tier | 5001 | mp1 ZFS |

`rspace-registry@.service` is a template; the per-tier `--listen` port is
set by a drop-in (`/etc/systemd/system/rspace-registry@archive.service.d/listen.conf`).

## Crates

| Crate | Purpose |
|---|---|
| `crates/qregistry/` | binary; CLI; loads config, runs the axum UI |
| `crates/qregistry-core/` | `AppConfig`, `RegistryEndpoint`, `Tenant` (with `StorageTier`), `User` |
| `crates/qregistry-ui/` | axum server + dark admin pages (Overview / Repos / Users / System) |

## Storage tiers

`StorageTier` = `fast` | `archive`. A tenant's rspacefs mount derives to
`<data_dir>/repos/<tier>/<name>`. Tiers map to separate Proxmox volumes /
physical drives (see the table above).

## Proxmox facts (pve.g8.lo)

- Stock template in use: `local:vztmpl/fedora-43-default_20251224_amd64.tar.xz`.
- Storage → drive: `production-lvm`=NVMe Samsung 990 4TB; `test-lvm-thin`=NVMe
  Crucial P3 2TB; `services-lvm-thin`=2×SATA SSD striped; `impulse1`=ZFS on
  8TB Seagate HDD; `local-lvm`=SATA SSD (boot).
- Free VMIDs were 118–120; we use **118**.
- Host SSH: `root@pve.g8.lo` (key-authorized).

## Work Plan

### v0.1.0 — Tiered Fedora LXC via forcicd (current)

- [x] Workspace skeleton + docs
- [x] `qregistry-core`: AppConfig, RegistryEndpoint (per-tier), Tenant(tier), User
- [x] `qregistry-ui`: standalone dark admin pages (no stormd iframe)
- [x] `qregistry` binary: clap CLI + UI server
- [x] Tiered config (`config/qregistry.toml`) — fast :5000 / archive :5001
- [x] systemd units (`qregistry.service`, `rspace-registry@.service`)
- [x] `deploy/provision-ct.sh` — idempotent, data-safe CT create/update
- [x] Packaging: `nfpm.yaml` (rpm+deb), Containerfile + entrypoint
- [x] `.github/workflows/ci.yml` — build → package → container → deploy
- [ ] First green forcicd run + CT 118 live
- [ ] Set `GH_TOKEN` + `PVE_SSH_KEY` Forgejo secrets (forcicd-side)

### v0.2.0 — Per-repo rspacefs + dynamic CRUD

- [ ] qregistry manages one `rspacefs-mount` per tenant under its tier
- [ ] UI add/remove tenant + user; rewrite config + reload services
- [ ] htpasswd export so rspace-registry `--auth-file` consumes the user list

### v0.3.0 — Offline mirror

- [ ] Pull-through cache per tenant (configurable upstream)
- [ ] Scheduled sync to a removable rspacefs partition; UI status + trigger

### v1.0 — Quay parity

Orgs, robot accounts, RBAC, cosign signing, scanning hooks, audit log,
tag immutability/retention. MikroTik Rose scratch-OCI flavor.

## Cross-references

- **Sibling repos**: `../rspace_registry/`, `../rspacefs/` (public; built in CI)
- **CI/CD**: `../forcicd/` — `scripts/bulk-mirror.sh qregistry` to onboard
- **Proxmox/Rose build rules**: `/Volumes/minihome/gwest/CLAUDE.md`
