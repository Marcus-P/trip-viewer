import { renderToStaticMarkup } from "react-dom/server";
import { expect, it } from "vitest";
import { StartupSplash } from "./StartupSplash";

it("shows measured file counts for the current phase, even while maintenance runs", () => {
  const html = renderToStaticMarkup(<StartupSplash
    snapshot={{ done: false, tasks: [{ id: "gps", label: "GPS maintenance", description: "", current: 0, total: 2, status: "running" }] }}
    libraryLoading libraryProgress={{ phase: "metadata", completed: 2, total: 10, cached: 1 }} />);
  expect(html).toContain("Checking video metadata");
  expect(html).toContain("2 / 10");
  expect(html).toContain("width:20%");
  expect(html).toContain("1 unchanged files reused");
});

it("does not invent a percentage for an unknown archive total", () => {
  const html = renderToStaticMarkup(<StartupSplash snapshot={{ done: true, tasks: [] }}
    libraryLoading libraryProgress={{ phase: "discovering", completed: 0, total: null, cached: 0 }} />);
  expect(html).toContain("Finding video files");
  expect(html).not.toMatch(/width:\d+%/);
});
