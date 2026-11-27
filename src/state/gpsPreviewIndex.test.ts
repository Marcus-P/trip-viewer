import { expect, it } from "vitest";
import { indexGpsPreview } from "./gpsPreviewIndex";
import type { GpsPoint } from "../types/model";

const point = (time: number) => ({ tOffsetS: time } as GpsPoint);

it("assigns exact boundaries to the next nonempty segment without dropping input order", () => {
  const points = [point(19), point(0), point(10), point(9), point(20), point(-1), point(NaN)];
  const result = indexGpsPreview(points, [10, 0, 10]);
  expect(result).toEqual([[points[1], points[3]], [], [points[0], points[2]]]);
  expect(result[0][0]).toBe(points[1]);
});

it("reads each cached timestamp once rather than rescanning the track per segment", () => {
  let reads = 0;
  const points = Array.from({ length: 300 }, (_, i) => ({
    get tOffsetS() { reads++; return i; },
  } as GpsPoint));
  const result = indexGpsPreview(points, Array(100).fill(3));
  expect(result.flat()).toEqual(points);
  expect(reads).toBe(300);
});

it("does not infer a timeline from invalid durations", () => {
  expect(indexGpsPreview([point(1)], [10, -1])).toEqual([[], []]);
  expect(indexGpsPreview([point(1)], [NaN])).toEqual([[]]);
  expect(indexGpsPreview([point(1)], [])).toEqual([]);
});
