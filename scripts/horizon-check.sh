#!/usr/bin/env bash
# Apply the mod to Horizon's source and compile-check the game, or package the
# .hnmod, with Horizon's own tool from horizon-rust/horizon-mods.
#   scripts/horizon-check.sh check            the commit in compatibility.json
#   scripts/horizon-check.sh check latest     Horizon's newest Nightly
#   scripts/horizon-check.sh package          .horizon-tools/native/target/*.hnmod
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
# The commit of horizon-mods whose tool this was tested with.
tools_rev=a4b09b49aa4eedb204972773fad2c02485961e54
tools="$root/.horizon-tools"
if [ ! -f "$tools/native/tools/native.py" ]; then
  rm -rf "$tools"
  git init -q "$tools"
  git -C "$tools" remote add origin https://github.com/horizon-rust/horizon-mods
  git -C "$tools" sparse-checkout set native/tools
  git -C "$tools" fetch -q --depth 1 origin "$tools_rev"
  git -C "$tools" checkout -q FETCH_HEAD
fi
action=${1:-check}
horizon=${2:-}
exec python3 "$tools/native/tools/native.py" "$action" "$root" ${horizon:+--horizon "$horizon"}
