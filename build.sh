#!/usr/bin/env bash
set -o pipefail
set -o errexit

SCRIPT_DIR=$(dirname $(readlink -f $0))
HDF5_RUST_DIR="$SCRIPT_DIR/hdf5-rust"
NETCDF_RUST_DIR="$SCRIPT_DIR/netcdf-rust"

HDF5_RUST_BRANCH="v0.7.1-emscripten"
NETCDF_RUST_BRANCH="v0.6.1-emscripten"

HDF5_SOURCE_DIR="hdf5-src/ext/hdf5"
HDF5_NATIVE_BUILD_DIR="build"

BUILD_TYPE="full"

if [ "$1" = "fast" ]; then
	BUILD_TYPE="fast"
fi

if [ "$BUILD_TYPE" = "full" ]; then
	echo "==> Making sure we have the right branches checked out"
	cd "$HDF5_RUST_DIR" && git checkout "$HDF5_RUST_BRANCH"
	cd "$NETCDF_RUST_DIR" && git checkout "$NETCDF_RUST_BRANCH"
	cd "$SCRIPT_DIR"

	echo "==> Running hdf5 pregen"
	cd "$HDF5_RUST_DIR/$HDF5_SOURCE_DIR"
	mkdir "$HDF5_NATIVE_BUILD_DIR" && cd "$HDF5_NATIVE_BUILD_DIR"
	cmake .. "-DHDF5_BUILDTOOLS=OFF" "-DBUILD_SHARED_LIBS=OFF" "-DHDF5_BUILD_EXAMPLES=OFF" "-DBUILD_TESTING=OFF"
	make -j$(nproc)
	cd "$SCRIPT_DIR"

	echo "==> Adding wasm32 target"
	rustup target add wasm32-unknown-emscripten
fi

echo "==> Building wrapper library"
cd "visualize"
cargo build --target wasm32-unknown-emscripten --release
