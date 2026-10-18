import { useEffect, useState } from "react";
import { registerSW } from "virtual:pwa-register";

// The web app only: a new version downloads in the background, then this bar asks before switching to it, so nobody
// loses what they were typing. It also checks for a new version every hour while the app stays open.
export default function UpdateBanner() {
  const [applyUpdate, setApplyUpdate] = useState(null);

  useEffect(() => {
    const update = registerSW({
      onNeedRefresh: () => setApplyUpdate(() => () => update(true)),
      onRegisteredSW: (_url, registration) => {
        if (registration) setInterval(() => registration.update().catch(() => {}), 60 * 60 * 1000);
      },
    });
  }, []);

  if (!applyUpdate) return null;
  return (
    <div role="status" style={{ position: "fixed", left: "10px", right: "10px", bottom: "max(10px, env(safe-area-inset-bottom))", zIndex: 1000, display: "flex", alignItems: "center", justifyContent: "space-between", gap: "10px", flexWrap: "wrap", padding: "12px 15px", backgroundColor: "#1e3a8a", color: "white", borderRadius: "8px", boxShadow: "0 4px 12px rgba(0,0,0,0.25)", fontSize: "14px" }}>
      <span>A new version of Vault is ready.</span>
      <span style={{ display: "flex", gap: "8px" }}>
        <button onClick={() => setApplyUpdate(null)} style={{ padding: "8px 12px", backgroundColor: "transparent", color: "white", border: "1px solid #93c5fd", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>Later</button>
        <button onClick={applyUpdate} style={{ padding: "8px 14px", backgroundColor: "white", color: "#1e3a8a", border: "none", borderRadius: "5px", cursor: "pointer", fontWeight: "bold" }}>Reload</button>
      </span>
    </div>
  );
}
