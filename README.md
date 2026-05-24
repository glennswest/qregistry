# qregistry — Container Registry Appliance

**Single scratch container. Multi-tenant. Per-repo rspacefs storage. Built for MikroTik Rose first.**

`qregistry` is a registry **appliance** — one image, no OS, supervised by [stormd](https://github.com/glennswest/stormd). It bundles:

- **[rspace_registry](https://github.com/glennswest/rspace_registry)** — Rust OCI Distribution Spec v1.1 registry head.
- **[rspacefs](https://github.com/glennswest/rspacefs)** — userspace LayerFS / FUSE storage. Each tenant repo is its own rspacefs filesystem.
- **qregistry-ui** — admin / GUI surface, registered as a stormd UI plugin so it shows up in stormd's nav.

stormd is PID 1 inside the container; it supervises the registry, the UI, and one `rspacefs-mount` daemon per repo. The container is built from `registry.gt.lo:5000/stormdbase` (multi-arch arm64/amd64) and runs unchanged on MikroTik Rose.

## Status

**v0.1.0 — scaffold.** Workspace layout, build chain, deploy artifacts.
Functional code arrives in subsequent commits per the work plan in [CLAUDE.md](./CLAUDE.md).

## Architecture

```
         scratch container (FROM stormdbase)
   ┌──────────────────────────────────────────────────────────┐
   │   stormd (PID 1)  ─ web dashboard on :9080               │
   │      ├── [[process]] qregistry-ui    127.0.0.1:8081      │
   │      │      └── registered as UI plugin "registry"       │
   │      │          surfaces in stormd nav via iframe        │
   │      │                                                   │
   │      ├── [[process]] rspace-registry  0.0.0.0:5000       │
   │      │      └── OCI /v2/* endpoints                      │
   │      │                                                   │
   │      ├── [[process]] rspacefs-mount.tenantA              │
   │      │      └── /var/lib/qregistry/repos/tenantA/        │
   │      ├── [[process]] rspacefs-mount.tenantB              │
   │      └── …  one per repo                                 │
   └──────────────────────────────────────────────────────────┘

   Ports:
     :9080  stormd dashboard + UI plugins (admin)
     :5000  OCI registry (podman push/pull)
     :22    stormsh (in-container shell)
```

## Build & Run

Cross-compile aarch64 musl static binary, build scratch image with podman, push to mkube registry which will sync to GHCR and roll the appliance on the Rose:

```sh
make image          # cross-compile + podman build + push to registry.gt.lo:5000
```

Local development run (no FUSE, plain dirs):

```sh
cargo run --release -p qregistry -- --config config/qregistry.toml
```

## Layout

```
qregistry/
├── Cargo.toml                  workspace
├── README.md / CLAUDE.md / CHANGELOG.md / LICENSE
├── crates/
│   ├── qregistry/              binary: CLI entry + main()
│   ├── qregistry-core/         lib: config types (Tenant, User, AppConfig)
│   └── qregistry-ui/           lib: axum server, HTML, admin pages
├── config/
│   ├── qregistry.toml          sample appliance config
│   └── stormd.toml             stormd config baked into the image
└── deploy/
    ├── Containerfile           FROM stormdbase, scratch
    └── Makefile                cross-compile + podman build + push
```

## Roadmap

- **v0.1**: scaffold, UI plugin registered with stormd, hard-coded tenant list, htpasswd auth file.
- **v0.2**: dynamic tenant CRUD (UI rewrites stormd config, requests reload). Per-repo rspacefs-mount lifecycle managed by qregistry.
- **v0.3**: offline mirror — pull-through cache + scheduled image sync to a USB/SD-card rspacefs partition.
- **v1.0**: Quay-parity feature set (orgs, robot accounts, signing, scanning hooks, audit log).

## License

MIT.
