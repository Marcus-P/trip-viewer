#!/usr/bin/env bash
set -euo pipefail

APP_ID="com.tripviewer.app"
APP_NAME="Trip Viewer"
INSTALL_DIR="${HOME}/.local/opt/trip-viewer"
APPLICATIONS_DIR="${HOME}/.local/share/applications"
ICONS_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

APPIMAGE="${1:-}"
if [[ -z "${APPIMAGE}" ]]; then
  APPIMAGE="$(find "${SCRIPT_DIR}" -maxdepth 1 -type f -name 'Trip.Viewer_*_Ubuntu-24.04_amd64.AppImage' -print -quit)"
fi
if [[ -z "${APPIMAGE}" || ! -f "${APPIMAGE}" ]]; then
  echo "Usage: $0 /path/to/Trip.Viewer_X.Y.Z_Ubuntu-24.04_amd64.AppImage" >&2
  exit 2
fi

APPIMAGE="$(realpath "${APPIMAGE}")"
mkdir -p "${INSTALL_DIR}" "${APPLICATIONS_DIR}" "${ICONS_DIR}"
DEST="${INSTALL_DIR}/Trip.Viewer.AppImage"
cp "${APPIMAGE}" "${DEST}"
chmod +x "${DEST}"

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT
(
  cd "${TMP}"
  "${DEST}" --appimage-extract >/dev/null
)

ROOT="${TMP}/squashfs-root"
DESKTOP_SRC="$(find "${ROOT}/usr/share/applications" -maxdepth 1 -type f -name '*.desktop' -print -quit 2>/dev/null || true)"
if [[ -z "${DESKTOP_SRC}" ]]; then
  echo "Could not find the desktop entry inside the AppImage." >&2
  exit 1
fi

DESKTOP_DEST="${APPLICATIONS_DIR}/${APP_ID}.desktop"
cp "${DESKTOP_SRC}" "${DESKTOP_DEST}"
sed -i -E "s|^Exec=.*|Exec=${DEST}|" "${DESKTOP_DEST}"
if grep -q '^Icon=' "${DESKTOP_DEST}"; then
  sed -i -E 's|^Icon=.*|Icon=trip-viewer|' "${DESKTOP_DEST}"
else
  printf '\nIcon=trip-viewer\n' >> "${DESKTOP_DEST}"
fi
chmod 0644 "${DESKTOP_DEST}"

ICON_SRC="$(find "${ROOT}/usr/share/icons/hicolor/256x256" -type f -name '*.png' -print -quit 2>/dev/null || true)"
if [[ -z "${ICON_SRC}" ]]; then
  ICON_SRC="$(find "${ROOT}/usr/share/icons/hicolor" -type f -name '*.png' -print -quit 2>/dev/null || true)"
fi
if [[ -n "${ICON_SRC}" ]]; then
  cp "${ICON_SRC}" "${ICONS_DIR}/trip-viewer.png"
  chmod 0644 "${ICONS_DIR}/trip-viewer.png"
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${APPLICATIONS_DIR}" >/dev/null 2>&1 || true
fi

echo
echo "${APP_NAME} installed for this user."
echo "Application: ${DEST}"
echo "Launcher:    ${DESKTOP_DEST}"
echo
echo "Open the Ubuntu application overview, start Trip Viewer once, then"
echo "right-click its Dock icon and choose 'Add to Favorites'."
