# Changelog

## [Unreleased]

### 2026-05-24
- **feat:** Initial project scaffold — Cargo workspace with `qregistry`, `qregistry-core`, `qregistry-ui` crates.
- **feat:** qregistry-core defines `AppConfig`, `Tenant`, `User` types.
- **feat:** qregistry-ui serves dark-themed admin HTML on port 8081, registered as stormd UI plugin.
- **feat:** qregistry binary loads TOML config and starts the UI server.
- **feat:** Deploy artifacts — `Containerfile` (FROM stormdbase), `Makefile` for aarch64 cross-compile + podman build + push to `registry.gt.lo:5000`.
- **feat:** Sample `config/stormd.toml` that supervises qregistry-ui + rspace-registry + per-repo rspacefs-mount processes.
- **docs:** README + CLAUDE.md describe architecture, roadmap, and per-repo rspacefs storage model.
