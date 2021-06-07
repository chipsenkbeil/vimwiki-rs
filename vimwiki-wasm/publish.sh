#!/usr/bin/env sh

wasm-pack build -t bundler --out-dir pkg.bundle
wasm-pack build -t nodejs --out-dir pkg.node
npm publish
