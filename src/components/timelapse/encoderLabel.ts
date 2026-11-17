import type { FfmpegCapabilities } from "../../ipc/timelapse";

export function encoderLabel(encoder: string): string {
  return ({
    hevc_nvenc: "NVIDIA / NVENC", hevc_amf: "AMD / AMF",
    hevc_qsv: "Intel / QSV", hevc_vaapi: "GPU / VA-API", libx265: "CPU / HEVC",
  } as Record<string, string>)[encoder] ?? encoder;
}

export function verifiedEncoderLabel(caps: FfmpegCapabilities): string {
  return encoderLabel(caps.nvencHevc ? "hevc_nvenc" : caps.amfHevc ? "hevc_amf"
    : caps.qsvHevc ? "hevc_qsv" : caps.vaapiDevice ? "hevc_vaapi" : "libx265");
}

export function decoderLabel(decoder?: string): string {
  return ({ software: "CPU", cuda: "NVIDIA / CUDA", d3d11va: "GPU / D3D11VA",
    vaapi: "GPU / VA-API", videotoolbox: "GPU / VideoToolbox" } as Record<string, string>)[decoder ?? "software"] ?? decoder ?? "CPU";
}
