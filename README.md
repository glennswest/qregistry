# qregistry — Container Registry Appliance

**Proxmox LXC. Fedora base. Tiered (fast/archive) storage. Multi-tenant.**

`qregistry` is a registry **appliance** delivered as a Proxmox LXC on a
Fedora base, supervised by **systemd**. First deploy target: CT 118 on
`pve.g8.lo` (`qregistry.g8.lo`, 192.168.8.50). It bundles:

- **[rspace_registry](https://github.com/glennswest/rspace_registry)** — Rust OCI Distribution Spec v1.1 registry head. One instance per storage tier.
- **[rspacefs](https://github.com/glennswest/rspacefs)** — userspace LayerFS / FUSE storage. Each tenant repo becomes its own rspacefs filesystem (v0.2).
- **qregistry** (this repo) — admin / GUI front door + config model.

Built and deployed by **[forcicd](https://github.com/glennswest/forcicd)** — the
local Forgejo Actions CI/CD on the LAN — which builds the binaries on a
Linux runner (where `rspacefs`/FUSE compiles natively) and provisions the
CT via `pct`. Outputs: **rpm, deb, container image, and the LXC**.

## Tiered storage — "fast and slow registry"

Two `rspace-registry` instances, each on a **separate physical drive**:

| Tier | Port | Drive | Mount |
|---|---|---|---|
| **fast** | 5000 | NVMe (`production-lvm`, Samsung 990 4TB) | `/var/lib/qregistry/repos/fast` |
| **archive** | 5001 | ZFS HDD (`impulse1`, Seagate 8TB) | `/var/lib/qregistry/repos/archive` |

The data volumes are **separate from the CT rootfs**, so the code/rootfs
can be rebuilt without touching repo data. Each repo (tenant) is assigned
a tier and gets its own rspacefs filesystem under that tier's mount.

## Architecture

```
   Proxmox CT 118 — qregistry.g8.lo (192.168.8.50)   Fedora base, systemd PID 1
   ┌────────────────────────────────────────────────────────────────┐
   │  systemd                                                         │
   │   ├── qregistry.service             admin UI  :8081             │
   │   ├── rspace-registry@fast.service  OCI       :5000  ──► mp0    │
   │   │                                       (NVMe, production-lvm) │
   │   └── rspace-registry@archive.service OCI     :5001  ──► mp1    │
   │                                          (ZFS HDD, impulse1)     │
   │                                                                  │
   │   mp0  /var/lib/qregistry/repos/fast      500G NVMe  (persists)  │
   │   mp1  /var/lib/qregistry/repos/archive   2T   ZFS   (persists)  │
   │   rootfs                                  20G        (rebuildable)│
   └────────────────────────────────────────────────────────────────┘
```

## Build & Deploy (via forcicd)

Push to `main` → forcicd mirrors the repo → the workflow
(`.github/workflows/ci.yml`) builds the three binaries on a Linux runner,
produces rpm/deb/container, and runs `deploy/provision-ct.sh` on
`pve.g8.lo` to create/update CT 118.

Required Forgejo Actions secrets: `GH_TOKEN` (clone sibling public repos),
`PVE_SSH_KEY` (root@pve.g8.lo for the deploy job).

Local dev run of just the UI (no registries):

```sh
cargo run -p qregistry -- --config config/qregistry.toml   # UI on http://127.0.0.1:8081
```

Local packaging:

```sh
cd deploy && make rpm deb container     # needs nfpm + podman
```

## Layout

```
qregistry/
├── Cargo.toml                  workspace
├── README.md / CLAUDE.md / CHANGELOG.md / LICENSE
├── .github/workflows/ci.yml    build → package → container → deploy CT
├── crates/
│   ├── qregistry/              binary: CLI entry + UI server
│   ├── qregistry-core/         lib: AppConfig, RegistryEndpoint, Tenant(tier), User
│   └── qregistry-ui/           lib: axum server + admin pages
├── config/
│   └── qregistry.toml          appliance config (registries, tenants, users)
├── packaging/
│   ├── nfpm.yaml               rpm + deb spec
│   └── qregistry.service       systemd unit (distro/package case)
└── deploy/
    ├── provision-ct.sh         idempotent, data-safe pct create/update
    ├── qregistry.service       UI unit (in-CT)
    ├── rspace-registry@.service per-tier registry unit (in-CT)
    ├── Containerfile           Fedora-minimal appliance image
    ├── entrypoint.sh           container init (runs both registries + UI)
    └── Makefile                local rpm/deb/container build
```

## Roadmap

- **v0.1**: tiered fast/archive registries on separate drives, Fedora LXC via forcicd, rpm/deb/container, static tenant list.
- **v0.2**: per-repo rspacefs mounts (qregistry-managed), dynamic tenant/user CRUD from the UI.
- **v0.3**: offline mirror — pull-through cache + scheduled sync to a removable rspacefs partition.
- **v1.0**: Quay-parity (orgs, robot accounts, signing, scanning hooks, audit log). MikroTik Rose scratch-OCI flavor.

## License

MIT.
