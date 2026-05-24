# qregistry — Consolidated Design

Status: **confirmed model** (2026-05-24). Implemented in the config/UI
layer; storage placement + migration depend on sibling work (see §5).

## 0. Hierarchy (the mental model)

```
qregistry (one LXC appliance, one OCI service)
└── registry = release/layer group, pinned to a tier   e.g. 4.18.4 (fast)
    └── repo  = an rspacefs filesystem                  e.g. system, user, pvc
OCI path: <registry>/<repo>  →  4.18.4/system
```

- **Tier is chosen per registry** (the whole release/layer sits on fast or
  archive); repos inherit it.
- **Migration between tiers** moves a registry's repos and is performed
  **locally by rspacefs** (`rspacefs-pvc` capture/pivot); qregistry
  coordinates and repoints. Two tiers only: `fast` (NVMe), `archive` (ZFS).
- **PVC snapshots** are captured + pushed back **by the host** (rspacefs-pvc).
- Keys/secrets may ship **as config data-container artifacts** — TBD.

## 1. What qregistry is

A multi-tenant, **hierarchical** OCI registry **appliance** that stores
container images, **PVC data containers**, and **config artifacts** — with
**per-repo placement onto storage tiers** (separate physical drives) and
**registry-level migration between tiers**.

Delivered as a **Proxmox LXC** on a Fedora base, built and deployed by
**forcicd** (local Forgejo Actions), rolled by the **cigate** deploy gate
(image-based; nothing touches the hypervisor).

## 2. Packaging & deploy

- **Appliance image** (`deploy/Containerfile`, Fedora-minimal): one
  container running the qregistry UI + a single `rspace-registry` instance
  (entrypoint). Built + pushed to `forcicd.g8.lo:5000/qregistry` by CI.
- Also shipped: **rpm + deb** (the UI as a systemd service on a normal
  distro).
- **LXC bootstrap (one-time, hypervisor-side)**: CT **150**,
  `qregistry.g8.lo`, **192.168.8.50/24** on vmbr0/g8, stock
  `fedora-43-default` template, `unprivileged + nesting=1 + fuse=1`. Tier
  drives attached as mount points (see §4). Prepared for cigate pods.
- **Ongoing deploy (cigate)**: CI builds+pushes the image; the deploy job
  SSHes into the CT with a forced-command, repo-scoped key that runs only
  `cigate deploy <image>` (or `cigate pod up`). **No access to the Proxmox
  host.** Tracked: forcicd#1 (gate), forcicd#2 (pod-in-LXC).

## 3. Repos: hierarchical, multi-kind

- OCI repo names are slash-paths. A leading group + a leaf:
  `4.18.2/kernel`, `4.18.2/system`, `4.18.2/general`.
- **One registry service** serves all repos (not one instance per
  tier/repo). Per-repo storage placement is a storage-backend concern.
- **Artifact kinds** (`kind`): `image` (default), `pvc`, `config`.
  - PVC examples: a read-only `4.18.2/pvc/default` baseline; a writable
    **host-specific** repo that snapshots are pushed back to
    (`hosts/<host>/pvc/...`); config-data repos.
  - PVC media types per `enhancements/pvc-content-type.md`.

## 4. Storage tiers (first-class definitions) + migration

Tiers are **defined**, not hard-coded. Each tier names a mount backed by a
specific physical drive/class:

| Tier (example) | Class | Proxmox storage | Drive | CT mount |
|---|---|---|---|---|
| `nvme` | nvme | production-lvm | Samsung 990 4TB | `/var/lib/qregistry/tiers/nvme` |
| `sata` | ssd | services-lvm-thin | 2×870 QVO | `/var/lib/qregistry/tiers/sata` |
| `archive` | hdd | impulse1 (ZFS) | Seagate 8TB | `/var/lib/qregistry/tiers/archive` |

Proposed config shape:

