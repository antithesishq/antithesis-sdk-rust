# Changelog

## Unreleased

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
