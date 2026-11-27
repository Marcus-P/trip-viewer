import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, expect, it } from "vitest";
import { SettingsDialog } from "./SettingsDialog";
import { useSettingsNavigation } from "../../settings/navigation";

const noop = () => {};
afterEach(() => useSettingsNavigation.getState().close());

it("offers one central dialog with immediate preferences and no Save/Cancel", () => {
  const html = renderToStaticMarkup(<SettingsDialog open section="player" onClose={noop} />);
  expect(html).toContain("Speed unit");
  expect(html).toContain("Preferences apply immediately");
  expect(html).not.toMatch(/>\s*(Save|Cancel)\s*</);
  expect(html.match(/role="dialog"/g)).toHaveLength(1);
});

it("routes FFmpeg into the same settings dialog rather than a second modal", () => {
  useSettingsNavigation.getState().open("timelapse");
  const section = useSettingsNavigation.getState().section!;
  const html = renderToStaticMarkup(<SettingsDialog open section={section} onClose={noop} />);
  expect(html).toContain("Path to ffmpeg executable");
  expect(html.match(/role="dialog"/g)).toHaveLength(1);
  expect(html.match(/fixed inset-0/g)).toHaveLength(1);
  expect(html).not.toContain("Speed unit");
});
