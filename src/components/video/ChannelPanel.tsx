import {
  forwardRef,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import clsx from "clsx";
import { useStore } from "../../state/store";

// Diagnostic toggles. Both default off so production builds stay silent.
//
//   DEBUG_MEDIA          — once-per-second summary per channel: media time,
//                          real-time ratio, readyState, buffered window, and
//                          event counts since the last tick. Also logs a
//                          "boundary" line whenever the src changes. Low-volume
//                          and enough to characterise stutter vs. boundary
//                          stalls without drowning the console.
//   DEBUG_MEDIA_VERBOSE  — per-event stream (~20 events/sec during stutter on
//                          WebKitGTK). Turn on only for short captures when
//                          you need event ordering.
const DEBUG_MEDIA = false;
const DEBUG_MEDIA_VERBOSE = false;

const MEDIA_EVENTS = [
  "loadstart",
  "loadedmetadata",
  "loadeddata",
  "canplay",
  "canplaythrough",
  "waiting",
  "stalled",
  "suspend",
  "emptied",
  "abort",
  "play",
  "playing",
  "pause",
  "seeking",
  "seeked",
  "ended",
  "error",
] as const;

type SlotIndex = 0 | 1;

interface BufferState {
  active: SlotIndex;
  sources: [string | null, string | null];
}

interface Props {
  label: string;
  src: string;
  /** Next segment's source for the same channel. It is decoded in a paused,
   *  invisible standby video so an automatic segment advance can promote an
   *  already-ready media element instead of changing the visible video's src. */
  preloadSrc?: string | null;
  isMaster: boolean;
  onClick?: () => void;
  onDoubleClick?: () => void;
}

function otherSlot(slot: SlotIndex): SlotIndex {
  return slot === 0 ? 1 : 0;
}

function mediaErrorText(video: HTMLVideoElement): string {
  const code = video.error?.code ?? 0;
  const map: Record<number, string> = {
    1: "aborted",
    2: "network error (failed to load)",
    3: "decode error",
    4: "source not supported (load or codec failure)",
  };
  return map[code] ?? `playback error ${code}`;
}

export const ChannelPanel = forwardRef<HTMLVideoElement, Props>(
  function ChannelPanel(
    { label, src, preloadSrc = null, isMaster, onClick, onDoubleClick },
    ref,
  ) {
    const [error, setError] = useState<string | null>(null);
    const [ready, setReady] = useState(false);
    // True while this channel is in a coverage gap (camera was off for
    // this stretch in tiered playback). The SyncEngine holds the
    // `<video>`; we paint black over it. Always false in Original mode.
    const gapped = useStore((s) => s.gappedChannels[label] ?? false);
    // `showLoading` is `!ready` debounced by LOADING_OVERLAY_DELAY_MS.
    // Fast loads complete before this flips true, so the user sees a smooth
    // cut rather than a flash of "Loading…". With the standby buffer, the
    // normal Original-mode segment boundary should already have loadeddata.
    const [showLoading, setShowLoading] = useState(false);
    const slotRefs = useRef<[HTMLVideoElement | null, HTMLVideoElement | null]>([
      null,
      null,
    ]);
    const [buffer, setBuffer] = useState<BufferState>(() => ({
      active: 0,
      sources: [src, preloadSrc && preloadSrc !== src ? preloadSrc : null],
    }));

    const setSlotRef = useCallback(
      (slot: SlotIndex) => (node: HTMLVideoElement | null) => {
        slotRefs.current[slot] = node;
      },
      [],
    );

    // Synchronize the two persistent media elements with current + next source.
    // If the requested current source is already sitting in the standby slot,
    // promote that DOM element without touching its src. That is the actual
    // double-buffer handoff and preserves its prepared decoder/first frame.
    //
    // Native single-video fullscreen is deliberately different: replacing the
    // fullscreen media element would make WebKit exit fullscreen. In that case
    // keep the same active slot and accept a normal src change for that one
    // transition. Dashboard fullscreen is a parent <div>, so it can use the
    // seamless slot promotion normally.
    useLayoutEffect(() => {
      const wantedStandby = preloadSrc && preloadSrc !== src ? preloadSrc : null;
      setBuffer((previous) => {
        const active = previous.active;
        const standby = otherSlot(active);
        const activeEl = slotRefs.current[active];
        const preserveFullscreenElement = document.fullscreenElement === activeEl;

        if (preserveFullscreenElement) {
          const nextSources: [string | null, string | null] = [
            previous.sources[0],
            previous.sources[1],
          ];
          nextSources[active] = src;
          nextSources[standby] = wantedStandby;
          if (
            previous.sources[active] === nextSources[active] &&
            previous.sources[standby] === nextSources[standby]
          ) {
            return previous;
          }
          return { active, sources: nextSources };
        }

        if (previous.sources[active] === src) {
          if (previous.sources[standby] === wantedStandby) return previous;
          const nextSources: [string | null, string | null] = [
            previous.sources[0],
            previous.sources[1],
          ];
          nextSources[standby] = wantedStandby;
          return { active, sources: nextSources };
        }

        if (previous.sources[standby] === src) {
          const nextSources: [string | null, string | null] = [
            previous.sources[0],
            previous.sources[1],
          ];
          nextSources[standby] = src;
          nextSources[active] = wantedStandby;
          return { active: standby, sources: nextSources };
        }

        // Non-sequential seek/jump: requested media was not preloaded. Keep
        // the active DOM element (same behaviour as before double buffering),
        // change its src normally, and prepare the following source in standby.
        const nextSources: [string | null, string | null] = [
          previous.sources[0],
          previous.sources[1],
        ];
        nextSources[active] = src;
        nextSources[standby] = wantedStandby;
        return { active, sources: nextSources };
      });
    }, [src, preloadSrc]);

    // Publish only the active media element to VideoGrid/useSyncEngine. Both
    // slots remain mounted, but the sync engine must never control standby.
    useLayoutEffect(() => {
      const node = slotRefs.current[buffer.active];
      if (typeof ref === "function") ref(node);
      else if (ref) ref.current = node;
      return () => {
        if (typeof ref === "function") ref(null);
        else if (ref) ref.current = null;
      };
    }, [buffer.active, ref]);

    useEffect(() => {
      setError(null);
      setReady(false);
      const video = slotRefs.current[buffer.active];
      if (video?.error) setError(mediaErrorText(video));
      if (DEBUG_MEDIA) {
        console.log(`[media/${label}] boundary src=…${src.slice(-50)}`);
      }
    }, [src, label, buffer.active]);

    // The loading overlay covers the active <video> until loadeddata fires.
    // A promoted standby normally already has readyState >= 2, so this resolves
    // immediately and the old last frame cuts directly to the new first frame.
    useEffect(() => {
      const video = slotRefs.current[buffer.active];
      if (!video) return;

      const onLoaded = () => setReady(true);
      video.addEventListener("loadeddata", onLoaded);
      if (video.readyState >= 2) setReady(true);

      return () => {
        video.removeEventListener("loadeddata", onLoaded);
      };
    }, [src, buffer.active]);

    // Debounce the overlay. If `ready` flips back to true within the delay
    // window, the timer is cleared and the overlay never paints.
    useEffect(() => {
      if (ready) {
        setShowLoading(false);
        return;
      }
      const LOADING_OVERLAY_DELAY_MS = 150;
      const t = window.setTimeout(() => setShowLoading(true), LOADING_OVERLAY_DELAY_MS);
      return () => window.clearTimeout(t);
    }, [ready]);

    useEffect(() => {
      if (!DEBUG_MEDIA && !DEBUG_MEDIA_VERBOSE) return;
      const video = slotRefs.current[buffer.active];
      if (!video) return;

      const mountedAt = performance.now();
      const counts = new Map<string, number>();

      // Verbose per-event stream.
      const verboseHandler = (ev: Event) => {
        const dt = ((performance.now() - mountedAt) / 1000).toFixed(3);
        const v = ev.currentTarget as HTMLVideoElement;
        const tail = (v.currentSrc || v.src).slice(-50);
        const base =
          `[media/${label}] +${dt}s ${ev.type} ` +
          `rs=${v.readyState} ns=${v.networkState} ` +
          `t=${v.currentTime.toFixed(3)} paused=${v.paused} ended=${v.ended} ` +
          `…${tail}`;
        if (ev.type === "error") {
          const err = v.error;
          console.error(`${base} code=${err?.code ?? "?"} msg=${err?.message ?? ""}`);
        } else {
          console.log(base);
        }
      };

      // Always-on counter (cheap).
      const countHandler = (ev: Event) => {
        counts.set(ev.type, (counts.get(ev.type) ?? 0) + 1);
      };

      for (const name of MEDIA_EVENTS) {
        if (DEBUG_MEDIA_VERBOSE) video.addEventListener(name, verboseHandler);
        if (DEBUG_MEDIA) video.addEventListener(name, countHandler);
      }

      // Summary tick every 1 s.
      let timer: number | null = null;
      if (DEBUG_MEDIA) {
        let lastWall = performance.now();
        let lastMedia = video.currentTime;
        timer = window.setInterval(() => {
          const nowWall = performance.now();
          const nowMedia = video.currentTime;
          const dWall = (nowWall - lastWall) / 1000;
          const dMedia = nowMedia - lastMedia;
          const rt = dWall > 0 ? ((dMedia / dWall) * 100).toFixed(0) : "—";
          const uptime = ((nowWall - mountedAt) / 1000).toFixed(1);

          let bufEnd = "—";
          let bufHead = "—";
          try {
            const b = video.buffered;
            if (b.length > 0) {
              const end = b.end(b.length - 1);
              bufEnd = end.toFixed(2);
              bufHead = (end - nowMedia).toFixed(2);
            }
          } catch {
            // some engines throw on buffered access with no data
          }

          const evSummary =
            counts.size === 0
              ? "events: (none)"
              : "events: " +
                Array.from(counts.entries())
                  .sort((a, b) => b[1] - a[1])
                  .map(([k, v]) => `${k}=${v}`)
                  .join(" ");

          console.log(
            `[media/${label}] +${uptime}s t=${nowMedia.toFixed(2)} ` +
              `Δt=${dMedia.toFixed(2)} rt=${rt}% ` +
              `rs=${video.readyState} ns=${video.networkState} ` +
              `buf=${bufEnd} ahead=${bufHead}s ${evSummary}`,
          );

          counts.clear();
          lastWall = nowWall;
          lastMedia = nowMedia;
        }, 1000);
      }

      return () => {
        if (timer !== null) window.clearInterval(timer);
        for (const name of MEDIA_EVENTS) {
          if (DEBUG_MEDIA_VERBOSE) video.removeEventListener(name, verboseHandler);
          if (DEBUG_MEDIA) video.removeEventListener(name, countHandler);
        }
      };
    }, [label, buffer.active, src]);

    const onVideoError = (video: HTMLVideoElement, isCurrent: boolean) => {
      const code = video.error?.code ?? 0;
      const message = video.error?.message ?? "";
      const networkState = video.networkState;
      console.error(
        `[${isCurrent ? label : `${label}/preload`}] video error code=${code} ` +
          `networkState=${networkState} src=${video.currentSrc || video.src} ` +
          `message=${message}`,
      );
      if (isCurrent) setError(mediaErrorText(video));
    };

    return (
      <div
        onClick={onClick}
        onDoubleClick={onDoubleClick}
        className={clsx(
          "group relative h-full w-full overflow-hidden rounded-md bg-black",
          (onClick || onDoubleClick) && "cursor-pointer",
        )}
      >
        {([0, 1] as const).map((slot) => {
          const source = buffer.sources[slot];
          const isCurrent = slot === buffer.active;
          return (
            <video
              key={slot}
              ref={setSlotRef(slot)}
              src={source ?? undefined}
              className={clsx(
                "absolute inset-0 h-full w-full object-contain",
                isCurrent ? "z-0 opacity-100" : "pointer-events-none -z-10 opacity-0",
              )}
              muted={!isCurrent || !isMaster}
              preload="auto"
              playsInline
              aria-hidden={!isCurrent}
              onError={(e) => onVideoError(e.currentTarget, isCurrent)}
            />
          );
        })}

        {gapped && (
          <div className="absolute inset-0 flex items-center justify-center bg-black text-xs text-neutral-600">
            camera off
          </div>
        )}

        <div className="absolute left-2 top-2 flex flex-col items-start gap-1">
          <div
            className={clsx(
              "rounded px-2 py-1 text-xs font-medium uppercase tracking-wide backdrop-blur",
              isMaster ? "bg-blue-500/80 text-white" : "bg-black/60 text-neutral-200",
            )}
          >
            {label}
          </div>
          {onClick && (
            <div className="rounded bg-black/60 px-2 py-0.5 text-[10px] text-neutral-300 opacity-0 backdrop-blur transition-opacity group-hover:opacity-100">
              Click to enlarge
            </div>
          )}
          {onDoubleClick && (
            <div className="rounded bg-black/60 px-2 py-0.5 text-[10px] text-neutral-300 opacity-0 backdrop-blur transition-opacity group-hover:opacity-100">
              Double-click for fullscreen
            </div>
          )}
        </div>

        {showLoading && !error && (
          <div className="absolute inset-0 flex items-center justify-center bg-black text-xs text-neutral-500">
            Loading…
          </div>
        )}

        {error && (
          <div className="absolute inset-0 flex flex-col items-center justify-center gap-1 bg-red-950/80 p-4 text-center">
            <div className="text-xs font-semibold uppercase tracking-wide text-red-300">
              {label}
            </div>
            <div className="text-xs text-red-200">{error}</div>
          </div>
        )}
      </div>
    );
  },
);
