import { expect, test } from "vitest";
import { decoderLabel, encoderLabel, verifiedEncoderLabel } from "./encoderLabel";

test("verified encoder display agrees with native preference and old capability payloads", () => {
  const base = { version: "ffmpeg", nvencHevc: false };
  expect(verifiedEncoderLabel(base)).toBe("CPU / HEVC");
  expect(verifiedEncoderLabel({ ...base, amfHevc: true })).toBe("AMD / AMF");
  expect(verifiedEncoderLabel({ ...base, vaapiDevice: "/dev/dri/renderD129" })).toBe("GPU / VA-API");
  expect(verifiedEncoderLabel({ ...base, nvencHevc: true, amfHevc: true })).toBe("NVIDIA / NVENC");
  expect(verifiedEncoderLabel({ ...base, qsvHevc: true })).toBe("Intel / QSV");
  expect(verifiedEncoderLabel({ ...base, qsvHevc: true, amfHevc: true })).toBe("AMD / AMF");
  expect(encoderLabel("hevc_qsv")).toBe("Intel / QSV");
  expect(encoderLabel("hevc_amf")).toBe("AMD / AMF");
  expect(encoderLabel("other_encoder")).toBe("other_encoder");
});


test("decoder labels distinguish GPU decoding from encoding and default old payloads to CPU", () => {
  expect(decoderLabel()).toBe("CPU");
  expect(decoderLabel("software")).toBe("CPU");
  expect(decoderLabel("d3d11va")).toBe("GPU / D3D11VA");
  expect(decoderLabel("vaapi")).toBe("GPU / VA-API");
  expect(decoderLabel("cuda")).toBe("NVIDIA / CUDA");
  expect(decoderLabel("videotoolbox")).toBe("GPU / VideoToolbox");
});
