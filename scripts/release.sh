#!/usr/bin/env sh

version=v$(dasel -f Cargo.toml .package.version | xargs)
git push origin main
gh release create $version --title "$version"
