#!/bin/sh
# nfpm postinstall — reload systemd so the unit is visible. We do NOT
# auto-enable/start: the operator decides when the registry comes up
# (and must set a real admin password first).
set -e
if command -v systemctl >/dev/null 2>&1; then
    systemctl daemon-reload || true
fi
echo "qregistry installed. Edit /etc/qregistry/qregistry.toml, then:"
echo "  systemctl enable --now qregistry"
exit 0
