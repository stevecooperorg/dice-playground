// Whole-document handoff: same-origin published source, bounded before storage.
// Keep this key in sync with src/ui/playground_handoff.rs.
// Published sources are literate: the 256 KiB bound matches MAX_LITERATE_BYTES
// in src/engine/literate/mod.rs (legacy scripts have a separate 64 KiB bound).
"use strict";
document.addEventListener("click", async (event) => {
  const link = event.target.closest("a.open-dice-document");
  if (!link) return;
  event.preventDefault();
  const status = link.parentElement.querySelector('[role="status"]');
  try {
    const url = new URL(link.href, location.href);
    if (url.origin !== location.origin || !url.pathname.endsWith(".dice")) {
      throw new Error("Invalid published source URL");
    }
    status.textContent = " Loading…";
    const response = await fetch(url, { redirect: "error" });
    if (!response.ok) throw new Error(`Source request failed (${response.status})`);
    const content = await response.text();
    if (new TextEncoder().encode(content).length > 256 * 1024) {
      throw new Error("Source exceeds the 256 KiB handoff limit");
    }
    const filename = decodeURIComponent(url.pathname.split("/").pop());
    localStorage.setItem("dice_playground_pending_load", JSON.stringify({ content, filename }));
    location.assign("/");
  } catch (error) {
    status.textContent = ` Could not open: ${error.message}. Download the source and open it in the playground instead.`;
  }
});
