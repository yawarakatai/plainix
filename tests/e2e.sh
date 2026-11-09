#!/usr/bin/env bash
# Drives the real CLI against Nix. It needs network access, so it cannot run
# inside `nix flake check`.
# Backticks below are literal parts of expected Plainix messages.
# shellcheck disable=SC2016
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
old_nixpkgs=9fbb54b33e91ee4ca368e35a78e0613c720600b3
plainix=$(nix build --no-link --print-out-paths "$repo#plainix")/bin/plainix
system=$(nix eval --impure --raw --expr builtins.currentSystem)
# Allows running against a checkout with uncommitted changes.
export NIX_CONFIG="allow-dirty-locks = true
${NIX_CONFIG:-}"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

fail() {
  echo "e2e: $1" >&2
  exit 1
}

new_project() {
  mkdir "$work/$1"
  cd "$work/$1"
  git init --quiet .
  "$plainix" init "${@:2}" >/dev/null
  # Test this checkout's library rather than the published one.
  sed -i.bak "s#github:yawarakatai/plainix#git+file://$repo?shallow=1#" flake.nix
  rm flake.nix.bak
  nix flake lock
}

expect_error() {
  printf '%s\n' "$1" >plainix.nix
  local output
  if output=$(nix eval --raw ".#devShells.$system.default.drvPath" 2>&1); then
    fail "accepted invalid config: $1"
  fi
  grep -qF "Plainix: $2" <<<"$output" || fail "expected '$2' from: $1, got: $output"
}

echo "e2e: every supported system evaluates"
new_project shared --with rust
for target in x86_64-linux aarch64-linux aarch64-darwin; do
  nix eval --raw ".#devShells.$target.default.drvPath" >/dev/null 2>&1 ||
    fail "generated environment does not evaluate on $target"
done

echo "e2e: update restores flake.lock when evaluation fails"
cp plainix.nix plainix.nix.good
nix flake lock --override-input nixpkgs "github:NixOS/nixpkgs/$old_nixpkgs" 2>/dev/null
pinned=$(cat flake.lock)
sed -i.bak 's/"cargo"/"not-a-real-package"/' plainix.nix
rm plainix.nix.bak
if output=$("$plainix" update 2>&1); then
  fail "update accepted a broken environment"
fi
grep -qF 'package `not-a-real-package` does not exist' <<<"$output" || fail "update hid the cause: $output"
[ "$(cat flake.lock)" = "$pinned" ] || fail "update did not restore flake.lock"

echo "e2e: update keeps a working result"
mv plainix.nix.good plainix.nix
"$plainix" update >/dev/null 2>&1 || fail "update failed on a working environment"
[ "$(cat flake.lock)" != "$pinned" ] || fail "update did not change flake.lock"

echo "e2e: invalid configuration reports Plainix errors"
expect_error '{ packages = [ "gitt" ]; }' 'package `gitt` does not exist'
expect_error '{ packages = [ "python3Packages" ]; }' '`python3Packages` is not a package'
expect_error '{ packages = [ "hello-unfree" ]; }' '`hello-unfree` is unfree'
expect_error '{ pakages = [ ]; }' '`plainix.nix` contains unknown field(s): pakages'

echo "e2e: presets limit generated systems"
if [[ "$system" == *-darwin ]]; then
  if "$plainix" init --with freecad >/dev/null 2>&1; then
    fail "macOS accepted a Linux-only preset"
  fi
else
  new_project linux-only --with freecad
  if grep -q aarch64-darwin plainix.nix; then
    fail "Linux-only preset declared aarch64-darwin"
  fi
fi

echo "e2e: passed"
