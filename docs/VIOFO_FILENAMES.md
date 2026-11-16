# VIOFO filename recognition and firmware evidence

Trip Viewer recognizes each file independently. A single folder may contain
multiple filename variants. There is no firmware-version selector, folder-wide
pattern cache, or requirement to rename original recordings.

## Supported structure

`YYYY_MMDD_HHMMSS_[counter][P]channel.MP4`

| Part | Rule |
| --- | --- |
| Timestamp | Four-digit year, four-digit month/day, six-digit time; valid calendar date and time required |
| Counter | Optional ASCII decimal digits; width is unrestricted and never used for grouping |
| Parking | Optional P immediately before the channel |
| Channel | F = Front, I = Interior, R = Rear |
| Case | MP4 extension and channel/parking letters are case-insensitive |

Owner-provided examples `2025_0706_201953_F.MP4` and
`2025_0706_201953_I.MP4` establish the sequence-free input shape. Existing
fixtures cover numbered F/I/R and numbered parking names. Accepting the
counter-free parking combination follows the same structural grammar; it does
not establish which firmware, if any, emitted that exact combination.

Normal/parking recordings have separate timestamp/mode group keys. A counter
change, omitted counter, different per-channel sequence or letter case does not
change the recording time. Timelapse sibling lookup uses the same recognizer,
and never substitutes a normal recording for a parking sibling. Unknown channels,
invalid dates, extra separators/markers and arbitrary filenames remain rejected.
The change adds no inference of camera channels from video contents.

## What firmware history can establish

The vendor's [firmware page](https://www.viofo.com/pages/firmware) is the reference
for release notes. A changelog can document a naming change only when the entry
actually says so. It is not an exhaustive filename specification unless the
vendor provides one. A video's recording date is not a firmware version, and
counter-free names alone do not prove that older firmware produced them.

Keep a historical review's concrete release entries, source links and retrieval
limits in the fix PR. Do not turn missing/archive-inaccessible notes into an
unqualified claim that all past releases were reviewed or tested. The parser's
format-based support applies across firmware releases that use these shapes;
GPS compatibility additionally depends on the embedded data, not just the name.

## Validation

Auto/manual scans, stable grouping, mixed-format sibling lookup, parking
separation, malformed-input rejection and GPS UTC/filename time anchoring are
covered by native tests. The opt-in FFmpeg test generates HEVC sources using both
filename shapes across all three channels and all three timelapse tiers, then
verifies playable outputs and progress. That does not certify an actual older
camera/firmware recording; use a copy of owner footage for that final check.
