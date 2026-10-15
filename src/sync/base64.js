// Vault files cross between the screens and the Rust side as base64 text: Android's IPC can't carry raw bytes,
// while plain text arrives intact on every platform.
export function toBase64(bytes) {
  let binary = "";
  // In slices, so a vault of several MB never overflows the call stack
  for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
  return btoa(binary);
}

export function fromBase64(text) {
  const binary = atob(text);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}
