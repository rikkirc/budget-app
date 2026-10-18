# vault-web

Vault's business logic (`vault-core`) compiled to WebAssembly for the web app (PWA). The browser runs exactly the
same rules as the desktop and Android apps, so every device turns the shared list of changes into identical balances.

- `src/lib.rs` — `WebVault`: open (from saved bytes or empty), `apply` changes, `call` reads, `snapshot`, `restore`.
- `src/sync/webReplica.js` uses it, keeping the vault in memory and saving it to IndexedDB after every change.

## Build

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked   # must match Cargo.toml
npm run build:wasm   # writes src/sync/vault-web-pkg/ (not committed)
npm run build:web    # the web app, in dist-web/
npm run dev:web      # the web app on a dev server
```

## Tests

```sh
# vault-core's tests, run inside WebAssembly (Node)
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner \
  cargo test --manifest-path vault-core/Cargo.toml --target wasm32-unknown-unknown

# browser and desktop replicas side by side (src/sync/webReplica.test.js)
npm run build:wasm && npm test
```
