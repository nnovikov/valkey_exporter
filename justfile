default:
  @just --list

build:
  cross build --release

deb:
  cargo deb --no-build --target x86_64-unknown-linux-gnu

