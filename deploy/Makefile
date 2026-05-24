# qregistry appliance build/deploy.
#
# Per /Volumes/minihome/gwest/CLAUDE.md MikroTik Rose rules:
#   - Cross-compile binaries locally for ARM64 Linux
#   - Build container with `scratch` base (stormdbase qualifies)
#   - Always use `podman` (not docker)
#   - Push to the local registry (registry.gt.lo:5000) that is part of mkube
#   - mkube pushes to GHCR + updates the running container

ARCH         ?= arm64
RUST_TARGET  := aarch64-unknown-linux-musl
REGISTRY     := registry.gt.lo:5000
IMAGE        := $(REGISTRY)/qregistry
TAG          ?= latest

# Sibling project paths (do NOT build these — they own their own deploy)
SIBLING_REGISTRY := ../../rspace_registry
SIBLING_FS       := ../../rspacefs

# Pre-built sibling binaries (cross-compiled by their own Makefiles).
# This project pulls them in but never rebuilds them.
SIBLING_REGISTRY_BIN := $(SIBLING_REGISTRY)/target/$(RUST_TARGET)/release/rspace-registry
SIBLING_FS_BIN       := $(SIBLING_FS)/target/$(RUST_TARGET)/release/rspacefs-mount

# Cross-compiled qregistry binary
QREGISTRY_BIN := ../target/$(RUST_TARGET)/release/qregistry

.PHONY: all qregistry image push clean check-siblings

all: image

# Cross-compile the qregistry binary (musl static, target arch).
qregistry:
	cd .. && cargo build --release --target $(RUST_TARGET) -p qregistry

# Verify sibling binaries exist — qregistry does not build them.
check-siblings:
	@test -f $(SIBLING_REGISTRY_BIN) || { \
		echo "MISSING: $(SIBLING_REGISTRY_BIN)"; \
		echo "Build rspace_registry first: (cd $(SIBLING_REGISTRY) && cargo build --release --target $(RUST_TARGET))"; \
		exit 1; }
	@test -f $(SIBLING_FS_BIN) || { \
		echo "MISSING: $(SIBLING_FS_BIN)"; \
		echo "Build rspacefs first: (cd $(SIBLING_FS) && cargo build --release --target $(RUST_TARGET))"; \
		exit 1; }

# Stage all artifacts in deploy/build/ then podman build from there.
image: qregistry check-siblings
	mkdir -p build
	cp $(QREGISTRY_BIN)        build/qregistry
	cp $(SIBLING_REGISTRY_BIN) build/rspace-registry
	cp $(SIBLING_FS_BIN)       build/rspacefs-mount
	cp -r ../config            build/config
	cp Containerfile           build/Containerfile
	cd build && podman build --platform linux/$(ARCH) -t $(IMAGE):$(TAG) -f Containerfile .

push: image
	podman push $(IMAGE):$(TAG)

clean:
	rm -rf build
