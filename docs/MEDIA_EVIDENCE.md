# Media evidence and parking follow-up

This feature branch first incorporates current upstream 70mai support, retaining
its original commits. It reuses the small P-marked parking GPS correction from
the separate PR #21 candidate. The existing full-fork archive schema is unchanged.

## Additional, separately portable module

`metadata/evidence.rs` inspects already parsed MP4 headers and sample tables.
It reports timescale, decode-duration sums, sample counts, composition offsets,
edit-list presence, sample-size table consistency and sync-sample bounds.
Arithmetic overflow is explicit; fragmented sample tables are not evaluated by
the nonfragmented checks. Header/decode-duration differences are observations,
not automatic corruption verdicts. No frames are decoded and no file is hashed.

The ordinary probe reuses its existing parsed metadata. Zero video timescale is
rejected before duration arithmetic; other findings are logged as reason codes.
`inspect_media_evidence` exposes the same structured report as a Tauri command.
It returns no paths, recording dates, GPS coordinates or arbitrary vendor values.
This first module does not implement every metadata idea from the investigation
or infer a parking submode, capture rate, hardware capability or camera pairing.

The module and its adapters are a separate commit suitable for a later upstream
contribution. They are deliberately absent from the PR #21 parking candidate.

## Isolated owner test

The explicit `isolated-evidence` packaging option uses
`tauri.evidence-test.conf.json`: separate product name, application identifier,
settings and disabled updater endpoints. Production configuration is unchanged.
The Windows installer is an acceptance prerelease below the next stable version;
it is not stable update metadata. Open the two existing parking test folders as
separate archives, not a production archive. No new recording is required.

For source builds after installing the repository's native prerequisites:

```sh
npm ci
npm run tauri -- build --no-bundle --config src-tauri/tauri.evidence-test.conf.json
```

Parking is represented by one validated observed file location, not a fabricated
GPS-to-frame time scaling or a proven stationary trace. Five-second filename/
channel alignment and parking trip-boundary product decisions remain separate.
