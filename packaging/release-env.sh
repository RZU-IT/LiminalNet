release_separator=$(printf '\037')
if [ -n "${CARGO_ENCODED_RUSTFLAGS:-}" ]; then
    release_flags=$CARGO_ENCODED_RUSTFLAGS
elif [ -n "${RUSTFLAGS:-}" ]; then
    release_flags=$(printf '%s' "$RUSTFLAGS" | tr '[:space:]' '\037')
else
    release_flags=
fi
export CARGO_ENCODED_RUSTFLAGS="${release_flags:+$release_flags$release_separator}--remap-path-prefix=$HOME=build-home${release_separator}--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=cargo${release_separator}--remap-path-prefix=$(pwd)=liminalnet"
