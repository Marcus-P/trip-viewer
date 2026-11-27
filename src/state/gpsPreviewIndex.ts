import type { GpsPoint } from "../types/model";

/** Partition an existing trip track once, independently of the camera decoder.
 * Intervals match the trip's concatenated segment clock: [start, end).
 * Retain input order and point objects, including when the input is unsorted.
 */
export function indexGpsPreview(
  points: readonly GpsPoint[],
  durations: readonly number[],
): GpsPoint[][] {
  const result = durations.map(() => [] as GpsPoint[]);
  const ends: number[] = [];
  let end = 0;
  for (const duration of durations) {
    // Invalid durations cannot define a trustworthy shared time axis.
    if (!Number.isFinite(duration) || duration < 0) return result;
    end += duration;
    if (!Number.isFinite(end)) return result;
    ends.push(end);
  }
  for (const point of points) {
    const time = point.tOffsetS;
    if (!Number.isFinite(time) || time < 0) continue;
    let low = 0;
    let high = ends.length;
    while (low < high) {
      const middle = Math.floor((low + high) / 2);
      if (time >= ends[middle]) low = middle + 1;
      else high = middle;
    }
    if (low < result.length) result[low].push(point);
  }
  return result;
}
