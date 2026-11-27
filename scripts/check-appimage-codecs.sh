#!/usr/bin/env bash
# Exercise the media framework actually shipped in the AppImage, not host plugins.
set -euo pipefail
image=$(realpath -- "${1:?Pass the built Ubuntu AppImage}")
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
(cd "$work" && "$image" --appimage-extract >/dev/null)
export APPDIR="$work/squashfs-root"
for plugin in libgstlibav.so libgstisomp4.so libgstvideoparsersbad.so; do
    if [ ! -s "$APPDIR/usr/lib/gstreamer-1.0/$plugin" ]; then
        echo "Missing bundled media plugin: $plugin" >&2
        exit 1
    fi
done
# Produce two synthetic frames per codec. No recordings or personal data.
ffmpeg -v error -f lavfi -i color=c=black:s=64x64:r=2 -frames:v 2 -pix_fmt yuv420p -c:v libx264 "$work/h264.mp4"
ffmpeg -v error -f lavfi -i color=c=black:s=64x64:r=2 -frames:v 2 -pix_fmt yuv420p -c:v libx265 -x265-params 'pools=1:frame-threads=1:log-level=error' -tag:v hvc1 "$work/h265.mp4"
export LD_LIBRARY_PATH="$APPDIR/usr/lib:${LD_LIBRARY_PATH:-}"
export GST_REGISTRY_1_0="$work/registry.bin"
source "$APPDIR/apprun-hooks/linuxdeploy-plugin-gstreamer.sh"
for codec in h264 h265; do
    timeout 30 gst-inspect-1.0 "avdec_$codec" >/dev/null
    timeout 30 gst-launch-1.0 -q filesrc location="$work/$codec.mp4" ! qtdemux ! "${codec}parse" ! "avdec_$codec" ! videoconvert ! video/x-raw,format=RGB ! fakesink sync=false
done
echo 'Bundled H.264 and H.265 decoding passed (synthetic software test).'
