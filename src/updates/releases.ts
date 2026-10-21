export const RELEASES_PAGE = "https://github.com/Marcus-P/trip-viewer/releases";
export const LATEST_RELEASE = RELEASES_PAGE + "/latest";
const RELEASE_API = "https://api.github.com/repos/Marcus-P/trip-viewer/releases/latest";

export type InstallTarget = "linux-x64" | "windows-x64" | "macos-universal" | "unsupported";
const INSTALLERS = {
  "linux-x64": { suffix: "Ubuntu-24.04_amd64.deb", label: "Download Ubuntu installer",
    instructions: "Download the .deb package, close Trip Viewer and double-click the package to install or update." },
  "windows-x64": { suffix: "Windows_x64_setup.exe", label: "Download Windows installer",
    instructions: "Download the .exe installer, close Trip Viewer and run the installer to install or update. The installer is not publisher-signed." },
  "macos-universal": { suffix: "macOS_universal_experimental.dmg", label: "Download Mac installer",
    instructions: "Experimental macOS build, not notarized by Apple. Open the .dmg, close Trip Viewer and drag the app into Applications to install or update." },
};

export function installerInstructions(target: InstallTarget): string {
  return target === "unsupported" ? "Open Releases to choose a package compatible with your computer." :
    INSTALLERS[target].instructions;
}

export interface PublishedRelease {
  version: string;
  relation: "newer" | "same" | "installedAhead";
  pageUrl: string;
  installerUrl: string | null;
  installerLabel: string | null;
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

export function parsePublishedRelease(value: unknown, installed: string,
  target: InstallTarget = "unsupported"): PublishedRelease {
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
  const installer = target === "unsupported" ? null : INSTALLERS[target];
  const name = installer ? "Trip.Viewer_" + version + "_" + installer.suffix : "";
  const expectedUrl = RELEASES_PAGE + "/download/v" + version + "/" + name;
  const assets = Array.isArray(release.assets) ? release.assets : [];
  const hasInstaller = !!installer && assets.some((asset) => {
    if (!asset || typeof asset !== "object") return false;
    const item = asset as Record<string, unknown>;
    return item.name === name && item.state === "uploaded" &&
      typeof item.size === "number" && item.size > 0 && item.browser_download_url === expectedUrl;
  });
  return { version, relation, pageUrl: RELEASES_PAGE + "/tag/v" + version,
    installerUrl: hasInstaller ? expectedUrl : null,
    installerLabel: hasInstaller ? installer!.label : null };
}

// Share one bounded request between the navigation hint and Settings. No polling,
// archive data, account or token is involved; manual Check can refresh the cache.
export function createReleaseClient(request?: typeof fetch) {
  let cached: { value: unknown; expires: number } | null = null;
  let pending: Promise<unknown> | null = null;
  return async (installed: string, force = false, target: InstallTarget = "unsupported"): Promise<PublishedRelease> => {
    if (!force && cached && cached.expires > Date.now()) {
      return parsePublishedRelease(cached.value, installed, target);
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
          parsePublishedRelease(value, installed, target);
          cached = { value, expires: Date.now() + 15 * 60 * 1000 };
          return value;
        } finally {
          clearTimeout(timer);
        }
      })();
    }
    const active = pending;
    try { return parsePublishedRelease(await active, installed, target); }
    finally { if (pending === active) pending = null; }
  };
}

export const readLatestRelease = createReleaseClient();
