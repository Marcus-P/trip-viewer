export const DOWNLOADS_API = "https://api.github.com/repos/Marcus-P/trip-viewer/releases?per_page=100";
export const FEEDBACK_URL = "https://github.com/Marcus-P/trip-viewer/issues/new";
export type Platform = "Ubuntu/Linux" | "Windows" | "macOS";
export interface PackageDownloads { name: string; platform: Platform; format: string; downloads: number }
export interface ReleaseDownloads { version: string; packages: PackageDownloads[]; total: number; platforms: Record<Platform, number> }
export interface DownloadReport { releases: ReleaseDownloads[]; total: number; platforms: Record<Platform, number>; fetchedAt: number }

function packageType(name: string): { platform: Platform; format: string } | null {
  if (name.endsWith("_Windows_x64_setup.exe")) return { platform: "Windows", format: "Installer" };
  if (name.endsWith("_macOS_universal_experimental.dmg")) return { platform: "macOS", format: "DMG (experimental)" };
  if (name.endsWith("_Ubuntu-24.04_amd64.deb")) return { platform: "Ubuntu/Linux", format: "DEB" };
  if (name.endsWith("_Ubuntu-24.04_amd64.AppImage")) return { platform: "Ubuntu/Linux", format: "AppImage" };
  if (name.endsWith("_Ubuntu-24.04_bundle.tar.gz")) return { platform: "Ubuntu/Linux", format: "Bundle" };
  return null;
}

export function summarizeDownloads(pages: unknown[], fetchedAt = Date.now()): DownloadReport {
  const releases: ReleaseDownloads[] = [];
  const platforms: DownloadReport["platforms"] = { "Ubuntu/Linux": 0, Windows: 0, macOS: 0 };
  const seen = new Set<string>();
  for (const page of pages) {
    if (!Array.isArray(page)) throw Error("Invalid GitHub release response.");
    for (const release of page) {
      if (!release || typeof release !== "object") throw Error("Invalid release.");
      if (release.draft || release.prerelease) continue;
      if (typeof release.tag_name !== "string" || !Array.isArray(release.assets) ||
          release.draft !== false || release.prerelease !== false ||
          typeof release.published_at !== "string") throw Error("Invalid published release.");
      if (seen.has(release.tag_name)) continue;
      seen.add(release.tag_name);
      const packages: PackageDownloads[] = [];
      const byPlatform: Record<Platform, number> = { "Ubuntu/Linux": 0, Windows: 0, macOS: 0 };
      for (const asset of release.assets) {
        if (!asset || typeof asset.name !== "string") throw Error("Invalid release asset.");
        const type = packageType(asset.name);
        if (!type || !asset.name.startsWith(`Trip.Viewer_${release.tag_name.replace(/^v/, "")}_`)) continue;
        if (!Number.isSafeInteger(asset.download_count) || asset.download_count < 0) throw Error("Invalid download count.");
        packages.push({ name: asset.name, ...type, downloads: asset.download_count });
        platforms[type.platform] += asset.download_count;
        byPlatform[type.platform] += asset.download_count;
      }
      releases.push({ version: release.tag_name, packages, platforms: byPlatform, total: packages.reduce((sum, p) => sum + p.downloads, 0) });
    }
  }
  return { releases, platforms, total: Object.values(platforms).reduce((a, b) => a + b, 0), fetchedAt };
}

// On-demand, shared across Settings openings. No app/version/installation data
// is sent; only GitHub's public counters are read. Failed refreshes keep cache.
export function createDownloadStatsClient(request: typeof fetch = fetch) {
  let cached: DownloadReport | null = null;
  let pending: Promise<DownloadReport> | null = null;
  return async (force = false): Promise<DownloadReport> => {
    if (pending) return pending;
    if (!force && cached && Date.now() - cached.fetchedAt < 15 * 60 * 1000) return cached;
    pending = (async () => {
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), 15000);
      try {
        const pages: unknown[] = [];
        for (let page = 1; page <= 20; page++) {
          const response = await request(page === 1 ? DOWNLOADS_API : `${DOWNLOADS_API}&page=${page}`, {
            signal: controller.signal, credentials: "omit", headers: { Accept: "application/vnd.github+json" },
          });
          if (!response.ok) throw Error(response.status === 403 || response.status === 429
            ? "GitHub request limit reached. Please try again later." : "GitHub download statistics are unavailable.");
          const value: unknown = await response.json();
          if (!Array.isArray(value)) throw Error("Invalid GitHub release response.");
          pages.push(value);
          if (value.length < 100) { cached = summarizeDownloads(pages); return cached; }
        }
        throw Error("Too many release pages to load a complete report.");
      } finally { clearTimeout(timer); }
    })();
    try { return await pending; } finally { pending = null; }
  };
}

export const readDownloadStats = createDownloadStatsClient();
