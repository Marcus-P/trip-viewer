import { useEffect, useMemo, useState } from "react";
import { getLayoutPreferences, saveLayoutPreferences, subscribeLayoutPreferences } from "../../settings/layout";
import { usePreferences, savePreferences } from "../../settings/preferences";
import type { CameraLayout } from "../../settings/overlayLayout";
import { useStore } from "../../state/store";
import { LABEL_FRONT, LABEL_INTERIOR, LABEL_REAR } from "../../types/model";

export function PlayerSettings() {
  const preferences = usePreferences();
  const [hiddenChannels, setHiddenChannels] = useState(() => getLayoutPreferences().hiddenChannels);
  const [cameraLayout, setCameraLayout] = useState<CameraLayout>(() => getLayoutPreferences().cameraLayout);
  const trips = useStore((s) => s.trips);
  const channelLabels = useMemo(() => [...new Set([
    LABEL_FRONT, LABEL_INTERIOR, LABEL_REAR,
    ...trips.flatMap(trip => trip.segments.flatMap(segment => segment.channels.map(channel => channel.label))),
    ...hiddenChannels,
  ])], [trips, hiddenChannels]);

  useEffect(() => subscribeLayoutPreferences(() => {
    setHiddenChannels(getLayoutPreferences().hiddenChannels);
    setCameraLayout(getLayoutPreferences().cameraLayout);
  }), []);

  return (
    <div className="space-y-5">
          <label className="block">
            <span className="mb-1.5 block text-xs font-medium text-neutral-300">
              Speed unit
            </span>
            <select
              value={preferences.speedUnit}
              onChange={(event) =>
                savePreferences({
                  ...preferences,
                  speedUnit: event.target.value === "mph" ? "mph" : "kmh",
                })
              }
              className="w-full rounded-md border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm text-neutral-200 outline-none focus:border-neutral-500"
            >
              <option value="kmh">km/h</option>
              <option value="mph">mph</option>
            </select>
          </label>

          <fieldset className="space-y-2 rounded-md border border-neutral-800 bg-neutral-950/60 px-3 pb-3">
            <legend className="px-1 text-xs font-medium text-neutral-300">Display layout</legend>
            <label className="flex items-center justify-between gap-4 py-1 text-xs text-neutral-300">
              <span>Camera layout</span>
              <select aria-label="Default camera layout" value={cameraLayout}
                onKeyDown={event => event.stopPropagation()}
                onChange={(event) => {
                  const next = event.target.value === "overlay" ? "overlay" : "grid";
                  setCameraLayout(next);
                  saveLayoutPreferences({ cameraLayout: next });
                }}
                className="rounded border border-neutral-700 bg-neutral-900 px-2 py-1">
                <option value="grid">Classic</option><option value="overlay">Overlay</option>
              </select>
            </label>
            <p className="text-[11px] text-neutral-500">
              Choose the cameras and map to display. Hidden views free up space.
            </p>
            {channelLabels.map(label => (
              <label key={label} className="flex items-center justify-between gap-4 py-1 text-xs text-neutral-300">
                <span>{label} camera</span>
                <input
                  type="checkbox"
                  aria-label={`Show ${label}`}
                  checked={!hiddenChannels.includes(label)}
                  onChange={(event) => {
                    const checked = event.currentTarget.checked;
                    const next = checked
                      ? hiddenChannels.filter(channel => channel !== label)
                      : [...hiddenChannels, label];
                    setHiddenChannels(next);
                    saveLayoutPreferences({ hiddenChannels: next });
                  }}
                  className="h-4 w-4 accent-blue-500"
                />
              </label>
            ))}
            <label className="flex items-center justify-between gap-4 border-t border-neutral-800 pt-2">
              <div>
                <span className="block text-xs font-medium text-neutral-300">Show map</span>
                <span className="mt-0.5 block text-[11px] text-neutral-500">
                  Hide the GPS map and give the video grid more space.
                </span>
          </div>
              <input
                type="checkbox"
                checked={preferences.showMap}
                onChange={(event) =>
                  savePreferences({
                    ...preferences,
                    showMap: event.target.checked,
                  })
                }
                className="h-4 w-4 accent-blue-500"
              />
            </label>
          </fieldset>
    </div>
  );
}
