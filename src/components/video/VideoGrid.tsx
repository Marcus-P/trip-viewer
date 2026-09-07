import {
  CSSProperties,
  MutableRefObject,
  PointerEvent as ReactPointerEvent,
  useEffect,
  useRef,
  useState,
} from "react";
import type { Segment } from "../../types/model";
import { ChannelPanel } from "./ChannelPanel";
import { useStore } from "../../state/store";
import { videoSrcFor } from "../../utils/videoSrc";

// Both Linux and macOS need the tiny loopback HTTP server
// (src-tauri/src/video_server.rs) for <video> playback, for different
// reasons:
//
//   Linux (WebKitGTK + GStreamer): Tauri's convertFileSrc returns
//     `asset://localhost/...` but WebKitGTK's <video> has no URI handler
//     for the `asset` scheme and fails with FormatError. `file://` URLs
//     are blocked by cross-origin policy between the webview and the
//     filesystem.
//
//   macOS (WKWebView + AVFoundation): the asset:// handler on macOS
//     does not honor HTTP Range requests. Wolfbox MP4s have `moov` at EOF,
//     so without range support AVFoundation linearly buffers ~14 s of mdat
//     before it can start decoding the primary 4K channel.
//
// The Rust server is fully Range-capable (206 Partial Content), so
// whoever has a non-zero videoPort uses HTTP. Windows (WebView2) handles
// the default asset protocol correctly and gets videoPort = 0, falling
// through to convertFileSrc.
const IS_LINUX =
  typeof navigator !== "undefined" &&
  navigator.userAgent.includes("Linux") &&
  !navigator.userAgent.includes("Android");

const IS_MAC =
  typeof navigator !== "undefined" &&
  navigator.userAgent.includes("Mac OS X");

interface Props {
  /** Shared map of label → <video> element, populated by callback refs.
   *  Stable identity across renders so useSyncEngine doesn't re-run. */
  channelRefs: MutableRefObject<Map<string, HTMLVideoElement | null>>;
  activeSegment: Segment | null;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

/**
 * Compute CSS grid placement for a channel panel.
 *
 * Layout philosophy: primary takes col 1 full height; secondaries stack
 * in col 2. Row count adapts to secondary count so each secondary gets
 * the full column width. Works for 1, 2, 3, 4+ channels.
 */
function gridStyle(
  isPrimary: boolean,
  secondaryIndex: number,
  secondaryCount: number,
  rowCount: number,
): CSSProperties {
  if (secondaryCount === 0) {
    // Single channel: fill the whole area.
    return { gridColumn: "1 / 3", gridRow: "1 / 3" };
  }
  if (isPrimary) {
    // Primary occupies col 1, spanning all rows. Using rowCount (not
    // secondaryCount) ensures the primary fills the full grid height even
    // when rowCount > secondaryCount (the 2-channel case, where rowCount=2
    // but secondaryCount=1). Without this, an empty grid row captures
    // pointer events from the transport controls below via a Chromium
    // compositor hit-testing edge case.
    return { gridColumn: 1, gridRow: `1 / ${rowCount + 1}` };
  }
  // Secondary cell: col 2, one row per secondary.
  return { gridColumn: 2, gridRow: secondaryIndex + 1 };
}

export function VideoGrid({ channelRefs, activeSegment }: Props) {
  const primaryChannel = useStore((s) => s.primaryChannel);
  const setPrimaryChannel = useStore((s) => s.setPrimaryChannel);
  const videoPort = useStore((s) => s.videoPort);
  const sourceMode = useStore((s) => s.sourceMode);
  const isPlaying = useStore((s) => s.isPlaying);
  const trips = useStore((s) => s.trips);
  const loadedTripId = useStore((s) => s.loadedTripId);
  const gridRef = useRef<HTMLDivElement | null>(null);
  const contextMenuRef = useRef<HTMLDivElement | null>(null);
  const [dashboardFullscreen, setDashboardFullscreen] = useState(false);
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);

  // Layout ratios intentionally start at the existing 2:1 / 3:1 proportions.
  // Persistence is added separately after the interaction model is validated.
  const [primaryShare, setPrimaryShare] = useState(2 / 3);
  const [secondarySplit, setSecondarySplit] = useState(0.5);
  const [mapShare, setMapShare] = useState(0.25);
  const [timelineHeightPx, setTimelineHeightPx] = useState<number | null>(null);
  const [hasMapPanel, setHasMapPanel] = useState(false);

