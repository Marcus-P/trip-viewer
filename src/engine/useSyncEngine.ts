import { useEffect, useRef, useState } from "react";
import { useStore } from "../state/store";
import { initialChannelPosition, positionMedia } from "./mediaPosition";
import { SyncEngine } from "./SyncEngine";
import type { CurveSegment } from "../utils/speedCurve";

/**
 * Wire up a `SyncEngine` instance for the current segment.
 *
 * @param channelRefs  Map keyed by channel label; populated by `VideoGrid`
 *                     as each `<video>` element mounts.
 * @param channelLabels Ordered list of labels in the current segment
 *                     (canonical order — first entry is the sync master).
 * @param activeSegmentId The current segment id (changing this recreates the engine).
 * @param channelCurves Per-channel speed curves keyed by label, for tiered
 *                     playback. The master's curve maps file-time → concat-time;
 *                     a slave's curve drives its coverage-gap hold/overlay.
 *                     Empty map in Original mode. Captured at engine
 *                     construction (stable per tier, so not a dep).
 */
export function useSyncEngine(
  channelRefs: React.MutableRefObject<Map<string, HTMLVideoElement | null>>,
  channelLabels: string[],
  activeSegmentId: string | null,
  channelCurves: Map<string, CurveSegment[]> = new Map(),
): SyncEngine | null {
  const [engine, setEngine] = useState<SyncEngine | null>(null);
  const engineRef = useRef<SyncEngine | null>(null);

  // Stable string key that captures the identity of the current segment's
  // channel lineup. If either the segment id OR the set of channel labels
  // changes, we tear down the engine and rebuild with the new lineup.
  const labelsKey = channelLabels.join("|");

  useEffect(() => {
    engineRef.current?.pause();
    engineRef.current?.dispose();
    engineRef.current = null;
    setEngine(null);

    if (!activeSegmentId || channelLabels.length === 0) return;

    const masterLabel = channelLabels[0];
    const slaveLabels = channelLabels.slice(1);

    const getEl = (label: string) => channelRefs.current.get(label) ?? null;

    const masterCurve = channelCurves.get(masterLabel) ?? null;
    const primed = new Set<HTMLVideoElement>();
    const prime = (label: string) => {
      const video = getEl(label);
      if (!video || primed.has(video)) return;
      const channelTarget = initialChannelPosition(
        useStore.getState().currentTime,
        masterCurve,
        channelCurves.get(label) ?? null,
      );
      try {
        if (positionMedia(video, channelTarget)) primed.add(video);
      } catch (error) {
        console.warn("[sync] initial positioning failed:", error);
      }
    };

    const tryInit = () => {
      for (const label of [masterLabel, ...slaveLabels]) prime(label);
      const master = getEl(masterLabel);
      if (!master || master.readyState < 2 || master.seeking) return;
      if (engineRef.current) return;

      // Wait for every slave to be ready — if we init with a partial set,
      // the engineRef guard prevents re-initialization and the missing
      // slaves are permanently excluded from control (observable as those
      // channels freezing after scrubbing).
      const slaves: HTMLVideoElement[] = [];
      const includedLabels: string[] = [];
      for (const label of slaveLabels) {
        const el = getEl(label);
        if (!el || el.readyState < 2 || el.seeking) return;
        slaves.push(el);
        includedLabels.push(label);
      }

      const slaveCurves = includedLabels.map((l) => channelCurves.get(l) ?? null);
      const e = new SyncEngine(master, slaves, includedLabels, masterCurve, slaveCurves);
      e.start();
      engineRef.current = e;
      setEngine(e);
      window.clearInterval(readinessPoll);
    };

    // WebKit can reach HAVE_CURRENT_DATA without delivering loadeddata after
    // a source reset. Readiness must not leave Play disabled forever merely
    // because that event was lost. The fallback requires actual metadata and
    // applies at most one initial position per element; time is not evidence
    // of readiness or seek completion.
    const readinessPoll = window.setInterval(tryInit, 250);
    tryInit();

    // Metadata permits the initial seek; seeked/current data permits engine
    // creation. The existing readiness fallback also handles a missed event,
    // but elapsed time never establishes a successful seek.
    const allLabels = [masterLabel, ...slaveLabels];
    const listeners: Array<[HTMLVideoElement, () => void]> = [];
    for (const label of allLabels) {
      const el = getEl(label);
      if (!el) continue;
      const h = () => tryInit();
      el.addEventListener("loadedmetadata", h);
      el.addEventListener("seeked", h);
      el.addEventListener("loadeddata", h);
      listeners.push([el, h]);
    }

    return () => {
      window.clearInterval(readinessPoll);
      for (const [el, h] of listeners) {
        el.removeEventListener("loadedmetadata", h);
        el.removeEventListener("seeked", h);
        el.removeEventListener("loadeddata", h);
      }
      engineRef.current?.pause();
      engineRef.current?.dispose();
      engineRef.current = null;
      setEngine(null);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeSegmentId, labelsKey, channelRefs]);

  return engine;
}
