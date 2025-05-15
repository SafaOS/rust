#!/bin/bash
# Script to build libstd and prepare SafaOS's toolchain
set -eu
export VERSION="1.86.0"
export ARCH="x86_64"

for arg in "$@"; do
    case $arg in
        -a|--arch)
            ARCH="$2"
            export CROSS_COMPILE="aarch64-linux-gnu"
            # FIXME:
            break
            ;;
        *)
            echo "Unknown argument: $arg"
            exit 1
            ;;
    esac
    shift
done
echo "Building for arch $ARCH, NOTE THAT to compile aarch64 you need aarch64-linux-gnu-gcc, for now it is only designed for cross compiling"

export TARGET_DIR="$(rustc "+$VERSION" --print sysroot)/lib/rustlib/$ARCH-unknown-safaos"
export TARGET_DIR_LIB="$TARGET_DIR/lib"
mkdir -p $TARGET_DIR_LIB

cp "target-$ARCH.json" "$TARGET_DIR/target.json"

export CARGO_PROFILE_RELEASE_DEBUG=0
export CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true
export RUSTC_BOOTSTRAP=1
export RUSTFLAGS="-Cforce-unwind-tables=yes -Cembed-bitcode=yes -Zforce-unstable-if-unmarked"
export __CARGO_DEFAULT_LIB_METADATA="stablestd"
export RUST_COMPILER_RT_ROOT="$(pwd)/src/llvm-project/compiler-rt"

cargo "+$VERSION" build --target "$ARCH-unknown-safaos" -Zbinary-dep-depinfo \
          --release \
          --features "compiler-builtins-c compiler-builtins-mem" \
          --manifest-path "library/sysroot/Cargo.toml"

rm -f $TARGET_DIR_LIB/*.rlib
cp library/target/$ARCH-unknown-safaos/release/deps/*.rlib $TARGET_DIR_LIB
rm -rf "$ARCH-unknown-safaos-toolchain"
cp -r $TARGET_DIR "$ARCH-unknown-safaos-toolchain"