  // On first render of a segment (or when primaryChannel is null from a
  // trip/segment change), initialize the visual primary to the first channel
  // in canonical order. The sync engine keeps its own stable canonical master;
  // changing the visual primary does not rebuild or retarget that engine.
  useEffect(() => {
    if (!activeSegment) return;
    const master = activeSegment.channels[0]?.label ?? null;
    if (!master) return;
    // If primaryChannel is stale (references a label no longer in the
    // segment, e.g. after switching from 3-channel Wolf Box to 2-channel
    // Thinkware), reset it.
    const valid = activeSegment.channels.some((c) => c.label === primaryChannel);
    if (!valid) setPrimaryChannel(master);
  }, [activeSegment, primaryChannel, setPrimaryChannel]);

  // The outer PlayerShell grid owns the GPS column. VideoGrid spans its first
  // two columns, so an inline template lets this component expose a draggable
  // video↔map boundary without changing playback ownership or the fullscreen
  // container. When no map is present, restore PlayerShell's normal template.
  useEffect(() => {
    const grid = gridRef.current;
    const viewingArea = grid?.parentElement;
    if (!grid || !viewingArea) return;
    const sibling = grid.nextElementSibling as HTMLElement | null;
    const mapPresent = Boolean(sibling?.querySelector(".leaflet-container"));
    setHasMapPanel(mapPresent);
    if (!mapPresent) {
      viewingArea.style.gridTemplateColumns = "";
      return;
    }

    viewingArea.style.gridTemplateColumns =
      `minmax(0, ${1 - mapShare}fr) 0px minmax(180px, ${mapShare}fr)`;
  }, [activeSegment, mapShare]);

  // Timeline height is inherited as a CSS custom property. This keeps the
  // lower strip structurally identical and lets the flexing video area give
  // up or reclaim space naturally when the user drags the horizontal handle.
  useEffect(() => {
    const shell = gridRef.current?.parentElement?.parentElement;
    if (!shell) return;
    if (timelineHeightPx === null) {
      shell.style.removeProperty("--tripviewer-timeline-height");
    } else {
      shell.style.setProperty(
        "--tripviewer-timeline-height",
        `${timelineHeightPx}px`,
      );
    }
  }, [timelineHeightPx]);

  // Track whether the *whole viewing area* (VideoGrid + GPS map) is the
  // browser fullscreen element. VideoGrid's parent is PlayerShell's main
  // viewing grid, so requesting fullscreen on that parent keeps F/I/R and
  // GPS together while the ordinary transport/timeline chrome stays out.
  useEffect(() => {
    const onFullscreenChange = () => {
      const viewingArea = gridRef.current?.parentElement ?? null;
      setDashboardFullscreen(
        Boolean(viewingArea && document.fullscreenElement === viewingArea),
      );
      setContextMenu(null);
    };
    document.addEventListener("fullscreenchange", onFullscreenChange);
    onFullscreenChange();
    return () => document.removeEventListener("fullscreenchange", onFullscreenChange);
  }, []);

  // Replace the generic webview menu over the viewing area with the small set
  // of actions that is useful during dashcam playback. Keep Reload available,
  // and add playback + dashboard fullscreen so those actions remain reachable
  // even while the normal transport bar is outside the fullscreen element.
  useEffect(() => {
    const viewingArea = gridRef.current?.parentElement;
    if (!viewingArea || !activeSegment) return;

    const onContextMenu = (event: MouseEvent) => {
      event.preventDefault();
      setContextMenu({ x: event.clientX, y: event.clientY });
    };

    viewingArea.addEventListener("contextmenu", onContextMenu);
    return () => {
      viewingArea.removeEventListener("contextmenu", onContextMenu);
      setContextMenu(null);
    };
  }, [activeSegment]);

