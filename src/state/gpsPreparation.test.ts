import { beforeEach, expect, it, vi } from "vitest";
import { ensureTripGps } from "../ipc/gps";
import type { GpsPoint, Trip } from "../types/model";
import { prepareTripGps } from "./gpsPreparation";
import { useStore } from "./store";
vi.mock("../ipc/gps", () => ({ ensureTripGps: vi.fn(), loadTripGps: vi.fn() }));
const trip = { id: "trip", segments: [] } as unknown as Trip;
const points = [{ tOffsetS: 1, lat: 1 }] as unknown as GpsPoint[];
beforeEach(() => {
  vi.mocked(ensureTripGps).mockReset();
  useStore.setState({ gpsPreparationByTrip: {}, tripGpsByTrip: {}, isPlaying: true });
});
it("shows background work without pausing transport, then publishes the cached GPS", async () => {
  let complete!: (points: GpsPoint[]) => void;
  vi.mocked(ensureTripGps).mockImplementationOnce(() => new Promise(resolve => { complete = resolve; }));
  const request = prepareTripGps(trip);
  expect(useStore.getState().gpsPreparationByTrip.trip).toBe("loading");
  expect(useStore.getState().isPlaying).toBe(true);
  complete(points);
  await request;
  expect(useStore.getState().gpsPreparationByTrip.trip).toBe("ready");
  expect(useStore.getState().tripGpsByTrip.trip).toEqual(points);
});
it("retains previous GPS and reports a failed refresh without disabling playback", async () => {
  useStore.setState({ tripGpsByTrip: { trip: points } });
  vi.mocked(ensureTripGps).mockRejectedValueOnce(new Error("read failed"));
  await expect(prepareTripGps(trip)).rejects.toThrow("read failed");
  expect(useStore.getState().gpsPreparationByTrip.trip).toBe("error");
  expect(useStore.getState().tripGpsByTrip.trip).toEqual(points);
  expect(useStore.getState().isPlaying).toBe(true);
});
it("keeps the busy state and ignores an older result when another refresh is queued", async () => {
  const complete: ((points: GpsPoint[]) => void)[] = [];
  vi.mocked(ensureTripGps).mockImplementation(() => new Promise(resolve => { complete.push(resolve); }));
  const first = prepareTripGps(trip);
  const second = prepareTripGps(trip, { segment: ["Front"] }, true);
  complete[0](points);
  await first;
  expect(useStore.getState().gpsPreparationByTrip.trip).toBe("loading");
  expect(useStore.getState().tripGpsByTrip.trip).toBeUndefined();
  complete[1]([]);
  await second;
  expect(useStore.getState().gpsPreparationByTrip.trip).toBe("ready");
  expect(useStore.getState().tripGpsByTrip.trip).toEqual([]);
});
