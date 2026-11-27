import { expect, it } from "vitest";
import { initialChannelPosition, positionMedia } from "./mediaPosition";
import type { CurveSegment } from "../utils/speedCurve";

class Media {
  readyState = 0;
  duration = 120;
  time = 0;
  writes: number[] = [];
  get currentTime() { return this.time; }
  set currentTime(value: number) { this.time = value; this.writes.push(value); }
  video() { return this as unknown as HTMLVideoElement; }
}

it("positions at metadata readiness without decoding frame zero first or seeking twice", () => {
  const media = new Media();
  expect(positionMedia(media.video(), 42)).toBe(false);
  expect(media.writes).toEqual([]);
  media.readyState = 1;
  expect(positionMedia(media.video(), 42)).toBe(true);
  media.readyState = 2;
  expect(positionMedia(media.video(), 42)).toBe(true);
  expect(media.writes).toEqual([42]);
});

it("bounds requests by actual duration and refuses unknown targets", () => {
  const media = new Media();
  media.readyState = 1;
  positionMedia(media.video(), 200);
  expect(media.currentTime).toBe(120);
  expect(positionMedia(media.video(), NaN)).toBe(false);
  expect(media.writes).toEqual([120]);
});

it("uses individual channel curves rather than applying a master file time to all cameras", () => {
  const master = [{ concatStart: 0, concatEnd: 80, rate: 8 }] as CurveSegment[];
  const rear = [{ concatStart: 40, concatEnd: 80, rate: 8 }] as CurveSegment[];
  expect(initialChannelPosition(8, master, rear)).toBe(3);
  expect(initialChannelPosition(8, null, null)).toBe(8);
});
