# Changelog

## 0.4.0 - 2026-11-10

Non-finite floats anywhere in details are now emitted as the strings `"NaN"`, `"Infinity"` or `"-Infinity"`.

Numeric guidance now compares `left as f64 - right as f64`, the difference the
platform itself computes from the operands it receives, instead of an exact
difference converted afterwards. Previously a pair whose exact difference
improved but whose converted difference did not — `9007199254740992` and
`9007199254740993` are the same `f64` — moved the watermark and emitted a
guidance event carrying nothing new.

Rich assertions (`assert_always_greater_than!`, `assert_always_some!`, ...)
whose `details` fail to serialize now emit no `details` key, instead of
emitting the generated `left`/`right` or named-bool fields after the
`antithesis_error`.

A `details` entry whose key a rich assertion *generates* no longer fails the
bag. The merge overwrites it, so it is now skipped rather than serialized:
`assert_always_greater_than!(2, 1, "m", &bag)` where `bag.right` cannot
serialize now emits the rest of `bag` with `left`/`right` laid over, where it
previously reported an error and emitted nothing.

## 0.3.0 - 2026-08-28

Add a `catalog` module exposing the assertion catalog compiled into the
binary. `catalog::assertions()` iterates the assertion declarations (id,
message, kind, source location) without executing the workload or calling
`antithesis_init()`; `catalog::write(out)` writes them in the format `antithesis_init()` emits.

Fix overly broad synchronization in `assert_raw`.

Remove small modulo bias from `random_choice`.

When used in local debug mode, the output file (`ANTITHESIS_SDK_LOCAL_OUTPUT`) will no longer be truncated at initialization.

`lifecycle::send_event` no longer trims event names or substitutes
"anonymous" for empty ones: every other SDK passes the name through
verbatim, so the same program emitted different events depending on which
SDK it was written against. Surfaced by the cross-SDK conformance harness
(star: sdk/conformance). Version bumped to 0.3.0.

## 0.2.9 - 2026-06-12

Support `rand` 0.8/0.9/0.10 via version-specific feature flags (`rand_v0_8`, `rand_v0_9`, `rand_v0_10`).

## 0.2.8 - 2026-02-09

Reduce verbosity of guidance tracking. The SDK now only emits guidance events when a value strictly exceeds the previous tracked min/max, rather than on equal values too.

## 0.2.7 - 2026-01-13

Major performance improvements so that serialization occurs lazily and unnecessary copies are elimated.

`details` can now be any type implementing `Serialize`, not just `serde_json::Value`.

## 0.2.6 - 2025-06-23

Remove extra clones for `details` values in assertions.

Fixes for broken links and compilation issues.

## 0.2.5 - 2025-01-16

Fixing build issues in the no-op version of the SDK (compiling without the `full` feature flag)

## 0.2.4 - 2024-10-31

Implement `rand::RngCore` for `AntithesisRng`

## 0.2.3 - 2024-10-30

Rust doc changes

## 0.2.2 - 2024-10-08

Adding guidance-based assertions. These are both assertions and guidance for the fuzzer to explore your program more effectively.

## 0.2.1 - 2024-07-11

Fixing a broken link

## 0.2.0 - 2024-07-08

Changing feature flags. Now the main flag is `full` (also the default), which turns on the SDK. 

## 0.1.6 - 2024-05-14

Fixing a link in the README

## 0.1.5 - 2024-05-14

More Rust doc changes

## 0.1.4 - 2024-05-14

Rust doc changes

## 0.1.3 - 2024-05-13

Cleanup and README changes


## 0.1.2 - 2024-05-13

Various CI fixes

## 0.1.0 - 2024-05-08

Initial release.