  useEffect(() => {
    if (!contextMenu) return;

    const onPointerDown = (event: PointerEvent) => {
      if (
        contextMenuRef.current &&
        event.target instanceof Node &&
        contextMenuRef.current.contains(event.target)
      ) {
        return;
      }
      setContextMenu(null);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setContextMenu(null);
    };

    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    window.addEventListener("blur", () => setContextMenu(null), { once: true });
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [contextMenu]);

  // Native fullscreen can suspend or pause video pipelines that are outside
  // the fullscreen element on WebKit-based platforms. In Original mode every
  // channel shares the same segment-local time axis, so the video the user was
  // actually watching in fullscreen is the best deterministic anchor when
  // fullscreen ends. Reposition all channels exactly once on the
  // `fullscreenchange` event and resume them if global playback is active.
  //
  // This is deliberately event-driven rather than timer-driven. Tiered modes
  // are excluded because gappy channels can live on different file-time axes;
  // those require the SyncEngine's curve mapping rather than direct equality.
  useEffect(() => {
    let fullscreenVideo: HTMLVideoElement | null = null;

    const onFullscreenChange = () => {
      const current = document.fullscreenElement;
      if (current instanceof HTMLVideoElement) {
        fullscreenVideo = current;
        return;
      }

      if (!fullscreenVideo) return;
      const exitedVideo = fullscreenVideo;
      fullscreenVideo = null;

      if (sourceMode !== "original") return;
      const anchorTime = exitedVideo.currentTime;
      if (!Number.isFinite(anchorTime)) return;

      const state = useStore.getState();
      for (const video of channelRefs.current.values()) {
        if (!video) continue;
        video.playbackRate = state.speed;
        video.currentTime = anchorTime;
      }
      state.setCurrentTime(anchorTime);

      if (state.isPlaying) {
        for (const video of channelRefs.current.values()) {
          if (!video || video.ended) continue;
          video.play().catch(() => {});
        }
      }
    };

    document.addEventListener("fullscreenchange", onFullscreenChange);
    return () => document.removeEventListener("fullscreenchange", onFullscreenChange);
  }, [channelRefs, sourceMode]);

  if ((IS_LINUX || IS_MAC) && !videoPort) {
    return (
      <div className="col-span-2 flex items-center justify-center text-sm text-neutral-500">
        Starting video server…
      </div>
    );
  }

  if (!activeSegment) {
    return (
      <div className="col-span-2 flex items-center justify-center text-sm text-neutral-500">
        Select a trip from the list to begin playback.
      </div>
    );
  }

  // Always use canonical (Rust-sorted) order. The store's `primaryChannel`
  // label just tells us which of the rendered panels gets the primary
  // slot — it doesn't change tree order.
  const channels = activeSegment.channels;
  const effectivePrimary =
    channels.find((c) => c.label === primaryChannel)?.label ??
    channels[0]?.label;

  const secondaries = channels.filter((c) => c.label !== effectivePrimary);

  // Original-mode continuity gets one-segment look-ahead. ChannelPanel keeps
  // that next file in its paused standby media element. Tiered playback is one
  // stitched file, and tombstones deliberately switch source mode instead of
  // trying to preload a nonexistent original segment.
  let preloadSegment: Segment | null = null;
  if (sourceMode === "original") {
    const activeTrip = trips.find((candidate) => candidate.id === loadedTripId);
    const segmentIndex = activeTrip?.segments.findIndex(
      (segment) => segment.id === activeSegment.id,
    ) ?? -1;
    const next = segmentIndex >= 0 ? activeTrip?.segments[segmentIndex + 1] : null;
    if (next && next.isTombstone !== true && next.channels.length > 0) {
      preloadSegment = next;
    }
  }

  function setRef(label: string) {
    return (node: HTMLVideoElement | null) => {
      if (node) {
        channelRefs.current.set(label, node);
      } else {
        channelRefs.current.delete(label);
      }
    };
  }

  function togglePlayback() {
    // TransportControls owns the SyncEngine instance. Dispatching this app
    // event lets fullscreen UI use the exact same pause/play path as the
    // ordinary transport bar and keyboard shortcut.
    window.dispatchEvent(new Event("tripviewer:toggle-playback"));
  }

  function startPointerDrag(
    event: ReactPointerEvent<HTMLDivElement>,
    cursor: "col-resize" | "row-resize",
    onMove: (event: PointerEvent) => void,
  ) {
    event.preventDefault();
    event.stopPropagation();
    const oldUserSelect = document.body.style.userSelect;
    const oldCursor = document.body.style.cursor;
    document.body.style.userSelect = "none";
    document.body.style.cursor = cursor;

    const move = (pointerEvent: PointerEvent) => {
      pointerEvent.preventDefault();
      onMove(pointerEvent);
    };
    const finish = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", finish);
      window.removeEventListener("pointercancel", finish);
      document.body.style.userSelect = oldUserSelect;
      document.body.style.cursor = oldCursor;
    };

    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", finish);
    window.addEventListener("pointercancel", finish);
  }

  function resizePrimarySecondary(event: ReactPointerEvent<HTMLDivElement>) {
    const rect = gridRef.current?.getBoundingClientRect();
    if (!rect || rect.width <= 0) return;
    startPointerDrag(event, "col-resize", (pointerEvent) => {
      const share = (pointerEvent.clientX - rect.left) / rect.width;
      setPrimaryShare(clamp(share, 0.35, 0.85));
    });
  }

