# Third-party notices

Trip Viewer itself is distributed under the repository's [MIT License](LICENSE). The MIT license does not relicense third-party software. Every dependency keeps its own license.

This file is an engineering/license inventory, not legal advice and not a warranty of legal compliance. Each GitHub Release also contains generated Node and Rust dependency inventories with exact package versions and declared license expressions.

## Frontend runtime dependencies

The exact versions are pinned by `package-lock.json`.

| Component | Declared license |
| --- | --- |
| React / React DOM | MIT |
| Zustand | MIT |
| clsx | MIT |
| Leaflet | BSD-2-Clause |
| React Leaflet / @react-leaflet/core | Hippocratic License 2.1 |
| Tauri JavaScript API/plugins used by the app | MIT and/or Apache-2.0 |

React Leaflet 5.0.0 declares **Hippocratic-2.1**, not MIT. Its license terms therefore apply to redistribution and use of that component independently of Trip Viewer's MIT license:
https://github.com/PaulLeCam/react-leaflet/blob/master/LICENSE.md

Leaflet's license:
https://github.com/Leaflet/Leaflet/blob/main/LICENSE

## Rust runtime dependencies

The Rust backend uses Tauri and Tauri plugins, chrono, mp4, rayon, serde, serde_json, sha2, rusqlite, rusqlite_migration, symphonia, thiserror, trash, uuid, walkdir, GTK bindings and platform-specific support crates. These crates are not relicensed by Trip Viewer. Their exact resolved versions and declared license expressions are emitted from `cargo metadata` into the release asset `RUST_DEPENDENCY_LICENSES.txt`.

Dependency license expressions are not uniformly MIT-only. For example, Symphonia is distributed under MPL-2.0 and many Rust ecosystem crates use MIT OR Apache-2.0.

## Linux media and system components

The Ubuntu AppImage uses the Linux media stack used by Tauri/WebKitGTK and GStreamer. Depending on AppImage bundling and the host system, media/system libraries may be loaded from the bundle or from the operating system. Those libraries retain their own licenses and are not covered by Trip Viewer's MIT license.

## Icons and project assets

The Trip Viewer icon files are inherited from the upstream MIT-licensed repository. No separate asset license notice was found in this repository. If an upstream asset owner later publishes a separate license or attribution requirement, that notice should be added here before the next release.

## Compatibility names

"VIOFO" and "A229 Pro" are used descriptively to identify compatible hardware. No VIOFO logo, firmware, proprietary application code or other VIOFO asset is included.
