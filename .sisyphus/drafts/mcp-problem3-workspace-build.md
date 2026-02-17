# Draft: Problem 3 - Workspace Build Failure

## Issue Summary
`cargo build --workspace` fails because Tauri build script references non-existent external binaries.

## Root Cause Analysis

**Error**:
```
resource path `..\target\aarch64-apple-darwin\release\memflow-mcp-x86_64-pc-windows-msvc.exe` doesn't exist
```

**Problem Location**: `src-tauri/tauri.conf.json` lines 32-36
```json
"externalBin": [
  "../target/release/memflow-mcp",
  "../target/x86_64-pc-windows-msvc/release/memflow-mcp",
  "../target/aarch64-apple-darwin/release/memflow-mcp",      // Doesn't exist
  "../target/x86_64-unknown-linux-gnu/release/memflow-mcp"   // Doesn't exist
]
```

**Additional Issue**: The binaries referenced don't match the actual naming pattern. The actual built binary would be `memflow-mcp.exe` (Windows) or `memflow-mcp` (Unix), not with the full target triple suffix.

**Minor Warning** (unrelated):
- `crates/memflow-core/src/ocr_enhance.rs:9` - unused import `ImageBuffer`

## Fix Strategy

1. **Remove non-existent external binary paths** from tauri.conf.json:
   - Remove `../target/aarch64-apple-darwin/release/memflow-mcp` (macOS ARM64 - doesn't exist)
   - Remove `../target/x86_64-unknown-linux-gnu/release/memflow-mcp` (Linux - doesn't exist)

2. **Fix the remaining paths** to match actual build output:
   - Keep `../target/release/memflow-mcp` for cross-platform
   - The Windows-specific path `../target/x86_64-pc-windows-msvc/release/memflow-mcp` may be redundant

3. **Fix the unused import warning** in ocr_enhance.rs

## Acceptance Criteria
- `cargo build --workspace` completes successfully
- `cargo clippy --workspace` has no warnings

## User Decisions
- Plan scope: Separate plans for each problem
- This is Problem 3 (first in sequence)
