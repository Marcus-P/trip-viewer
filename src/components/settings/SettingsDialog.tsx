import { useEffect, useState } from "react";
import {
  savePreferences,
  usePreferences,
  type UserPreferences,
} from "../../settings/preferences";

interface Props {
  open: boolean;
  onClose: () => void;
}

export function SettingsDialog({ open, onClose }: Props) {
  const preferences = usePreferences();
  const [draft, setDraft] = useState<UserPreferences>(preferences);

  useEffect(() => {
    if (open) setDraft(preferences);
  }, [open, preferences]);

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);

  if (!open) return null;

  function save() {
    savePreferences(draft);
    onClose();
  }

  return (
    <div
      className="fixed inset-0 z-[5000] flex items-center justify-center bg-black/70 p-4"
      role="presentation"
      onMouseDown={(event) => {
        if (event.currentTarget === event.target) onClose();
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
        className="w-full max-w-md rounded-lg border border-neutral-700 bg-neutral-900 p-5 shadow-2xl"
      >
        <div className="mb-5 flex items-center justify-between gap-4">
          <h2 id="settings-title" className="text-base font-semibold text-neutral-100">
            Settings
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="text-xl leading-none text-neutral-500 hover:text-neutral-200"
            aria-label="Close settings"
          >
            ×
          </button>
        </div>

        <div className="space-y-5">
          <label className="block">
            <span className="mb-1.5 block text-xs font-medium text-neutral-300">
              Speed unit
            </span>
            <select
              value={draft.speedUnit}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  speedUnit: event.target.value === "mph" ? "mph" : "kmh",
                }))
              }
              className="w-full rounded-md border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm text-neutral-200 outline-none focus:border-neutral-500"
            >
              <option value="kmh">km/h</option>
              <option value="mph">mph</option>
            </select>
          </label>

          <label className="block">
            <span className="mb-1.5 block text-xs font-medium text-neutral-300">
              Camera model
            </span>
            <select
              value={draft.cameraModel}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  cameraModel:
                    event.target.value === "viofoA229Pro"
                      ? "viofoA229Pro"
                      : "auto",
                }))
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

          <label className="flex items-center justify-between gap-4 rounded-md border border-neutral-800 bg-neutral-950/60 px-3 py-2.5">
            <div>
              <span className="block text-xs font-medium text-neutral-300">Show map</span>
              <span className="mt-0.5 block text-[11px] text-neutral-500">
                Hide the GPS map and give the video grid more space.
              </span>
            </div>
            <input
              type="checkbox"
              checked={draft.showMap}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  showMap: event.target.checked,
                }))
              }
              className="h-4 w-4 accent-blue-500"
            />
          </label>
        </div>

        <div className="mt-6 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-md px-3 py-2 text-sm text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={save}
            className="rounded-md bg-blue-600 px-3 py-2 text-sm font-medium text-white hover:bg-blue-500"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
