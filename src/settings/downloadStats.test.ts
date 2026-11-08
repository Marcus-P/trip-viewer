import { expect, it, vi } from "vitest";
import { createDownloadStatsClient, summarizeDownloads } from "./downloadStats";
const release = (tag = "v0.9.0") => ({ tag_name: tag, draft: false, prerelease: false, published_at: "2026-11-06T23:09:03Z", assets: [
  { name: `Trip.Viewer_${tag.slice(1)}_Ubuntu-24.04_amd64.deb`, download_count: 2 },
  { name: `Trip.Viewer_${tag.slice(1)}_Ubuntu-24.04_amd64.AppImage`, download_count: 1 },
  { name: `Trip.Viewer_${tag.slice(1)}_Ubuntu-24.04_bundle.tar.gz`, download_count: 1 },
  { name: `Trip.Viewer_${tag.slice(1)}_Windows_x64_setup.exe`, download_count: 2 },
  { name: `Trip.Viewer_${tag.slice(1)}_macOS_universal_experimental.dmg`, download_count: 1 },
  { name: "LICENSE", download_count: 99 },
] });
it("counts app packages by OS, release and format without counting notices or duplicate releases", () => {
 const r = release();
 const result = summarizeDownloads([[r, { ...release("v0.8.2"), draft: true }, { ...release("v0.8.1"), prerelease: true }], [r]], 123);
 expect(result).toMatchObject({ total: 7, platforms: { "Ubuntu/Linux": 4, Windows: 2, macOS: 1 }, fetchedAt: 123 });
 expect(result.releases).toHaveLength(1);
 expect(result.releases[0].platforms).toEqual(result.platforms);
 expect(result.releases[0].packages).toHaveLength(5);
 expect(result.releases[0].packages.map(p => p.format)).toContain("Bundle");
});
it("does not turn invalid counters or API errors into zero downloads", () => {
 const r=release(); r.assets[0].download_count=-1;
 expect(()=>summarizeDownloads([[r]])).toThrow();
 expect(()=>summarizeDownloads([{ message:"rate limit" }])).toThrow();
 expect(summarizeDownloads([[]]).total).toBe(0);
});
it("loads all release pages and shares cached/in-flight results", async () => {
 const first=Array.from({length:100},(_,i)=>release(`v0.${i}.0`));
 const request=vi.fn().mockResolvedValueOnce(new Response(JSON.stringify(first))).mockResolvedValueOnce(new Response(JSON.stringify([release("v1.0.0")])));
 const load=createDownloadStatsClient(request);
 const [a,b]=await Promise.all([load(),load()]);
 expect(a).toEqual(b); expect(a.releases).toHaveLength(101);expect(a.total).toBe(707);
 expect(request).toHaveBeenCalledTimes(2);expect(request.mock.calls[1][0]).toContain("page=2");
 await load();expect(request).toHaveBeenCalledTimes(2);
 expect(request.mock.calls[0][1]).toMatchObject({credentials:"omit"});
});
it("reports rate limits and retains a successful cache after a failed refresh", async () => {
 const request=vi.fn().mockResolvedValueOnce(new Response(JSON.stringify([release()]))).mockResolvedValueOnce(new Response('{}',{status:403}));
 const load=createDownloadStatsClient(request);const initial=await load();
 await expect(load(true)).rejects.toThrow('limit');expect(await load()).toEqual(initial);
});
