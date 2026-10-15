#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "usage: $0 <package-directory> <output-directory>" >&2
    exit 2
fi

packages=$(realpath "$1")
output=$(realpath -m "$2")

mapfile -d '' debs < <(find "$packages" -type f -name 'cloneholio_*_aarch64.deb' -print0)
if [[ ${#debs[@]} -ne 1 ]]; then
    echo "expected exactly one aarch64 cloneholio package, found ${#debs[@]}" >&2
    exit 2
fi

rm -rf "$output"
mkdir -p "$output/pool/main/c/cloneholio" \
    "$output/dists/stable/main/binary-aarch64"
install -m 0644 "${debs[0]}" "$output/pool/main/c/cloneholio/"

cd "$output"
dpkg-scanpackages --arch aarch64 pool /dev/null \
    > dists/stable/main/binary-aarch64/Packages
gzip -9n -c dists/stable/main/binary-aarch64/Packages \
    > dists/stable/main/binary-aarch64/Packages.gz

apt-ftparchive \
    -o APT::FTPArchive::Release::Origin=cloneholio \
    -o APT::FTPArchive::Release::Label=cloneholio \
    -o APT::FTPArchive::Release::Suite=stable \
    -o APT::FTPArchive::Release::Codename=stable \
    -o APT::FTPArchive::Release::Architectures=aarch64 \
    -o APT::FTPArchive::Release::Components=main \
    -o APT::FTPArchive::Release::Description='cloneholio packages for Termux' \
    release dists/stable > dists/stable/Release
