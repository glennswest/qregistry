#!/usr/bin/env bash
# provision-ct.sh — create/update the qregistry LXC on pve.g8.lo.
#
# Runs ON the Proxmox host. Expects a staging dir (default /tmp/qregistry-deploy)
# containing:
#   dist/<arch>/{qregistry,rspace-registry,rspacefs-mount}
#   config/qregistry.toml
#   deploy/{qregistry.service,rspace-registry@.service}
#
# Fedora base, systemd-supervised (no stormd). Services:
#   qregistry.service            — admin UI :8081
#   rspace-registry@fast.service — OCI :5000, data on NVMe mount
#   rspace-registry@archive.service — OCI :5001, data on ZFS mount
#
# Idempotent and DATA-SAFE: the persistent tier volumes (mp0 fast, mp1
# archive) are created only if absent and are NEVER recreated, so the
# rootfs/binaries can be rebuilt without touching repo data.
set -euo pipefail

# --- Deploy coordinates (overridable via env) ---
VMID="${VMID:-118}"
HOSTNAME_FQDN="${HOSTNAME_FQDN:-qregistry.g8.lo}"
IP_CIDR="${IP_CIDR:-192.168.8.50/24}"
GW="${GW:-192.168.8.1}"
BRIDGE="${BRIDGE:-vmbr0}"
TEMPLATE="${TEMPLATE:-local:vztmpl/fedora-43-default_20251224_amd64.tar.xz}"

ROOTFS_STORAGE="${ROOTFS_STORAGE:-local-lvm}"
ROOTFS_GB="${ROOTFS_GB:-20}"

FAST_STORAGE="${FAST_STORAGE:-production-lvm}"
FAST_GB="${FAST_GB:-500}"
ARCHIVE_STORAGE="${ARCHIVE_STORAGE:-impulse1}"
ARCHIVE_GB="${ARCHIVE_GB:-2048}"

MEMORY_MB="${MEMORY_MB:-16384}"
CORES="${CORES:-4}"

ARCH="${ARCH:-x86_64}"
STAGE="${STAGE:-/tmp/qregistry-deploy}"

REPOS_ROOT="/var/lib/qregistry/repos"
FAST_MP="${REPOS_ROOT}/fast"
ARCHIVE_MP="${REPOS_ROOT}/archive"

log() { echo ">>> $*"; }

# --- 1. Create CT if it does not exist ---
if pct status "${VMID}" >/dev/null 2>&1; then
    log "CT ${VMID} exists — updating in place (data volumes preserved)."
else
    log "Creating CT ${VMID} (${HOSTNAME_FQDN}) from ${TEMPLATE}"
    pct create "${VMID}" "${TEMPLATE}" \
        --hostname "${HOSTNAME_FQDN}" \
        --cores "${CORES}" \
        --memory "${MEMORY_MB}" \
        --ostype fedora \
        --unprivileged 1 \
        --features nesting=1,fuse=1 \
        --rootfs "${ROOTFS_STORAGE}:${ROOTFS_GB}" \
        --net0 "name=eth0,bridge=${BRIDGE},ip=${IP_CIDR},gw=${GW}" \
        --onboot 1
fi

# --- 2. Attach tier volumes only if absent (DATA-SAFE) ---
if pct config "${VMID}" | grep -q '^mp0:'; then
    log "mp0 (fast) already attached — leaving data intact."
else
    log "Creating fast tier volume: ${FAST_STORAGE} ${FAST_GB}G -> ${FAST_MP}"
    pct set "${VMID}" -mp0 "${FAST_STORAGE}:${FAST_GB},mp=${FAST_MP}"
fi

if pct config "${VMID}" | grep -q '^mp1:'; then
    log "mp1 (archive) already attached — leaving data intact."
else
    log "Creating archive tier volume: ${ARCHIVE_STORAGE} ${ARCHIVE_GB}G -> ${ARCHIVE_MP}"
    pct set "${VMID}" -mp1 "${ARCHIVE_STORAGE}:${ARCHIVE_GB},mp=${ARCHIVE_MP}"
fi

# --- 3. Start CT ---
if [ "$(pct status "${VMID}" | awk '{print $2}')" != "running" ]; then
    log "Starting CT ${VMID}"
    pct start "${VMID}"
    sleep 5
fi

# --- 4. Push binaries + config + units ---
log "Pushing binaries"
pct exec "${VMID}" -- mkdir -p /usr/local/bin /etc/qregistry \
    "${FAST_MP}/registry" "${ARCHIVE_MP}/registry" /run/rspacefs
for bin in qregistry rspace-registry rspacefs-mount; do
    pct push "${VMID}" "${STAGE}/dist/${ARCH}/${bin}" "/usr/local/bin/${bin}" --perms 0755
done

log "Pushing config + systemd units"
pct push "${VMID}" "${STAGE}/config/qregistry.toml" /etc/qregistry/qregistry.toml --perms 0644
pct push "${VMID}" "${STAGE}/deploy/qregistry.service" /etc/systemd/system/qregistry.service --perms 0644
pct push "${VMID}" "${STAGE}/deploy/rspace-registry@.service" /etc/systemd/system/rspace-registry@.service --perms 0644

# Per-tier listen-port drop-ins (archive on :5001).
pct exec "${VMID}" -- mkdir -p /etc/systemd/system/rspace-registry@archive.service.d
pct exec "${VMID}" -- sh -c 'printf "[Service]\nEnvironment=LISTEN=0.0.0.0:5001\n" > /etc/systemd/system/rspace-registry@archive.service.d/listen.conf'

# --- 5. Enable + (re)start services ---
log "Enabling services"
pct exec "${VMID}" -- systemctl daemon-reload
pct exec "${VMID}" -- systemctl enable --now rspace-registry@fast.service
pct exec "${VMID}" -- systemctl enable --now rspace-registry@archive.service
pct exec "${VMID}" -- systemctl enable --now qregistry.service

log "Done. CT ${VMID} (${HOSTNAME_FQDN}):"
IP="${IP_CIDR%/*}"
log "  admin UI       : http://${IP}:8081/"
log "  fast registry  : http://${IP}:5000/v2/"
log "  archive registry: http://${IP}:5001/v2/"
