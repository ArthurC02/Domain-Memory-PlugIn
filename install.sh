#!/bin/sh
# Installs the Domain Memory executable of one release, after checking it
# against that release's SHA256SUMS.
#
#   curl -fsSL https://github.com/ArthurC02/Domain-Memory-PlugIn/releases/latest/download/install.sh | sh
#
# Options, as a flag or in the environment:
#   --version <tag>  DOMAIN_MEMORY_VERSION      the release (default: the latest)
#   --to <dir>       DOMAIN_MEMORY_INSTALL_DIR  where it lands (default: ~/.local/bin)
# so a pinned install reads: curl -fsSL <url> | DOMAIN_MEMORY_VERSION=v6.6.0 sh
# DOMAIN_MEMORY_DOWNLOAD_URL replaces the release's download address, for a mirror.
set -eu

version=${DOMAIN_MEMORY_VERSION:-latest}
to=${DOMAIN_MEMORY_INSTALL_DIR:-$HOME/.local/bin}
while [ $# -gt 0 ]; do
  case $1 in
    --version) version=$2; shift 2 ;;
    --to) to=$2; shift 2 ;;
    *) echo "install.sh: unknown argument $1; it takes --version <tag> and --to <dir>" >&2; exit 2 ;;
  esac
done

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) asset=domain-memory-linux-x86_64 name=domain-memory ;;
  Darwin-*) asset=domain-memory-macos-universal name=domain-memory ;;
  MINGW*-x86_64|MSYS*-x86_64|CYGWIN*-x86_64) asset=domain-memory-windows-x86_64.exe name=domain-memory.exe ;;
  *)
    echo "install.sh: no release carries an executable for $(uname -s)-$(uname -m); build one from a clone with cargo build --release --locked --manifest-path engine/Cargo.toml" >&2
    exit 1 ;;
esac

releases=https://github.com/ArthurC02/Domain-Memory-PlugIn/releases
if [ "$version" = latest ]; then
  base=${DOMAIN_MEMORY_DOWNLOAD_URL:-$releases/latest/download}
else
  base=${DOMAIN_MEMORY_DOWNLOAD_URL:-$releases/download/$version}
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
curl -fsSL "$base/$asset" -o "$work/$asset"
curl -fsSL "$base/SHA256SUMS" -o "$work/SHA256SUMS"
wanted=$(awk -v asset="$asset" '$2 == asset || $2 == "*" asset { print $1 }' "$work/SHA256SUMS")
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$work/$asset" | cut -d' ' -f1)
else
  actual=$(shasum -a 256 "$work/$asset" | cut -d' ' -f1)
fi
if [ -z "$wanted" ] || [ "$wanted" != "$actual" ]; then
  echo "install.sh: $asset from $base does not match its SHA256SUMS entry (${wanted:-none}); nothing was installed" >&2
  exit 1
fi

mkdir -p "$to"
chmod +x "$work/$asset"
mv -f "$work/$asset" "$to/$name"
installed=$("$to/$name" --version 2>/dev/null) || installed="Domain Memory $version"
echo "Installed $installed at $to/$name"
case ":$PATH:" in
  *":$to:"*) echo "Try it: domain-memory readiness" ;;
  *) echo "$to is not on PATH; add it to run domain-memory by name and let git hooks find it: export PATH=\"$to:\$PATH\"" ;;
esac
