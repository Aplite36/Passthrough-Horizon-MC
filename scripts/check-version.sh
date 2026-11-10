#!/usr/bin/env bash
# The mod's version is written in four places. Fail unless they agree, and
# unless they match the tag given as $1 (v0.1.0), when there is one.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)
mixin=$(sed -n 's/^version = "\(.*\)"$/\1/p' mixins/passthrough/horizon-mixin.toml | head -1)
code=$(sed -n 's/^pub const VERSION: &str = "\(.*\)";$/\1/p' mixins/passthrough/runtime.rs)
for v in "$mixin" "$code"; do
  [ "$v" = "$cargo" ] || { echo "versions differ: Cargo.toml $cargo, horizon-mixin.toml $mixin, runtime.rs $code" >&2; exit 1; }
done
grep -q "^## $cargo\$" CHANGELOG.md || { echo "CHANGELOG.md has no section for $cargo" >&2; exit 1; }
a=$(sed -n 's/.*"horizon_revision": "\(.*\)".*/\1/p' compatibility.json)
b=$(sed -n 's/^horizon_revision = "\(.*\)"$/\1/p' mixins/passthrough/horizon-mixin.toml)
[ "$a" = "$b" ] || { echo "horizon_revision differs: compatibility.json $a, horizon-mixin.toml $b" >&2; exit 1; }
if [ -n "${1:-}" ] && [ "$1" != "v$cargo" ]; then
  echo "tag $1 does not match version $cargo" >&2; exit 1
fi
echo "$cargo"
