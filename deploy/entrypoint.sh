#!/bin/sh
# Container entrypoint — runs the two tiered registries in the background
# and the admin UI in the foreground. (The LXC uses systemd instead.)
set -e

mkdir -p /var/lib/qregistry/repos/fast/registry \
         /var/lib/qregistry/repos/archive/registry

/usr/local/bin/rspace-registry --listen 0.0.0.0:5000 \
    --data /var/lib/qregistry/repos/fast/registry &
/usr/local/bin/rspace-registry --listen 0.0.0.0:5001 \
    --data /var/lib/qregistry/repos/archive/registry &

exec /usr/local/bin/qregistry --config /etc/qregistry/qregistry.toml
