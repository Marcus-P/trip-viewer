export const RELEASES_PAGE = "https://github.com/Marcus-P/trip-viewer/releases";
export const LATEST_RELEASE = RELEASES_PAGE + "/latest";
const RELEASE_API = "https://api.github.com/repos/Marcus-P/trip-viewer/releases/latest";

export interface PublishedRelease {
  version: string;
  relation: "newer" | "same" | "installedAhead";
  pageUrl: string;
  debUrl: string | null;
}

function versionParts(value: string): bigint[] {
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.exec(value);
  if (!match) throw Error("Invalid stable release version");
  return match.slice(1).map(BigInt);
}

function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw Error("Invalid release response");
  return value as Record<string, unknown>;
}

export function parsePublishedRelease(value: unknown, installed: string): PublishedRelease {
  const release = object(value);
  if (release.draft !== false || release.prerelease !== false ||
      typeof release.published_at !== "string" || !Number.isFinite(Date.parse(release.published_at)) ||
      typeof release.tag_name !== "string" || !release.tag_name.startsWith("v")) {
    throw Error("No published stable release");
  }
  const version = release.tag_name.slice(1);
  const available = versionParts(version);
  const current = versionParts(installed);
  const different = available.findIndex((part, index) => part !== current[index]);
  const relation = different < 0 ? "same" :
    available[different] > current[different] ? "newer" : "installedAhead";
  const name = "Trip.Viewer_" + version + "_Ubuntu-24.04_amd64.deb";
  const expectedUrl = RELEASES_PAGE + "/download/v" + version + "/" + name;
  const assets = Array.isArray(release.assets) ? release.assets : [];
  const hasDeb = assets.some((asset) => {
    if (!asset || typeof asset !== "object") return false;
    const item = asset as Record<string, unknown>;
    return item.name === name && item.state === "uploaded" &&
      typeof item.size === "number" && item.size > 0 && item.browser_download_url === expectedUrl;
  });
  return { version, relation, pageUrl: RELEASES_PAGE + "/tag/v" + version,
    debUrl: hasDeb ? expectedUrl : null };
}

// Share one bounded request between the navigation hint and Settings. No polling,
// archive data, account or token is involved; manual Check can refresh the cache.
export function createReleaseClient(request?: typeof fetch) {
  let cached: { value: unknown; expires: number } | null = null;
  let pending: Promise<unknown> | null = null;
  return async (installed: string, force = false): Promise<PublishedRelease> => {
    if (!force && cached && cached.expires > Date.now()) {
      return parsePublishedRelease(cached.value, installed);
    }
    if (!pending) {
      pending = (async () => {
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), 8000);
        try {
          const response = await (request ?? fetch)(RELEASE_API, {
            signal: controller.signal,
            headers: { Accept: "application/vnd.github+json" },
            credentials: "omit",
          });
          if (!response.ok) throw Error("Release check unavailable");
          const value: unknown = await response.json();
          parsePublishedRelease(value, installed);
          cached = { value, expires: Date.now() + 15 * 60 * 1000 };
          return value;
        } finally {
          clearTimeout(timer);
        }
      })();
    }
    const active = pending;
    try { return parsePublishedRelease(await active, installed); }
    finally { if (pending === active) pending = null; }
  };
}

export const readLatestRelease = createReleaseClient();