```toml
[[tiers]]
name = "nvme"
class = "nvme"
mount = "/var/lib/qregistry/tiers/nvme"

[[tiers]]
name = "archive"
class = "hdd"
mount = "/var/lib/qregistry/tiers/archive"

[[repos]]
name = "4.18.2/kernel"
kind = "image"
tier = "nvme"

[[repos]]
name = "4.18.2/pvc/default"
kind = "pvc"
tier = "nvme"
public = true

[[repos]]
name = "hosts/node1/pvc/state"
kind = "pvc"
tier = "archive"          # host snapshots land on slow/cheap storage
```

A repo's storage root = `<tier.mount>/<repo-name>` (overridable). Placement
onto per-repo roots is delivered by **rspace_registry#1** (thread `repo`
through blob ops + `RepoRouter` with longest-prefix tier rules).

### Tier migration

Move a repo between tiers without losing data or (ideally) downtime:

1. Allocate the repo's root on the destination tier mount.
2. Copy/sync content (content-addressed → resumable, idempotent).
3. Repoint the registry's per-repo root to the destination.
4. Verify, then reclaim the source.

Granularity is the leaf repo. Trigger: manual (UI/API) now; policy-based
(age/last-pull) later. Needs rspace_registry per-repo-root repointing
(extends #1) + a qregistry orchestration verb. **Open question for #1:
should repoint be atomic/online in rspace_registry, or
drain-and-swap by qregistry?**

## 5. Component / dependency map

| Concern | Owner | Status |
|---|---|---|
| Per-repo storage roots, hierarchical, tier mounts | rspace_registry | **rspace_registry#1** (filed) |
| PVC artifact media types | rspace_registry/qregistry | `enhancements/pvc-content-type.md` |
| Tier migration (repoint root) | rspace_registry + qregistry | extends #1 (open) |
| cigate deploy gate (image-based) | forcicd | **forcicd#1** (done) |
| cigate pod-in-LXC | forcicd | **forcicd#2** (in progress) |
| Runner build (fuse-overlayfs/XDG) | forcicd | done — container build green |
| Appliance, UI, config model, tier defs, migration orchestration, deploy glue | **qregistry** | this repo |

## 6. qregistry config model (target)

- `[server]` — UI bind, data_dir.
- `[registry]` — single endpoint (url, listen).
- `[[tiers]]` — name, class, mount.
- `[[repos]]` — name (hierarchical), kind, tier, public, optional mount override.
- `[[users]]` — bcrypt, admin, push scopes.

(Current code: single `registry` ✓, hierarchical repo `name` + `group()` ✓,
`kind` ✓. **Pending this confirmation:** replace the `StorageTier` enum
with `[[tiers]]` definitions + `tier` as a name reference, and add
migration orchestration.)

## 7. Decisions & remaining TBDs

Resolved:
1. **Two tiers** — `fast` (NVMe) + `archive` (ZFS). No middle tier.
2. **Migration repoint** is done by **rspacefs locally** (capture/pivot);
   rspace_registry only repoints a repo's root (rspace_registry#1).
3. **Single appliance** — one OCI service + UI in one CT.
4. **Snapshots pushed by the PVC host** (rspacefs-pvc capture).

Still TBD:
- Host-snapshot PVC auth (per-host robot/scoped token).
- Keys/secrets distribution as config data-container artifacts.
- Cigate single-container vs pod for the appliance (leaning single).

## 8. Sibling state (rescanned 2026-05-24)

- **forcicd**: runner fixed at source — `--privileged + /dev/fuse`,
  `BUILDAH_ISOLATION=chroot`, per-job XDG; now **webhook-driven**. The
  plain `podman build` in CI works. cigate gate (forcicd#1) + pod verb
  (forcicd#2) shipped.
- **rspacefs**: has the **`rspacefs-pvc`** crate — `capture_layer`,
  `pivot_upper`, `PvcMount`/`PvcAccessMode`/`PvcLifecycle`. This is the
  host-snapshot + local tier-pivot machinery.
- **rspace_registry**: still digest-only blobs, MultiStore (replicate-and-
  pivot) only; per-repo storage roots **not yet implemented**
  (rspace_registry#1 open).
