import { ensureTripGps } from "../ipc/gps";
import type { Trip } from "../types/model";
import { useStore } from "./store";

const requests = new Map<string, object>();

/** Show background work without coupling transport to GPS availability. */
export async function prepareTripGps(
  trip: Trip,
  excluded: Record<string, string[]> = {},
  force = false,
): Promise<void> {
  const request = {};
  requests.set(trip.id, request);
  useStore.setState((s) => ({ gpsPreparationByTrip: { ...s.gpsPreparationByTrip, [trip.id]: "loading" } }));
  try {
    const points = await ensureTripGps(trip, excluded, force);
    if (requests.get(trip.id) !== request) return;
    useStore.setState((s) => ({
      tripGpsByTrip: { ...s.tripGpsByTrip, [trip.id]: points },
      gpsPreparationByTrip: { ...s.gpsPreparationByTrip, [trip.id]: "ready" },
    }));
  } catch (error) {
    if (requests.get(trip.id) === request) {
      useStore.setState((s) => ({ gpsPreparationByTrip: { ...s.gpsPreparationByTrip, [trip.id]: "error" } }));
    }
    throw error;
  } finally {
    if (requests.get(trip.id) === request) requests.delete(trip.id);
  }
}
