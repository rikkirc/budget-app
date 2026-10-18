// Builds vault-core for the browser: compiles vault-web/ to WebAssembly and writes the JavaScript that loads it into
// src/sync/vault-web-pkg/ (not committed; `npm run build:web` and `npm run dev:web` run this first).
// Needs the wasm32 Rust target and the wasm-bindgen CLI at the exact version vault-web/Cargo.toml pins:
//   rustup target add wasm32-unknown-unknown
//   cargo install wasm-bindgen-cli --version 0.2.129 --locked
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const root = fileURLToPath(new URL("..", import.meta.url));
const WASM_BINDGEN = "0.2.129";
const run = (cmd, args) => execFileSync(cmd, args, { cwd: root, stdio: "inherit" });

let version = "";
try { version = execFileSync("wasm-bindgen", ["--version"], { encoding: "utf8" }); } catch { /* not installed */ }
if (!version.includes(WASM_BINDGEN)) {
  console.error(`wasm-bindgen ${WASM_BINDGEN} is needed. Install it with:\n  cargo install wasm-bindgen-cli --version ${WASM_BINDGEN} --locked`);
  process.exit(1);
}

run("cargo", ["build", "--release", "--target", "wasm32-unknown-unknown", "--manifest-path", "vault-web/Cargo.toml"]);
run("wasm-bindgen", [
  "--target", "web", "--out-dir", join("src", "sync", "vault-web-pkg"), "--no-typescript",
  join("vault-web", "target", "wasm32-unknown-unknown", "release", "vault_web.wasm"),
]);
