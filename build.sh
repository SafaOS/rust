#!/bin/bash
# Script to build libstd and prepare SafaOS's toolchain
export TARGET_DIR="$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-safaos"
export TARGET_DIR_LIB="$TARGET_DIR/lib"
mkdir -p $TARGET_DIR_LIB

cp target.json $TARGET_DIR

export CARGO_PROFILE_RELEASE_DEBUG=0
export CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true
export RUSTC_BOOTSTRAP=1
export RUSTFLAGS="-Cforce-unwind-tables=yes -Cembed-bitcode=yes -Zforce-unstable-if-unmarked"
export __CARGO_DEFAULT_LIB_METADATA="stablestd"
export RUST_COMPILER_RT_ROOT="$(pwd)/src/llvm-project/compiler-rt"

cargo build --target x86_64-unknown-safaos -Zbinary-dep-depinfo \
          --release \
          --features "panic-unwind compiler-builtins-c compiler-builtins-mem" \
          --manifest-path "library/sysroot/Cargo.toml"

rm "$TARGET_DIR_LIB/*.rlib"
cp library/target/x86_64-unknown-safaos/release/deps/*.rlib $TARGET_DIR_LIB
rm -rf x86_64-unknown-safaos-toolchain
cp -r $TARGET_DIR x86_64-unknown-safaos-toolchain
