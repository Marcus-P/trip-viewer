import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { LATEST_RELEASE, readLatestRelease, installerInstructions, type InstallTarget, type PublishedRelease } from "../../updates/releases";

export function ReleaseUpdates({ compact = false }: { compact?: boolean }) {
  const [version, setVersion] = useState("");
  const [target, setTarget] = useState<InstallTarget>("unsupported");
  const [release, setRelease] = useState<PublishedRelease | null>(null);
  const [checking, setChecking] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const [installed, compiledTarget] = await Promise.all([getVersion(), invoke<InstallTarget>("get_install_target")]);
        if (!cancelled) { setVersion(installed); setTarget(compiledTarget); }
        const latest = await readLatestRelease(installed, false, compiledTarget);
        if (!cancelled) setRelease(latest);
      } catch {
        if (!cancelled) setError("Could not check for updates. Try again or open Releases.");
      } finally {
        if (!cancelled) setChecking(false);
      }
    })();
    return () => { cancelled = true; };
  }, []);

  async function check() {
    setChecking(true);
    setError("");
    try {
      const [installed, compiledTarget] = await Promise.all([
        version || getVersion(), invoke<InstallTarget>("get_install_target")]);
      setVersion(installed);
      setTarget(compiledTarget);
      setRelease(await readLatestRelease(installed, true, compiledTarget));
    } catch {
      setError("Could not check for updates. Try again or open Releases.");
    } finally { setChecking(false); }
  }

  function open(url: string) {
    void openUrl(url).catch(() => setError("Could not open your browser. Visit " + LATEST_RELEASE));
  }

  const available = release?.relation === "newer";
  const download = release?.installerUrl ?? release?.pageUrl ?? LATEST_RELEASE;
  if (compact) {
    return (
      <button type="button" onClick={() => available ? open(download) : void check()}
        disabled={checking} title={error || (available ? "Download the published update" : "Check published releases")}
        className={"shrink-0 px-2 py-1.5 text-xs disabled:opacity-50 " +
          (available ? "text-emerald-400 hover:text-emerald-200" : "text-neutral-400 hover:text-neutral-200")}>
        {checking ? "Checking…" : available ? "Update v" + release.version : "Updates"}
      </button>
    );
  }
  return (
    <section className="rounded-md border border-neutral-800 bg-neutral-950/60 p-3" aria-label="Application updates">
      <h3 className="text-xs font-medium text-neutral-300">Application updates</h3>
      <p className="mt-1 text-xs text-neutral-400">
        {version ? "Installed v" + version + ". " : ""}
        {checking ? "Checking published releases…" : available ? "Version " + release.version + " is available." :
          release?.relation === "installedAhead" ? "This build is newer than the latest published release." :
          release ? "You have the latest published version." : ""}
      </p>
      {error && <p role="status" className="mt-1 text-xs text-amber-300">{error}</p>}
      <div className="mt-2 flex flex-wrap gap-2">
        <button type="button" onClick={() => void check()} disabled={checking}
          className="rounded bg-neutral-800 px-2 py-1 text-xs text-neutral-200 hover:bg-neutral-700 disabled:opacity-50">
          Check for updates
        </button>
        <button type="button" onClick={() => open(available ? download : LATEST_RELEASE)}
          className="rounded bg-blue-600 px-2 py-1 text-xs text-white hover:bg-blue-500">
          {available && release.installerUrl ? release.installerLabel : "Open Releases"}
        </button>
      </div>
      <p className="mt-2 text-[11px] text-neutral-500">
        {installerInstructions(target)}
      </p>
    </section>
  );
}
