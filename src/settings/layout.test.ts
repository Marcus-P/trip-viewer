import { afterEach, expect, it, vi } from "vitest";
import { getLayoutPreferences, saveLayoutPreferences } from "./layout";
afterEach(() => vi.unstubAllGlobals());
it("persists camera choices while retaining existing layout preferences", () => {
  let stored = JSON.stringify({ primaryShare: .7, dashboardTimelineCollapsed: true });
  vi.stubGlobal("window", { localStorage: { getItem: () => stored, setItem: (_: string, value: string) => { stored = value; } } });
  expect(getLayoutPreferences().hiddenChannels).toEqual([]);
  saveLayoutPreferences({ hiddenChannels: ["Front", "Rear", "Rear"] });
  const restored = getLayoutPreferences();
  expect(restored.hiddenChannels).toEqual(["Front", "Rear"]);
  expect(restored.primaryShare).toBe(.7);
  expect(restored.dashboardTimelineCollapsed).toBe(true);
});
