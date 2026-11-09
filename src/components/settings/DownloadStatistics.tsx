import { useRef, useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { FEEDBACK_URL, readDownloadStats, type DownloadReport } from "../../settings/downloadStats";

export function DownloadStatistics() {
  const [report, setReport] = useState<DownloadReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const alive = useRef(true);
  useEffect(() => { alive.current = true; return () => { alive.current = false; }; }, []);
  async function load() {
    setLoading(true); setError("");
    try { const value = await readDownloadStats(report !== null); if (alive.current) setReport(value); }
    catch (err) { if (alive.current) setError(err instanceof Error ? err.message : "Could not load download statistics."); }
    finally { if (alive.current) setLoading(false); }
  }
  return <section className="mt-5 rounded-md border border-neutral-800 bg-neutral-950/60 p-3" aria-label="Download statistics">
    <h3 className="text-xs font-medium text-neutral-300">Download statistics</h3>
    <p className="mt-2 text-xs text-neutral-400">Public GitHub package downloads across all published stable releases. These are downloads, not unique users, computers or confirmed installations. Repeat, own and automated downloads are included. Platforms describe the package target, not the downloader's operating system.</p>
    <button type="button" disabled={loading} onClick={() => void load()}
      className="mt-2 rounded bg-neutral-800 px-2 py-1 text-xs text-white disabled:opacity-50">
      {loading ? "Loading statistics…" : report ? "Refresh statistics" : "Load statistics"}
    </button>
    {error && <p role="status" className="mt-2 text-xs text-amber-300">{error}{report ? " Showing previously loaded figures." : ""}</p>}
    {report && <>
      <p className="mt-2 text-xs text-neutral-400">Retrieved: {new Date(report.fetchedAt).toLocaleString("en-GB")} · Cached for up to 15 minutes.</p>
      <table className="mt-2 w-full text-left text-xs text-neutral-300">
        <caption className="text-left font-medium">All versions</caption>
        <thead><tr><th scope="col">Package platform</th><th scope="col" className="text-right">Downloads</th></tr></thead>
        <tbody>{Object.entries(report.platforms).map(([platform, count]) => <tr key={platform}><th scope="row" className="font-normal">{platform}</th><td className="text-right tabular-nums">{count.toLocaleString("en-GB")}</td></tr>)}</tbody>
        <tfoot><tr><th scope="row">Total</th><td className="text-right tabular-nums">{report.total.toLocaleString("en-GB")}</td></tr></tfoot>
      </table>
      <div className="mt-3 space-y-2">{report.releases.map(release => <details key={release.version} className="rounded border border-neutral-800 p-2">
        <summary className="cursor-pointer text-xs text-neutral-300">{release.version} — {release.total.toLocaleString("en-GB")} package downloads</summary>
        <p className="mt-2 text-xs text-neutral-400">{Object.entries(release.platforms).map(([platform, count]) => `${platform}: ${count.toLocaleString("en-GB")}`).join(" · ")}</p>
        {release.packages.length === 0 ? <p className="mt-2 text-xs text-neutral-500">No matching app packages in this release.</p> :
          <table className="mt-2 w-full text-left text-xs text-neutral-400"><thead><tr><th scope="col">Package</th><th scope="col" className="text-right">Downloads</th></tr></thead>
            <tbody>{release.packages.map(asset => <tr key={asset.name}><td className="py-1 pr-2"><span className="block">{asset.platform} · {asset.format}</span><span className="block break-all text-[10px] text-neutral-500">{asset.name}</span></td><td className="text-right tabular-nums">{asset.downloads.toLocaleString("en-GB")}</td></tr>)}</tbody></table>}
      </details>)}</div>
      {report.releases.length === 0 && <p className="mt-2 text-xs text-neutral-400">No published stable releases found.</p>}
    </>}
    <p className="mt-3 text-xs text-neutral-400">Using Trip Viewer? Please leave a comment or feedback in our GitHub project. Your experience helps improve the app. A GitHub account is required to post.</p>
    <button type="button" onClick={() => void openUrl(FEEDBACK_URL).catch(() => { if (alive.current) setError(`Please open ${FEEDBACK_URL} in your browser to leave feedback.`); })}
      className="mt-2 rounded bg-blue-600 px-2 py-1 text-xs text-white">Leave feedback on GitHub</button>
  </section>;
}
