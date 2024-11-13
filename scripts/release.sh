#!/usr/bin/env sh

version=v$(dasel -f crates/cli/Cargo.toml .package.version | xargs)
git push origin main
gh release create $version --title "$version"
