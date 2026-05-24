# Changelog

## [Unreleased]

### 2026-05-24
- **feat:** Initial project scaffold — Cargo workspace with `qregistry`, `qregistry-core`, `qregistry-ui` crates.
- **feat:** qregistry-core defines `AppConfig`, `RegistryEndpoint`, `Tenant` (with `StorageTier`), `User`.
- **feat:** Tiered storage — `StorageTier` (fast/archive); each tier on a separate physical drive; tenant mounts derive to `<data_dir>/repos/<tier>/<name>`.
- **feat:** Two-registry model ("fast and slow registry") — `rspace-registry@fast` (:5000, NVMe) and `rspace-registry@archive` (:5001, ZFS HDD) as systemd template instances.
- **feat:** qregistry-ui — standalone dark admin pages (Overview/Repos/Users/System) showing registry endpoints, repo tiers, users.
- **feat:** qregistry binary — clap CLI loading TOML config, runs the UI server on 0.0.0.0:8081.
- **feat:** Deploy — `deploy/provision-ct.sh`: idempotent, data-safe `pct` create/update of CT 118 (`qregistry.g8.lo`, 192.168.8.50) from the stock `fedora-43-default` template; tier volumes mp0 (production-lvm NVMe, 500G) + mp1 (impulse1 ZFS, 2T) are never recreated so data survives rootfs rebuilds.
- **feat:** systemd units — `qregistry.service`, `rspace-registry@.service` (+ per-tier listen drop-in).
- **feat:** Packaging — `nfpm.yaml` builds rpm + deb; Fedora-minimal `Containerfile` + `entrypoint.sh` build the container image; `deploy/Makefile` for local rpm/deb/container.
- **feat:** CI/CD — `.github/workflows/ci.yml` builds the three public-repo binaries on a Linux runner (rspacefs/FUSE builds natively), packages rpm/deb/container, and provisions CT 118. Designed to run on forcicd (Forgejo Actions mirror).
- **docs:** README + CLAUDE.md describe the Proxmox-LXC/Fedora/systemd/tiered architecture, forcicd pipeline, and Proxmox facts for pve.g8.lo.

### Direction changes during initial build
- Pivoted deploy target from MikroTik Rose scratch-OCI (stormd PID 1) to **Proxmox LXC on a Fedora base** (systemd). Rose scratch-OCI flavor deferred to v1.0.
- Dropped stormd on Fedora (systemd supervises instead); this also keeps CI to public repos only.
- Build moved to the forcicd Linux runner because `rspacefs`/`fuser` cannot cross-compile from a macOS host.