  function resizeSecondaryStack(event: ReactPointerEvent<HTMLDivElement>) {
    const rect = gridRef.current?.getBoundingClientRect();
    if (!rect || rect.height <= 0) return;
    startPointerDrag(event, "row-resize", (pointerEvent) => {
      const share = (pointerEvent.clientY - rect.top) / rect.height;
      setSecondarySplit(clamp(share, 0.2, 0.8));
    });
  }

  function resizeMap(event: ReactPointerEvent<HTMLDivElement>) {
    const viewingArea = gridRef.current?.parentElement;
    const rect = viewingArea?.getBoundingClientRect();
    if (!rect || rect.width <= 0) return;
    startPointerDrag(event, "col-resize", (pointerEvent) => {
      const share = (rect.right - pointerEvent.clientX) / rect.width;
      setMapShare(clamp(share, 0.15, 0.5));
    });
  }

  function resizeTimeline(event: ReactPointerEvent<HTMLDivElement>) {
    const shell = gridRef.current?.parentElement?.parentElement;
    const timeline = shell?.querySelector(
      "[data-tripviewer-timeline]",
    ) as HTMLElement | null;
    if (!shell || !timeline) return;
    const startY = event.clientY;
    const startHeight = timeline.getBoundingClientRect().height;
    const maxHeight = Math.max(96, shell.getBoundingClientRect().height * 0.45);
    startPointerDrag(event, "row-resize", (pointerEvent) => {
      const next = startHeight + startY - pointerEvent.clientY;
      setTimelineHeightPx(clamp(next, 56, maxHeight));
    });
  }

  function handleMainDoubleClick() {
    const el = channelRefs.current.get(effectivePrimary);
    if (!el) return;

    // If a single camera is already fullscreen, pop that fullscreen layer.
    // When it was entered from the dashboard, the dashboard remains as the
    // underlying fullscreen element, so this returns directly to F/I/R + GPS.
    if (document.fullscreenElement instanceof HTMLVideoElement) {
      void document.exitFullscreen();
      return;
    }

    // The dashboard viewing area may itself already be fullscreen. The
    // Fullscreen API supports putting a descendant on top of that fullscreen
    // element, so request the selected video directly instead of first leaving
    // the dashboard. This preserves the dashboard underneath for the return.
    void el.requestFullscreen();
  }

  async function toggleDashboardFullscreen() {
    const viewingArea = gridRef.current?.parentElement;
    if (!viewingArea) return;
    if (document.fullscreenElement === viewingArea) {
      await document.exitFullscreen();
      return;
    }
    if (document.fullscreenElement instanceof HTMLVideoElement) {
      await document.exitFullscreen();
      if (document.fullscreenElement === viewingArea) return;
    }
    if (document.fullscreenElement) {
      await document.exitFullscreen();
    }
    await viewingArea.requestFullscreen();
  }

  // Row template: if primary takes full height and there are N
  // secondaries, we need N rows. Minimum of 2 rows for aesthetic
  // symmetry when there's only 1 secondary.
  const rowCount = Math.max(secondaries.length, 2);
  const gridTemplateRows =
    secondaries.length === 2
      ? `${secondarySplit}fr ${1 - secondarySplit}fr`
      : `repeat(${rowCount}, minmax(0, 1fr))`;

