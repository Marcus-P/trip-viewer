import { describe, expect, it, vi } from "vitest";
import { createReleaseClient, parsePublishedRelease, RELEASES_PAGE } from "./releases";

function release(version = "0.7.0") {
  const name = "Trip.Viewer_" + version + "_Ubuntu-24.04_amd64.deb";
  return { tag_name: "v" + version, draft: false, prerelease: false,
    published_at: "2026-10-21T12:00:00Z", assets: [{ name, size: 100, state: "uploaded",
      browser_download_url: RELEASES_PAGE + "/download/v" + version + "/" + name }] };
}

describe("published release selection", () => {
  it("compares numeric versions and never recommends a downgrade", () => {
    expect(parsePublishedRelease(release("0.10.0"), "0.9.9").relation).toBe("newer");
    expect(parsePublishedRelease(release(), "0.7.0").relation).toBe("same");
    expect(parsePublishedRelease(release("0.6.0"), "0.7.0").relation).toBe("installedAhead");
  });
  it("rejects drafts, prereleases, unpublished responses and malformed versions", () => {
    for (const patch of [{ draft: true }, { prerelease: true }, { published_at: null },
      { tag_name: "v0.8.0-beta" }, { tag_name: "v01.0.0" }]) {
      expect(() => parsePublishedRelease({ ...release(), ...patch }, "0.6.0")).toThrow();
    }
  });
  it("accepts only a complete Ubuntu package from the exact fork release", () => {
    const data = release();
    expect(parsePublishedRelease(data, "0.6.0").debUrl).toContain("/Marcus-P/trip-viewer/");
    data.assets[0].browser_download_url = "https://example.invalid/download.deb";
    expect(parsePublishedRelease(data, "0.6.0").debUrl).toBeNull();
    data.assets[0].size = 0;
    expect(parsePublishedRelease(data, "0.6.0").debUrl).toBeNull();
  });
  it("keeps the release page available for older releases without a deb", () => {
    expect(parsePublishedRelease({ ...release("0.6.0"), assets: [] }, "0.5.0"))
      .toMatchObject({ relation: "newer", debUrl: null, pageUrl: RELEASES_PAGE + "/tag/v0.6.0" });
  });
});

describe("release check isolation", () => {
  it("shares requests and caches a successful result while supporting manual refresh", async () => {
    const request = vi.fn().mockResolvedValue({ ok: true, json: async () => release() });
    const client = createReleaseClient(request);
    const results = await Promise.all([client("0.6.0"), client("0.7.0")]);
    expect(results.map((r) => r.relation)).toEqual(["newer", "same"]);
    expect(request).toHaveBeenCalledTimes(1);
    await client("0.6.0");
    expect(request).toHaveBeenCalledTimes(1);
    await client("0.6.0", true);
    expect(request).toHaveBeenCalledTimes(2);
  });
  it("does not cache failures, so offline/rate-limited checks can be retried", async () => {
    const request = vi.fn().mockResolvedValueOnce({ ok: false })
      .mockResolvedValueOnce({ ok: true, json: async () => release() });
    const client = createReleaseClient(request);
    await expect(client("0.6.0")).rejects.toThrow();
    await expect(client("0.6.0", true)).resolves.toMatchObject({ relation: "newer" });
  });
  it("aborts a stalled request instead of hanging the UI", async () => {
    vi.useFakeTimers();
    try {
      const client = createReleaseClient(vi.fn((_url: RequestInfo | URL, init?: RequestInit) => new Promise<Response>((_resolve, reject) => {
        init?.signal?.addEventListener("abort", () => reject(Error("aborted")));
      })));
      const checked = expect(client("0.6.0")).rejects.toThrow("aborted");
      await vi.advanceTimersByTimeAsync(8000);
      await checked;
    } finally { vi.useRealTimers(); }
  });
});
