# qregistry packaging Makefile.
#
# Produces the four artifact types the project ships:
#   make binaries   — cross/native build of qregistry (siblings via CI)
#   make rpm        — nfpm rpm into dist/
#   make deb        — nfpm deb into dist/
#   make container  — podman appliance image
#
# In CI (forcicd Fedora x86_64 runner) the siblings are cloned + built
# natively; locally only qregistry is built here. See .github/workflows.

VERSION   ?= $(shell grep -m1 '^version' ../Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')
ARCH      ?= x86_64
PKG_ARCH  ?= amd64
REGISTRY  ?= forcicd.g8.lo:5000
IMAGE     := $(REGISTRY)/qregistry
TAG       ?= latest
DIST      := ../dist/$(ARCH)

.PHONY: all binaries rpm deb container clean

all: rpm deb container

# Native build of the qregistry binary into dist/<arch>/.
binaries:
	cd .. && cargo build --release -p qregistry
	mkdir -p $(DIST)
	cp ../target/release/qregistry $(DIST)/qregistry

rpm:
	cd .. && VERSION=$(VERSION) PKG_ARCH=$(PKG_ARCH) nfpm package -f packaging/nfpm.yaml -p rpm -t dist/

deb:
	cd .. && VERSION=$(VERSION) PKG_ARCH=$(PKG_ARCH) nfpm package -f packaging/nfpm.yaml -p deb -t dist/

container:
	cd .. && podman build --build-arg ARCH=$(ARCH) -f deploy/Containerfile -t $(IMAGE):$(TAG) -t $(IMAGE):$(VERSION) .

push: container
	podman push $(IMAGE):$(TAG)
	podman push $(IMAGE):$(VERSION)

clean:
	rm -rf ../dist