  return (
    <div
      ref={gridRef}
      className="relative col-span-2 grid gap-2"
      style={{
        gridTemplateColumns: `${primaryShare}fr ${1 - primaryShare}fr`,
        gridTemplateRows,
      }}
    >
      <div className="absolute right-2 top-2 z-20 flex items-center gap-2">
        {dashboardFullscreen && (
          <button
            type="button"
            onClick={togglePlayback}
            className="rounded bg-blue-600/90 px-3 py-1.5 text-xs font-medium text-white backdrop-blur transition-colors hover:bg-blue-500"
            title={isPlaying ? "Pause playback" : "Resume playback"}
          >
            {isPlaying ? "Pause" : "Play"}
          </button>
        )}
        <button
          type="button"
          onClick={() => void toggleDashboardFullscreen()}
          className="rounded bg-black/70 px-2.5 py-1.5 text-xs font-medium text-neutral-100 opacity-70 backdrop-blur transition-opacity hover:opacity-100"
          title={
            dashboardFullscreen
              ? "Exit F/I/R + GPS fullscreen"
              : "Fullscreen F/I/R + GPS"
          }
        >
          {dashboardFullscreen ? "Exit dashboard" : "F/I/R + GPS fullscreen"}
        </button>
      </div>

      {secondaries.length > 0 && (
        <div
          role="separator"
          aria-orientation="vertical"
          title="Drag to resize main and secondary cameras"
          onPointerDown={resizePrimarySecondary}
          className="absolute bottom-0 top-0 z-30 w-3 -translate-x-1/2 cursor-col-resize"
          style={{ left: `${primaryShare * 100}%` }}
        >
          <div className="mx-auto h-full w-px bg-neutral-500/0 transition-colors hover:bg-neutral-400/70" />
        </div>
      )}

      {secondaries.length === 2 && (
        <div
          role="separator"
          aria-orientation="horizontal"
          title="Drag to resize the secondary cameras"
          onPointerDown={resizeSecondaryStack}
          className="absolute right-0 z-30 h-3 -translate-y-1/2 cursor-row-resize"
          style={{
            left: `${primaryShare * 100}%`,
            top: `${secondarySplit * 100}%`,
          }}
        >
          <div className="my-auto h-px w-full bg-neutral-500/0 transition-colors hover:bg-neutral-400/70" />
        </div>
      )}

      {hasMapPanel && (
        <div
          role="separator"
          aria-orientation="vertical"
          title="Drag to resize video and GPS map"
          onPointerDown={resizeMap}
          className="absolute -right-2 bottom-0 top-0 z-30 w-4 cursor-col-resize"
        >
          <div className="mx-auto h-full w-px bg-neutral-500/0 transition-colors hover:bg-neutral-400/70" />
        </div>
      )}

      {!dashboardFullscreen && (
        <div
          role="separator"
          aria-orientation="horizontal"
          title="Drag to resize timeline and playback controls"
          onPointerDown={resizeTimeline}
          className="absolute -bottom-2 left-0 right-0 z-30 h-4 cursor-row-resize"
        >
          <div className="my-auto h-px w-full bg-neutral-500/0 transition-colors hover:bg-neutral-400/70" />
        </div>
      )}

      {contextMenu && (
        <div
          ref={contextMenuRef}
          className="fixed z-50 min-w-[13rem] overflow-hidden rounded-md border border-neutral-700 bg-neutral-900 py-1 shadow-xl"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          role="menu"
        >
          <button
            type="button"
            onClick={() => {
              setContextMenu(null);
              togglePlayback();
            }}
            className="block w-full px-3 py-2 text-left text-sm text-neutral-100 hover:bg-neutral-800"
            role="menuitem"
          >
            {isPlaying ? "Pause" : "Play"}
          </button>
          <button
            type="button"
            onClick={() => {
              setContextMenu(null);
              void toggleDashboardFullscreen();
            }}
            className="block w-full px-3 py-2 text-left text-sm text-neutral-100 hover:bg-neutral-800"
            role="menuitem"
          >
            {dashboardFullscreen
              ? "Exit F/I/R + GPS fullscreen"
              : "F/I/R + GPS fullscreen"}
          </button>
          <div className="my-1 border-t border-neutral-700" />
          <button
            type="button"
            onClick={() => window.location.reload()}
            className="block w-full px-3 py-2 text-left text-sm text-neutral-300 hover:bg-neutral-800 hover:text-white"
            role="menuitem"
          >
            Reload
          </button>
        </div>
      )}

      {channels.map((channel) => {
        const isPrimary = channel.label === effectivePrimary;
        const idx = isPrimary
          ? 0
          : secondaries.findIndex((c) => c.label === channel.label);
        const preloadChannel = preloadSegment?.channels.find(
          (candidate) => candidate.label === channel.label,
        );

        return (
          <div
            key={channel.label}
            style={gridStyle(isPrimary, idx, secondaries.length, rowCount)}
          >
            <ChannelPanel
              ref={setRef(channel.label)}
              label={channel.label}
              src={videoSrcFor(channel.filePath, videoPort)}
              preloadSrc={
                preloadChannel
                  ? videoSrcFor(preloadChannel.filePath, videoPort)
                  : null
              }
              isMaster={isPrimary}
              onClick={isPrimary ? undefined : () => setPrimaryChannel(channel.label)}
              onDoubleClick={isPrimary ? handleMainDoubleClick : undefined}
            />
          </div>
        );
      })}
    </div>
  );
}
