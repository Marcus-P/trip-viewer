import { create } from "zustand";

export const SETTINGS_SECTIONS = [
  { id: "general", label: "General" },
  { id: "player", label: "Player" },
  { id: "scan", label: "Scan" },
  { id: "review", label: "Review" },
  { id: "timelapse", label: "Timelapse" },
  { id: "places", label: "Places" },
] as const;

export type SettingsSection = typeof SETTINGS_SECTIONS[number]["id"];

interface SettingsNavigation {
  section: SettingsSection | null;
  open: (section?: SettingsSection) => void;
  close: () => void;
}

// Navigation only. Preferences keep their existing storage and subscribers;
// opening FFmpeg setup never creates a second settings implementation.
export const useSettingsNavigation = create<SettingsNavigation>((set) => ({
  section: null,
  open: (section = "general") => set({ section }),
  close: () => set({ section: null }),
}));
