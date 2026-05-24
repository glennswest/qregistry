# Enhancement: PVC / Data-Container Content Type

Author: rspacefs side, requested 2026-05-24.
Target: qregistry, likely landing in v0.3 alongside offline-mirror work.

## Why

A K8s / OpenShift cluster needs more than container images served by its
registry. The boot path also needs:

- **PVC data containers** — pre-baked persistent-volume contents (database
  seed data, model weights, license keys, certificate bundles, prebuilt
  filesystems). Today these live in object storage and are pulled by an
  initContainer; the round-trip is slow, the format is ad-hoc, and there's
  no signed manifest.

If qregistry serves them as **content-addressed OCI artifacts** alongside
images, the same auth, GC, mirror, signing, and tenant model applies for
free. A node consuming PVCs from qregistry uses the same `/v2/<tenant>/<repo>/manifests/<ref>`
contract it already speaks for images — just a different `mediaType`.

## What

Add a "PVC" artifact type to qregistry. Concretely:

### New media type

```
application/vnd.qregistry.pvc.v1+json   (manifest)
application/vnd.qregistry.pvc-data.v1.tar+zstd   (data blob)
```

A PVC manifest is the OCI artifact-manifest shape (per OCI 1.1 §
"artifact" / OCI Reference Types), with one or more data blob layers
that, when concatenated and untarred, give the directory tree the PVC
should contain. The manifest carries:

```jsonc
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.oci.artifact.manifest.v1+json",
  "artifactType": "application/vnd.qregistry.pvc.v1+json",
  "blobs": [
    {
      "mediaType": "application/vnd.qregistry.pvc-data.v1.tar+zstd",
      "digest": "sha256:…",
      "size": …
    }
  ],
  "annotations": {
    "qregistry.pvc.size":         "10Gi",
    "qregistry.pvc.fs-type":      "ext4",      // or "tar" for content-only
    "qregistry.pvc.access-modes": "ReadWriteOnce",
    "qregistry.pvc.lifecycle":    "ephemeral-then-persistent",
    "qregistry.pvc.description":  "OpenShift built-in registry seed"
  }
}
```

`qregistry.pvc.lifecycle` is the load-bearing annotation for the
boot-time use case — see `rspaced/enhancements/multi-registry-boot.md`
for the consumer semantics. Three values:

| Value | Meaning |
|---|---|
| `persistent` | Restore into a real PVC backing on disk. Default. |
| `ephemeral` | Stage into tmpfs only. Discard on reboot. |
| `ephemeral-then-persistent` | Stage into tmpfs at boot; promote (snapshot to disk) when the consumer asks via control surface. |

### Storage layout (per-tenant)

qregistry already gives each tenant its own rspacefs filesystem at
`/var/lib/qregistry/repos/<tenant>/`. PVCs are stored exactly like
images — content-addressed blobs under `blobs/sha256/…`, manifest
under `manifests/<repo>/{tags,digests}/`. **No new directories.** The
only thing different is the `artifactType` field in the manifest.

Tenants can freely mix image repos and PVC repos. By convention:

```
<tenant>/images/<image-name>:<tag>      # OCI image
<tenant>/pvcs/<pvc-name>:<rev>          # PVC
```

…but the registry doesn't enforce naming; it's a convention for humans.

### UI

`qregistry-ui` gains a per-tenant PVC list view: name, revision, size,
lifecycle, last-modified, link to manifest digest. Push from CLI; UI is
read-only in v0.

### CLI: pushing a PVC

The expectation is a small `qregctl` tool (sibling of `rspacefs`/CLI
families) that takes a directory and pushes it:

```bash
qregctl push \
  --tenant openshift \
  --repo pvcs/registry-seed \
  --tag v1 \
  --lifecycle ephemeral-then-persistent \
  --size 10Gi \
  ./seed-tree/
```

…but in v0 we can also accept manually-crafted `oras push` invocations
with the right media types, since the registry is OCI-conformant.

## What stays out of scope (for this enhancement)

- **No mount/loop logic in qregistry.** The registry is bytes only.
  Mounting the PVC content on a node is the boot agent's job (rspaced).
- **No size enforcement on push.** Annotation-only.
- **No content-aware GC.** PVC blobs are GC'd by the same mark-and-sweep
  as image blobs (anything referenced by any manifest is reachable).

## Acceptance

- [ ] A directory can be pushed via `oras push` with the new media types
- [ ] `oras pull` round-trips the bytes exactly
- [ ] `qregistry-ui` lists PVC manifests under their tenant
- [ ] An rspaced consumer (per `rspaced/enhancements/multi-registry-boot.md`)
  can fetch a PVC manifest, pull the data blob, and stage it.

## Open questions

1. **Naming convention enforced or convention-only?** Recommend convention
   so the registry doesn't have to special-case `pvcs/` vs `images/`.
2. **Multi-blob PVCs (chunked)?** v0 = single blob. Chunking is a v1
   optimisation if blob sizes get too large for atomic upload.
3. **Encrypted PVCs?** v0 stores cleartext blobs. Encryption is a tenant
   concern (encrypt before push); a future enhancement could add
   server-side keys.
