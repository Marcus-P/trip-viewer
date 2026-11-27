import { coverageAt, fileToConcat, type CurveSegment } from "../utils/speedCurve";

/** Convert the requested master position using only known channel curves.
 * No filename clock, camera-model rule or guessed time compression is used.
 */
export function initialChannelPosition(
  target: number,
  masterCurve: CurveSegment[] | null,
  channelCurve: CurveSegment[] | null,
): number {
  if (!masterCurve?.length || !channelCurve?.length) return target;
  return coverageAt(fileToConcat(target, masterCurve), channelCurve).fileTime;
}

/** Position as soon as metadata exists, before waiting for a frame at zero.
 * Return false until a position can actually be requested. A matching position
 * needs no second currentTime assignment and decoder flush.
 */
export function positionMedia(video: HTMLVideoElement, target: number): boolean {
  if (video.readyState < 1 || !Number.isFinite(target)) return false;
  const duration = Number.isFinite(video.duration) ? video.duration : Infinity;
  const clamped = Math.min(Math.max(0, target), duration);
  if (Math.abs(video.currentTime - clamped) > 0.000001) {
    video.currentTime = clamped;
  }
  return true;
}
