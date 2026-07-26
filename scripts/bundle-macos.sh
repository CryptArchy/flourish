#!/usr/bin/env bash
#
# Assembles Flourish.app.
#
#   scripts/bundle-macos.sh [--universal] [--output DIR]
#
# --universal builds for both Apple Silicon and Intel and lipos them together,
# which is what a downloadable build should be. Without it you get a binary for
# the host architecture only, which is faster and fine for local use.
#
# Signing: the bundle is ad-hoc signed so macOS will run it locally. That is
# NOT enough for distribution -- see "Shipping it to other people" in
# packaging/macos/README.md for the Developer ID and notarization path.

set -euo pipefail

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly APP_NAME="Flourish"
readonly BINARY_NAME="flourish"

universal=false
output_dir="${PROJECT_ROOT}/target/bundle"

while [[ $# -gt 0 ]]; do
	case "$1" in
	--universal)
		universal=true
		shift
		;;
	--output)
		output_dir="${2:?--output needs a directory}"
		shift 2
		;;
	-h | --help)
		sed -n '3,14p' "${BASH_SOURCE[0]}" | sed 's|^# \{0,1\}||'
		exit 0
		;;
	*)
		echo "unknown argument: $1" >&2
		exit 2
		;;
	esac
done

if [[ "$(uname -s)" != "Darwin" ]]; then
	echo "error: macOS bundles can only be built on macOS (iconutil and codesign are required)" >&2
	exit 1
fi

cd "${PROJECT_ROOT}"

# Single source of truth for the version: the manifest. Parsed from the
# [package] section only, so a dependency's version can never be picked up.
version="$(awk '/^\[package\]/{p=1;next} /^\[/{p=0} p && /^version[[:space:]]*=/{gsub(/[";]/,"");print $3;exit}' Cargo.toml)"
if [[ -z "${version}" ]]; then
	echo "error: could not read the package version from Cargo.toml" >&2
	exit 1
fi
echo "==> Bundling ${APP_NAME} ${version}"

# --- Binary ------------------------------------------------------------------

if [[ "${universal}" == true ]]; then
	echo "==> Building universal release binary"
	for target in aarch64-apple-darwin x86_64-apple-darwin; do
		if ! rustup target list --installed | grep -qx "${target}"; then
			echo "    installing missing target ${target}"
			rustup target add "${target}"
		fi
		cargo build --locked --release --target "${target}"
	done
	binary="${PROJECT_ROOT}/target/universal-flourish"
	lipo -create -output "${binary}" \
		"target/aarch64-apple-darwin/release/${BINARY_NAME}" \
		"target/x86_64-apple-darwin/release/${BINARY_NAME}"
else
	echo "==> Building release binary for the host architecture"
	cargo build --locked --release
	binary="${PROJECT_ROOT}/target/release/${BINARY_NAME}"
fi

# --- Icon --------------------------------------------------------------------

echo "==> Rendering the icon"
iconset="${PROJECT_ROOT}/target/${APP_NAME}.iconset"
rm -rf "${iconset}"
cargo run --locked --release --quiet --example iconset -- "${iconset}" >/dev/null
iconutil --convert icns --output "${PROJECT_ROOT}/target/${APP_NAME}.icns" "${iconset}"

# --- Assemble ----------------------------------------------------------------

app="${output_dir}/${APP_NAME}.app"
echo "==> Assembling ${app}"
rm -rf "${app}"
mkdir -p "${app}/Contents/MacOS" "${app}/Contents/Resources"

install -m 0755 "${binary}" "${app}/Contents/MacOS/${BINARY_NAME}"
install -m 0644 "${PROJECT_ROOT}/target/${APP_NAME}.icns" "${app}/Contents/Resources/${APP_NAME}.icns"
sed "s|__VERSION__|${version}|g" \
	"${PROJECT_ROOT}/packaging/macos/Info.plist" \
	>"${app}/Contents/Info.plist"

# APPL + the four-byte signature; some older Finder paths still look for it.
printf 'APPL????' >"${app}/Contents/PkgInfo"

plutil -lint "${app}/Contents/Info.plist" >/dev/null

# --- Sign --------------------------------------------------------------------

# CODESIGN_IDENTITY lets a release build use a real Developer ID; otherwise
# ad-hoc ("-") so the bundle at least runs on the machine that built it.
identity="${CODESIGN_IDENTITY:--}"
echo "==> Signing with identity: ${identity}"
codesign --force --deep --options runtime --sign "${identity}" "${app}"
codesign --verify --deep --strict "${app}"

echo
echo "Built ${app}"
echo "  architectures: $(lipo -archs "${app}/Contents/MacOS/${BINARY_NAME}")"
echo "  version:       ${version}"
if [[ "${identity}" == "-" ]]; then
	echo
	echo "Ad-hoc signed. It will run here, but other machines will refuse it"
	echo "until it is signed with a Developer ID and notarized."
fi
