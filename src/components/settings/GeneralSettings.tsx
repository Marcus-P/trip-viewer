import { savePreferences, usePreferences } from "../../settings/preferences";
import { ReleaseUpdates } from "./ReleaseUpdates";
import { DownloadStatistics } from "./DownloadStatistics";

export function GeneralSettings() {
  const preferences = usePreferences();
  return (
    <div className="space-y-5">
          <label className="block">
            <span className="mb-1.5 block text-xs font-medium text-neutral-300">
              Camera model
            </span>
            <select
              value={preferences.cameraModel}
              onChange={(event) =>
                savePreferences({
                  ...preferences,
                  cameraModel:
                    event.target.value === "viofoA229Pro"
                      ? "viofoA229Pro"
                      : "auto",
                })
              }
              className="w-full rounded-md border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm text-neutral-200 outline-none focus:border-neutral-500"
            >
              <option value="auto">Automatic</option>
              <option value="viofoA229Pro">VIOFO A229 Pro</option>
            </select>
            <span className="mt-1 block text-[11px] text-neutral-500">
              Automatic keeps filename-based camera detection enabled.
            </span>
          </label>

          <ReleaseUpdates />
          <DownloadStatistics />
        </div>
  );
}
