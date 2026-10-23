import { afterEach, expect, it, vi } from "vitest";
import { getLayoutPreferences, saveLayoutPreferences, subscribeLayoutPreferences } from "./layout";
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
it("notifies a mounted player of saved settings and retains them during resizing", () => {
  let stored = JSON.stringify({ hiddenChannels: ["Interior"], primaryShare: .7 });
  vi.stubGlobal("window", { localStorage: { getItem: () => stored, setItem: (_: string, value: string) => { stored = value; } } });
  const seen: string[][] = [];
  const unsubscribe = subscribeLayoutPreferences(() => {
    seen.push(getLayoutPreferences().hiddenChannels);
  });
  try {
    saveLayoutPreferences({ hiddenChannels: ["Interior", "Rear"] });
    expect(seen).toEqual([["Interior", "Rear"]]);
    saveLayoutPreferences({ primaryShare: .8 });
    expect(getLayoutPreferences().hiddenChannels).toEqual(["Interior", "Rear"]);
    expect(getLayoutPreferences().primaryShare).toBe(.8);
  } finally {
    unsubscribe();
  }
  const count = seen.length;
  saveLayoutPreferences({ hiddenChannels: [] });
  expect(seen).toHaveLength(count);
});
