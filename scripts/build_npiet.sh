#!/usr/bin/env bash
# Build the reference interpreter without installing over the system npiet.
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
revision=843a5863a587e54155d3871f3d6ba735484e62f7
patch_file="$repo_root/scripts/patches/npiet-modern-c.patch"
prefix="$repo_root/target/npiet"
stamp="$prefix/build-stamp"
fingerprint=$(
  {
    cat "$patch_file" "${BASH_SOURCE[0]}"
    printf '%s\n' "${CC:-cc}" "${CFLAGS:-}" "${CPPFLAGS:-}" "${LDFLAGS:-}" "${LIBRARY_PATH:-}"
    uname -sm
  } | cksum
)

mkdir -p "$prefix/bin"
if [[ -x "$prefix/bin/npiet" && -f "$stamp" && "$(cat "$stamp")" == "$fingerprint" ]]; then
  echo "Using cached npiet ($revision)"
  exit 0
fi

# Preserve failed builds for inspection; all generated files live under target/.
source_dir=$(mktemp -d "$prefix/source.XXXXXX")
git -C "$source_dir" init -q
git -C "$source_dir" fetch --depth 1 https://github.com/gleitz/npiet.git "$revision"
git -C "$source_dir" checkout --detach -q FETCH_HEAD
git -C "$source_dir" apply --unidiff-zero "$patch_file"

(
  cd "$source_dir"
  ./configure
  if ! grep -q '^#define HAVE_PNG_H 1' config.h; then
    echo "npiet requires libpng development headers for the PNG test fixtures." >&2
    exit 1
  fi
  # Build only the interpreter, avoiding unrelated editor, yacc, and manpage tools.
  make npiet CFLAGS="${CFLAGS:--g -O2} -Wall -DHAVE_CONFIG_H ${CPPFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
  ./npiet "$repo_root/images/hw.png"
)
cp "$source_dir/npiet" "$prefix/bin/npiet"
printf '%s\n' "$fingerprint" > "$stamp"
echo
echo "Built npiet at $prefix/bin/npiet"
