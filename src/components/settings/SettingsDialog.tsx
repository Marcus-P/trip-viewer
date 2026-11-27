import { useEffect } from "react";
import { SETTINGS_SECTIONS, type SettingsSection } from "../../settings/navigation";
import { FfmpegConfig } from "../timelapse/FfmpegConfig";
import { GeneralSettings } from "./GeneralSettings";
import { PlayerSettings } from "./PlayerSettings";

interface Props {
  open: boolean;
  onClose: () => void;
  section?: SettingsSection;
  onSectionChange?: (section: SettingsSection) => void;
}

export function SettingsDialog({ open, onClose, section = "general", onSectionChange }: Props) {
  useEffect(() => {
    if (!open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);

  if (!open) return null;

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
        className="max-h-[calc(100vh-2rem)] w-full max-w-2xl overflow-y-auto rounded-lg border border-neutral-700 bg-neutral-900 p-5 shadow-2xl"
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

        <div role="tablist" aria-label="Settings sections" className="mb-5 flex flex-wrap gap-1 border-b border-neutral-700 pb-2">
          {SETTINGS_SECTIONS.map((item) => (
            <button
              key={item.id}
              type="button"
              role="tab"
              aria-selected={section === item.id}
              aria-controls={`settings-panel-${item.id}`}
              id={`settings-tab-${item.id}`}
              onClick={() => onSectionChange?.(item.id)}
              className={`rounded px-3 py-1.5 text-sm ${section === item.id ? "bg-neutral-700 text-white" : "text-neutral-400 hover:bg-neutral-800"}`}
            >
              {item.label}
            </button>
          ))}
        </div>
        <div role="tabpanel" id={`settings-panel-${section}`} aria-labelledby={`settings-tab-${section}`}>
        {section === "player" && <PlayerSettings />}
        {section === "general" && <GeneralSettings />}
        {section === "timelapse" && <FfmpegConfig embedded onClose={onClose} />}
        {section === "scan" && <p className="text-sm text-neutral-400">Choose detection options for each scan in the Scan tab.</p>}
        {section === "review" && <p className="text-sm text-neutral-400">Review filters apply to the current review and are selected in the Review tab.</p>}
        {section === "places" && <p className="text-sm text-neutral-400">Manage archive-specific places in the Places tab.</p>}
        </div>
        <p className="mt-5 text-xs text-neutral-500">Preferences apply immediately. FFmpeg paths are applied after a successful check.</p>
        <div className="mt-4 flex justify-end">
          <button type="button" onClick={onClose} className="rounded-md border border-neutral-700 px-3 py-2 text-sm text-neutral-300 hover:bg-neutral-800">
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
