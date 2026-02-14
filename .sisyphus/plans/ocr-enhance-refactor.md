# OCR Enhancement Module Refactor

## TL;DR

> **Quick Summary**: Refactor `crates/memflow-core/src/ocr_enhance.rs` to fix critical bugs (P0), integrate into OCR pipeline, and improve test coverage using TDD approach.
>
> **Deliverables**:
> - Fixed `correct_code_symbols()` with context-aware corrections
> - Fixed `fix_bracket_pairs()` with string literal detection
> - Fixed `normalize_whitespace()` preserving indentation
> - Generic `levenshtein_distance()` implementation
> - Implemented `preprocess_terminal_image()` using image crate
> - Integrated enhancement functions into `ocr_worker.rs`
> - Comprehensive test suite with proptest
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 3 waves
> **Critical Path**: Write RED tests → Fix P0 bugs → Integrate → Verify CER improvement

---

## Context

### Original Request

User identified 8 issues in `ocr_enhance.rs` requiring refactor:

1. **P0**: `correct_code_symbols()` has bidirectional conflict in HashMap
2. **P0**: `fix_bracket_pairs()` breaks string literals
3. **P1**: Duplicate Levenshtein implementations (95% identical code)
4. **P2**: `is_likely_code()` too simple, false positives
5. **P2**: `normalize_whitespace()` destroys code formatting
6. **P3**: `preprocess_terminal_image()` is TODO placeholder
7. **Missing**: Comprehensive test coverage
8. **Missing**: Integration with main OCR pipeline

### Interview Summary

**User Decisions**:
- **Test Strategy**: TDD (Tests first) - RED-GREEN-REFACTOR cycle
- **API Changes**: Allowed - only `calculate_cer` used externally (benchmark tool)
- **Integration Scope**: Full integration into `ocr_worker.rs`
- **Image Preprocessing**: Implement now using `image` crate
- **Symbol Correction**: Context-aware (detect code vs plain text context)
- **Whitespace Preservation**: Preserve indentation only, compress internal multiple spaces
- **String Literal Handling**: Full Rust support (escaped quotes, raw strings `r#"..."#`, multiline)
- **CER Target**: 5%+ improvement on test fixtures
- **Integration Gate**: Code-detection gated using `is_likely_code()`

### Research Findings

**From Explore Agent**:
- `ocr_enhance.rs` is isolated - only `calculate_cer()` used in `crates/memflow-core/src/bin/ocr_compare.rs`
- Main OCR pipeline: `ocr_worker.rs` → `ocr/mod.rs::process_image()` → `rapidocr.rs::recognize()`
- No language detection exists in codebase
- Error handling pattern: Main pipeline uses `anyhow::Result`, `ocr_enhance` uses primitive returns
- Test pattern: In-module `#[cfg(test)]` with simple assertions

**From Librarian Agent**:
- **Generic Levenshtein**: Use `generic_levenshtein<T: PartialEq>()` pattern from [strsim-rs](https://github.com/rapidfuzz/strsim-rs)
- **Static HashMap**: Use `std::sync::LazyLock` (Rust 1.80+) or `once_cell::sync::Lazy`
- **OCR Postprocessing**: Multi-stage pipeline (char dict → fuzzy match → substitution)
- **String Literal Detection**: State machine pattern (not regex) - more robust for escapes/nesting
- **Testing**: Use `proptest` for property-based testing + unit tests for edge cases

**From Metis Review**:
- **Risk**: Silent behavior drift in text normalization - lock with RED tests before refactor
- **Risk**: OCR throughput regression from preprocessing - add timing assertions
- **Risk**: Over-correction from symbol mapping - enforce context-aware tests
- **Guardrails**: Do NOT add language detection, tokenizers, ML correction in this task
- **Guardrails**: Do NOT modify `ocr/mod.rs` or `rapidocr.rs` architecture
- **QA Requirement**: All acceptance criteria must be executable commands (zero user intervention)

---

## Work Objectives

### Core Objective

Refactor OCR enhancement module to fix critical bugs, integrate into main OCR pipeline, and establish comprehensive test coverage using TDD methodology.

### Concrete Deliverables

1. Modified `crates/memflow-core/src/ocr_enhance.rs` with all bugs fixed
2. Modified `src-tauri/src/ocr_worker.rs` calling enhancement functions
3. New test fixtures in `tests/fixtures/ocr/` directory
4. Added `proptest` dev-dependency in `crates/memflow-core/Cargo.toml`
5. Integration tests in `src-tauri/tests/ocr_enhancement_integration.rs`

### Definition of Done

```bash
# All tests pass
cargo test --manifest-path crates/memflow-core/Cargo.toml ocr_enhance

# Integration tests pass
cargo test --manifest-path src-tauri/Cargo.toml ocr_enhancement

# Property tests pass
cargo test --manifest-path crates/memflow-core/Cargo.toml prop

# CER improvement verified on fixtures
cargo test --manifest-path crates/memflow-core/Cargo.toml cer_improvement -- --nocapture
# Expected: CER reduced by >=5% on noisy fixture set

# No regression in existing functionality
cargo test --manifest-path crates/memflow-core/Cargo.toml
# Expected: All existing tests still pass
```

### Must Have

- Fix all P0 bugs (bidirectional conflict, string literal breaking)
- Merge duplicate Levenshtein into generic implementation
- Implement `preprocess_terminal_image()` with grayscale/contrast/binarization
- Add `proptest` suite with property-based tests
- Integrate enhancement into `ocr_worker.rs` code path
- Achieve 5%+ CER improvement on test fixtures

### Must NOT Have (Guardrails)

- **DO NOT** add language detection subsystem (tree-sitter, etc.)
- **DO NOT** add ML-based correction or fuzzy dictionary matching
- **DO NOT** modify `ocr/mod.rs` or `rapidocr.rs` public APIs
- **DO NOT** change `calculate_cer()` signature (used by benchmark tool)
- **DO NOT** introduce "cleanup" changes in unrelated files
- **DO NOT** add AI slop patterns (over-engineering, premature abstraction)
- **DO NOT** create acceptance criteria requiring human intervention

---

## Verification Strategy

### Test Decision

- **Infrastructure exists**: YES (`[cfg(test)]` modules, Cargo test integration)
- **Automated tests**: YES (TDD approach)
- **Framework**: Unit tests + proptest for property-based testing
- **Agent-Executed QA**: Every task includes runnable verification commands

### TDD Workflow (RED-GREEN-REFACTOR)

Each task follows this cycle:

**1. RED Phase**:
```bash
# Write failing test first
# Test file: crates/memflow-core/src/ocr_enhance.rs (in #[cfg(test)])
# Test command:
cargo test --manifest-path crates/memflow-core/Cargo.toml {test_name}
# Expected: FAIL (test exists, implementation doesn't match)
```

**2. GREEN Phase**:
```bash
# Implement minimum code to pass test
cargo test --manifest-path crates/memflow-core/Cargo.toml {test_name}
# Expected: PASS
```

**3. REFACTOR Phase**:
```bash
# Clean up while keeping green
cargo test --manifest-path crates/memflow-core/Cargo.toml {test_name}
# Expected: PASS (still)
```

### Agent-Executed QA Scenarios (MANDATORY)

**For Each Task**:

#### Unit Test Verification

```bash
Scenario: Run specific test suite for issue
  Tool: Bash (cargo test)
  Preconditions: Cargo build succeeds
  Steps:
    1. cargo test --manifest-path crates/memflow-core/Cargo.toml {test_function}
    2. Assert exit code is 0
    3. Assert output contains "test {test_function} ... ok"
  Expected Result: Test passes with expected assertions
  Evidence: Terminal output captured

Scenario: Run property tests with proptest
  Tool: Bash (cargo test)
  Preconditions: proptest dependency added
  Steps:
    1. cargo test --manifest-path crates/memflow-core/Cargo.toml prop -- --test-threads=1
    2. Wait for completion (may take 30s+ for property tests)
    3. Assert exit code is 0
    4. Assert output contains "passed N tests"
  Expected Result: All property tests pass
  Evidence: Terminal output captured
```

#### Integration Verification

```bash
Scenario: Verify ocr_worker calls enhancement functions
  Tool: Bash (cargo test + grep)
  Preconditions: Integration test written
  Steps:
    1. cargo test --manifest-path src-tauri/Cargo.toml ocr_integration
    2. Assert test simulates OCR worker flow
    3. grep -r "postprocess_terminal_text\|preprocess_terminal_image" src-tauri/src/ocr_worker.rs
    4. Assert function calls present in ocr_worker.rs
  Expected Result: Enhancement functions integrated into worker
  Evidence: Terminal output + grep result

Scenario: Verify image preprocessing pipeline
  Tool: Bash (cargo test)
  Preconditions: Test fixture images exist
  Steps:
    1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_preprocess_terminal_image
    2. Assert test loads fixture image
    3. Assert test verifies grayscale/contrast/binarization applied
    4. Assert output format matches expected
  Expected Result: Preprocessing produces deterministic output
  Evidence: Test output with image dimensions/metrics
```

#### CER Improvement Verification

```bash
Scenario: Measure CER improvement on fixtures
  Tool: Bash (cargo test)
  Preconditions: Fixture OCR pairs (image + ground truth text) in tests/fixtures/ocr/
  Steps:
    1. cargo test --manifest-path crates/memflow-core/Cargo.toml cer_improvement -- --nocapture
    2. Parse output for "Before CER: X%, After CER: Y%"
    3. Assert (X - Y) / X >= 0.05  # 5% improvement
    4. Assert Y < X  # CER reduced
  Expected Result: CER improved by 5%+ on noisy fixtures
  Evidence: Test output captured

Scenario: Verify no regression on clean fixtures
  Tool: Bash (cargo test)
  Preconditions: Clean OCR fixtures (high quality source images)
  Steps:
    1. cargo test --manifest-path crates/memflow-core/Cargo.toml cer_regression
    2. Assert CER on clean fixtures < 1.0%
    3. Assert CER not increased vs baseline
  Expected Result: No regression on already-good OCR
  Evidence: Test output with CER values
```

#### Performance Envelope Verification

```bash
Scenario: Verify preprocessing doesn't exceed latency budget
  Tool: Bash (cargo test)
  Preconditions: Benchmark test written
  Steps:
    1. cargo test --manifest-path crates/memflow-core/Cargo.toml preprocess_performance -- --nocapture
    2. Parse output for "Preprocessing time: Xms"
    3. Assert X < 100  # 100ms per frame budget
  Expected Result: Preprocessing within latency budget
  Evidence: Timing output in test logs

Scenario: Verify enhancement doesn't block OCR pipeline
  Tool: Bash (cargo test)
  Preconditions: Integration test simulates OCR worker
  Steps:
    1. cargo test --manifest-path src-tauri/Cargo.toml ocr_throughput -- --test-threads=1
    2. Parse output for "processed N items in Xms"
    3. Assert X / N < 50  # 50ms per item average
  Expected Result: Enhancement doesn't significantly slow throughput
  Evidence: Throughput metrics in test output
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── Task 1: Add proptest dependency and test infrastructure
└── Task 2: Create test fixtures directory and baseline CER tests

Wave 2 (After Wave 1):
├── Task 3: Write RED tests for all 8 issues
├── Task 4: Implement context-aware correct_code_symbols
└── Task 5: Implement generic levenshtein_distance

Wave 3 (After Wave 2):
├── Task 6: Implement fix_bracket_pairs with string literal detection
├── Task 7: Implement normalize_whitespace preserving indentation
├── Task 8: Implement preprocess_terminal_image
└── Task 9: Integrate enhancement into ocr_worker.rs

Critical Path: Task 1 → Task 3 → Task 4/5/6/7/8 → Task 9
Parallel Speedup: ~50% faster than sequential
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
|-------|-------------|----------|---------------------|
| 1 | None | 2, 3 | 2 |
| 2 | 1 | 3 | 1 |
| 3 | 1, 2 | 4, 5, 6, 7, 8 | 4, 5 |
| 4 | 3 | 6, 7, 8 | 5, 6, 7 |
| 5 | 3 | 6, 7, 8 | 4, 6, 7 |
| 6 | 3 | 9 | 4, 5, 7, 8 |
| 7 | 3 | 9 | 4, 5, 6, 8 |
| 8 | 3 | 9 | 4, 5, 6, 7 |
| 9 | 4, 5, 6, 7, 8 | None | None (final) |

### Agent Dispatch Summary

| Wave | Tasks | Recommended Agents |
|-------|---------|-------------------|
| 1 | 1, 2 | task(category="quick", load_skills=[], run_in_background=false) |
| 2 | 3, 4, 5 | task(category="unspecified-high", load_skills=[], run_in_background=false) |
| 3 | 6, 7, 8, 9 | task(category="unspecified-high", load_skills=[], run_in_background=false) |

---

## TODOs

### Wave 1: Test Infrastructure

- [ ] 1. Add proptest dependency and test fixtures

  **What to do**:
  - Add `proptest = "1.5"` to `[dev-dependencies]` in `crates/memflow-core/Cargo.toml`
  - Create directory `tests/fixtures/ocr/` in `crates/memflow-core/`
  - Add fixture files: `clean_terminal.txt`, `noisy_terminal.txt`, `code_sample.rs`
  - Add OCR pair fixtures: `clean_terminal.png` + `clean_terminal.txt` (ground truth)
  - Write baseline CER test to measure current state

  **Must NOT do**:
  - Do NOT modify main dependencies (only dev-dependencies)
  - Do NOT add language detection dependencies

  **Recommended Agent Profile**:
  > **Category**: `quick`
  > **Reason**: Simple dependency addition and directory creation, no complex logic

  **Parallelization**:
  - **Can Run In Parallel**: NO (Must start first)
  - **Parallel Group**: Wave 1 (with Task 2)
  - **Blocks**: Tasks 3, 4, 5, 6, 7, 8, 9
  - **Blocked By**: None

  **References**:

  **Pattern References** (existing code to follow):
  - `crates/memflow-core/src/ocr_enhance.rs:273-317` - Existing test structure with `#[cfg(test)]`

  **Test References** (testing patterns to follow):
  - `crates/memflow-core/src/ocr_enhance.rs:277-286` - Test structure using `assert_eq!()`, `assert!()`

  **External References** (libraries and frameworks):
  - [proptest docs](https://docs.rs/proptest/1.5/proptest/) - Property-based testing strategies
  - [strsim-rs tests](https://github.com/rapidfuzz/strsim-rs/blob/main/tests/levenshtein.rs) - Levenshtein test patterns

  **Acceptance Criteria**:

  > **AGENT-EXECUTABLE VERIFICATION ONLY**

  ```bash
  # Verify proptest compiles
  cargo test --manifest-path crates/memflow-core/Cargo.toml --help

  # Verify fixtures exist
  ls tests/fixtures/ocr/
  # Expected: clean_terminal.txt, noisy_terminal.txt, code_sample.rs, clean_terminal.png

  # Verify baseline CER test runs
  cargo test --manifest-path crates/memflow-core/Cargo.toml baseline_cer
  # Expected: Test passes, outputs "Baseline CER: X%"
  ```

  **Agent-Executed QA Scenarios**:

  ```bash
  Scenario: Proptest integration compiles
    Tool: Bash (cargo)
    Preconditions: Cargo.toml modified
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml --help | grep proptest
      2. Assert grep finds proptest in help output
    Expected Result: proptest dependency successfully added
    Evidence: Grep output shows proptest available

  Scenario: Fixture files created
    Tool: Bash (ls)
    Preconditions: tests/fixtures/ocr/ directory created
    Steps:
      1. ls -la tests/fixtures/ocr/
      2. Assert output contains "clean_terminal.txt"
      3. Assert output contains "noisy_terminal.txt"
      4. Assert output contains "code_sample.rs"
      5. Assert output contains "clean_terminal.png"
    Expected Result: All fixture files present
    Evidence: Directory listing output
  ```

  **Commit**: YES
  - Message: `test(ocr_enhance): add proptest dependency and test fixtures`
  - Files: `crates/memflow-core/Cargo.toml`, `tests/fixtures/ocr/*`

---

- [ ] 2. Write RED tests for all 8 issues

  **What to do**:
  - Write failing test `test_correct_code_symbols_context_aware()` for bidirectional conflict
  - Write failing test `test_fix_bracket_pairs_preserves_string_literals()` for string breaking
  - Write failing test `test_normalize_whitespace_preserves_indentation()` for format destruction
  - Write failing test `test_levenshtein_generic_works_for_chars_and_strs()` for duplicate code
  - Write failing test `test_preprocess_terminal_image_grayscale_contrast_binarization()` for TODO
  - Write failing test `test_is_likely_code_reduced_false_positives()` for simple heuristics
  - Write failing test `test_postprocess_terminal_text_integration()` for end-to-end flow
  - Write failing test `test_cer_improvement_on_noisy_fixtures()` for 5%+ improvement

  **Must NOT do**:
  - Do NOT implement any fixes yet (RED phase only)
  - Do NOT modify function signatures (tests must match current API)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Reason**: Multiple related test implementations requiring consistency and coordination
  > **Skills**: None needed - straightforward test writing

  **Parallelization**:
  - **Can Run In Parallel**: YES (Tests are independent)
  - **Parallel Group**: Wave 2 (with Tasks 4, 5)
  - **Blocks**: Tasks 6, 7, 8, 9
  - **Blocked By**: Task 1, 2

  **References**:

  **Pattern References** (existing code to follow):
  - `crates/memflow-core/src/ocr_enhance.rs:277-294` - Test naming and structure
  - `crates/memflow-core/src/ocr_enhance.rs:277-286` - Bracket fix test pattern

  **Test References** (testing patterns to follow):
  - `crates/memflow-core/src/ocr_enhance.rs:305-309` - Code detection test pattern
  - `crates/memflow-core/src/ocr_enhance.rs:288-294` - CER/WER test pattern

  **Acceptance Criteria**:

  ```bash
  # All new tests fail (RED)
  cargo test --manifest-path crates/memflow-core/Cargo.toml ocr_enhance::tests
  # Expected: 8 new test failures, format: "test {name} ... FAILED"

  # Verify test count increased
  cargo test --manifest-path crates/memflow-core/Cargo.toml ocr_enhance::tests -- --list
  # Expected: 13 tests (5 existing + 8 new)
  ```

  **Agent-Executed QA Scenarios**:

  ```bash
  Scenario: All RED tests fail as expected
    Tool: Bash (cargo test)
    Preconditions: Test fixtures exist from Task 1
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml ocr_enhance::tests 2>&1
      2. grep -c "FAILED" output
      3. Assert grep count >= 8  # At least 8 failures
      4. grep -c "test result: FAILED" output
    Expected Result: RED phase confirmed - tests fail before implementation
    Evidence: Test output showing 8+ failures

  Scenario: Test names follow naming convention
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml ocr_enhance::tests -- --list
      2. Assert output contains "test_correct_code_symbols_context_aware"
      3. Assert output contains "test_fix_bracket_pairs_preserves_string_literals"
      4. Assert output contains "test_normalize_whitespace_preserves_indentation"
    Expected Result: All 8 new test names present
    Evidence: Test listing output
  ```

  **Commit**: YES
  - Message: `test(ocr_enhance): write RED tests for 8 issues`
  - Files: `crates/memflow-core/src/ocr_enhance.rs`

---

### Wave 2: Fix P0 Bugs

- [ ] 3. Implement context-aware correct_code_symbols

  **What to do**:
  - Remove bidirectional conflict in HashMap (lines 64-82)
  - Implement context detection using code keyword scanning
  - Define code keywords: `fn`, `def`, `class`, `let`, `var`, `const`, `=`, `;`
  - When in code context: apply `l→1`, `O→0`, `I→l` substitutions
  - When NOT in code context: preserve original characters
  - Use `std::sync::LazyLock` for static corrections map (Rust 1.80+)

  **Must NOT do**:
  - Do NOT add fuzzy dictionary matching (out of scope)
  - Do NOT add ML-based correction (out of scope)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Reason**: Requires implementing new logic with context awareness and static HashMap optimization
  > **Skills**: None needed - core Rust functionality

  **Parallelization**:
  - **Can Run In Parallel**: YES (Independent of other fixes)
  - **Parallel Group**: Wave 2 (with Tasks 4, 5)
  - **Blocks**: Task 9
  - **Blocked By**: Task 3

  **References**:

  **Pattern References** (existing code to follow):
  - `src-tauri/src/ocr/mod.rs:91-162` - Regex-based redaction pattern for PII
  - `crates/memflow-core/src/ocr_enhance.rs:62-82` - Current symbol correction structure

  **API/Type References** (contracts to implement against):
  - `crates/memflow-core/src/ocr_enhance.rs:62` - `fn correct_code_symbols(text: &str) -> String`

  **Documentation References** (specs and requirements):
  - [strsim-rs generic implementation](https://github.com/rapidfuzz/strsim-rs/blob/main/src/lib.rs) - Generic pattern reference
  - [std::sync::LazyLock docs](https://doc.rust.org/std/sync/struct.LazyLock.html) - Static HashMap pattern

  **External References** (libraries and frameworks):
  - [once_cell vs LazyLock](https://docs.rs/once_cell/latest/once_cell/) - Static initialization patterns

  **Acceptance Criteria**:

  ```bash
  # Context-aware test passes (GREEN)
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_correct_code_symbols_context_aware
  # Expected: "ok" result, test passes

  # No bidirectional conflict
  cargo test --manifest-path crates/memflow-core/Cargo.toml correct_code_symbols_no_conflict
  # Expected: No flip-flop behavior (l→1→l)

  # Context detection works
  cargo test --manifest-path crates/memflow-core/Cargo.toml context_detection_code_vs_text
  # Expected: Code keywords trigger corrections, plain text preserved
  ```

  **Agent-Executed QA Scenarios**:

  ```bash
  Scenario: Code context triggers l→1 correction
    Tool: Bash (cargo test)
    Preconditions: Implementation complete
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_code_context_correction -- --nocapture
      2. Assert output contains "let x = 1"  # l corrected to 1 in code
      3. Assert output contains "hello world"  # l preserved in plain text
    Expected Result: Context-aware correction applied correctly
    Evidence: Test output showing corrections

  Scenario: Plain text preserves original characters
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_plain_text_preserved -- --nocapture
      2. Assert output contains "Label: hello"  # l NOT changed to 1
      3. Assert output contains "Number: 0"  # 0 NOT changed to O
    Expected Result: Non-code text unchanged
    Evidence: Test output with preserved text

  Scenario: LazyLock static HashMap initialized once
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml static_corrections_map -- --nocapture
      2. Assert output contains "CORRECTIONS initialized once"
    Expected Result: Static map uses LazyLock correctly
    Evidence: Test log output
  ```

  **Commit**: YES
  - Message: `fix(ocr_enhance): implement context-aware symbol correction`
  - Files: `crates/memflow-core/src/ocr_enhance.rs`

---

- [ ] 4. Merge duplicate Levenshtein into generic implementation

  **What to do**:
  - Create `fn levenshtein_distance<T, U>(a: &[T], b: &[U]) -> usize` where `T: PartialEq<U>`
  - Remove `levenshtein_distance_chars()` (lines 182-212)
  - Remove `levenshtein_distance_str()` (lines 215-245)
  - Update `calculate_cer()` to use generic version
  - Update `calculate_wer()` to use generic version
  - Add property tests: reflexivity (dist(a,a) == 0), symmetry (dist(a,b) == dist(b,a)), triangle inequality

  **Must NOT do**:
  - Do NOT change `calculate_cer()` public signature (line 152-163)
  - Do NOT change `calculate_wer()` public signature (line 168-179)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Reason**: Generic implementation requires trait bounds and type system work
  > **Skills**: None needed - core Rust generics

  **Parallelization**:
  - **Can Run In Parallel**: YES (Independent of other fixes)
  - **Parallel Group**: Wave 2 (with Tasks 3, 5)
  - **Blocks**: Task 9
  - **Blocked By**: Task 3

  **References**:

  **Pattern References** (existing code to follow):
  - `crates/memflow-core/src/ocr_enhance.rs:182-212` - Current char-based Levenshtein
  - `crates/memflow-core/src/ocr_enhance.rs:215-245` - Current str-based Levenshtein

  **API/Type References** (contracts to implement against):
  - `crates/memflow-core/src/ocr_enhance.rs:152` - `pub fn calculate_cer(reference: &str, hypothesis: &str) -> f64`
  - `crates/memflow-core/src/ocr_enhance.rs:168` - `pub fn calculate_wer(reference: &str, hypothesis: &str) -> f64`

  **External References** (libraries and frameworks):
  - [strsim-rs generic_levenshtein](https://github.com/rapidfuzz/strsim-rs/blob/b21046a/src/lib.rs#L267-L296) - Generic implementation pattern
  - [jellyfish vec_levenshtein](https://github.com/jamesturk/jellyfish/blob/f3e425c/src/levenshtein.rs#L10-L30) - Alternative generic pattern

  **Acceptance Criteria**:

  ```bash
  # Generic implementation works for chars
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_levenshtein_chars
  # Expected: Pass, uses generic fn with &[char]

  # Generic implementation works for strs
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_levenshtein_strs
  # Expected: Pass, uses generic fn with &[&str]

  # Property tests pass
  cargo test --manifest-path crates/memflow-core/Cargo.toml prop_levenshtein_properties
  # Expected: Reflexivity, symmetry, triangle inequality all pass

  # CER/WER still work
  cargo test --manifest-path crates/memflow-core/Cargo.toml calculate_cer calculate_wer
  # Expected: All existing CER/WER tests pass
  ```

  **Agent-Executed QA Scenarios**:

  ```bash
  Scenario: Generic Levenshtein works for char slices
    Tool: Bash (cargo test)
    Preconditions: Generic implementation complete
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_levenshtein_generic_chars -- --nocapture
      2. Assert test creates &[char] inputs
      3. Assert output contains "distance: 3"
    Expected Result: Generic function handles char slices correctly
    Evidence: Test output with distance calculation

  Scenario: Generic Levenshtein works for word slices
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_levenshtein_generic_words -- --nocapture
      2. Assert test creates &[&str] inputs
      3. Assert output contains "distance: 2"
    Expected Result: Generic function handles word slices correctly
    Evidence: Test output with distance calculation

  Scenario: Property tests verify mathematical properties
    Tool: Bash (cargo test)
    Preconditions: proptest dependency added
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml prop_levenshtein -- --test-threads=1
      2. Wait for property tests to complete (30s timeout)
      3. Assert output contains "passed 256 tests" or similar
      4. Assert exit code is 0
    Expected Result: All property tests pass with many random cases
    Evidence: Proptest output showing test count and passes
  ```

  **Commit**: YES
  - Message: `refactor(ocr_enhance): merge duplicate Levenshtein into generic implementation`
  - Files: `crates/memflow-core/src/ocr_enhance.rs`

---

### Wave 3: Fix Remaining Bugs & Implement Features

- [ ] 5. Implement fix_bracket_pairs with string literal detection

  **What to do**:
  - Implement state machine to track string literals (lines 84-128)
  - Track: `in_string: Option<char>` (quote type), `escaped: bool`, `raw_string_level: u8`
  - Handle escaped quotes: `\"` inside strings
  - Handle raw strings: `r#"..."#`, `r##"..."##`
  - Handle multiline strings: `"` at start/end of lines
  - Skip bracket processing when inside string literal
  - Only auto-close brackets outside strings

  **Must NOT do**:
  - Do NOT use regex for state machine (use explicit char-by-char parsing)
  - Do NOT add full tokenizer (over-engineering)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Reason**: State machine implementation with complex edge cases (escapes, raw strings)
  > **Skills**: None needed - core Rust string parsing

  **Parallelization**:
  - **Can Run In Parallel**: YES (Independent of other Wave 3 tasks)
  - **Parallel Group**: Wave 3 (with Tasks 6, 7, 8, 9)
  - **Blocks**: Task 9
  - **Blocked By**: Task 3

  **References**:

  **Pattern References** (existing code to follow):
  - `crates/memflow-core/src/ocr_enhance.rs:84-128` - Current bracket fix structure
  - `src-tauri/src/ocr/mod.rs:92-106` - Regex pattern for pattern matching

  **External References** (libraries and frameworks):
  - [a2ltool scanner.rs](https://github.com/DanielT/a2ltool/blob/f3e425c/src/creator/scanner.rs#L203-L227) - State machine for string literals

  **Acceptance Criteria**:

  ```bash
  # String literals preserved
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_string_literals_preserved
  # Expected: print("hello) stays as print("hello), not modified

  # Raw strings handled
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_raw_string_literals
  # Expected: r#"(parenthesis)"# stays unchanged

  # Escaped quotes handled
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_escaped_quotes_in_strings
  # Expected: "say \"hello\"" stays as is, brackets inside skipped

  # Code brackets still fixed
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_code_brackets_fixed
  # Expected: (hello world → (hello world) with closing added
  ```

  **Agent-Executed QA Scenarios**:

  ```bash
  Scenario: String literal brackets are not modified
    Tool: Bash (cargo test)
    Preconditions: State machine implementation complete
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_string_literal_brackets -- --nocapture
      2. Assert output contains 'print("hello")'
      3. Assert output does NOT contain 'print("hello)"'  # No auto-close inside string
    Expected Result: String literals left intact
    Evidence: Test output showing preserved strings

  Scenario: Raw string literals handled correctly
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_raw_string_r_pound -- --nocapture
      2. Assert output contains r#"x = (value)"#
      3. Assert parentheses NOT auto-closed
    Expected Result: Raw strings preserved with content intact
    Evidence: Test output showing raw string handling

  Scenario: Escaped quotes don't end string literal
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_escaped_quote_state -- --nocapture
      2. Assert output contains "say \\\"hello\\\""  # Escaped, not string end
      3. Assert brackets after escaped quote NOT processed as in-string
    Expected Result: State machine correctly tracks escaped quotes
    Evidence: Test output with state transitions
  ```

  **Commit**: YES
  - Message: `fix(ocr_enhance): implement string literal detection in fix_bracket_pairs`
  - Files: `crates/memflow-core/src/ocr_enhance.rs`

---

- [ ] 6. Implement normalize_whitespace preserving indentation

  **What to do**:
  - Modify `normalize_whitespace()` (lines 131-147)
  - Preserve leading whitespace (indentation) exactly as-is
  - Compress consecutive spaces to single space internally (after leading whitespace)
  - Preserve line breaks
  - Do NOT use `split_whitespace()` (loses indentation)
  - Use char-by-char parsing: track leading_space_count, compress runs after

  **Must NOT do**:
  - Do NOT preserve all intra-line spacing (user wants indent-only preservation)
  - Do NOT change trailing/leading whitespace per line (indentation is leading)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Reason**: Requires careful whitespace handling without breaking code structure
  > **Skills**: None needed - core Rust string manipulation

  **Parallelization**:
  - **Can Run In Parallel**: YES (Independent of other Wave 3 tasks)
  - **Parallel Group**: Wave 3 (with Tasks 5, 7, 8, 9)
  - **Blocks**: Task 9
  - **Blocked By**: Task 3

  **References**:

  **Pattern References** (existing code to follow):
  - `crates/memflow-core/src/ocr_enhance.rs:131-147` - Current whitespace normalization
  - `crates/memflow-core/src/ocr_enhance.rs:311-316` - Existing whitespace test

  **Acceptance Criteria**:

  ```bash
  # Indentation preserved
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_indentation_preserved
  # Expected: "    hello world" keeps 4 leading spaces

  # Multiple spaces compressed internally
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_internal_spaces_compressed
  # Expected: "hello    world" → "hello world"

  # Code alignment not destroyed
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_code_alignment_preserved
  # Expected: Indentation kept, internal spaces compressed per contract
  ```

  **Agent-Executed QA Scenarios**:

  ```bash
  Scenario: Leading indentation is preserved
    Tool: Bash (cargo test)
    Preconditions: Whitespace normalization implementation complete
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_leading_indent_preserved -- --nocapture
      2. Assert output contains "    hello"  # 4 spaces preserved
      3. Assert output does NOT contain "hello"  # Not trimmed
    Expected Result: Leading indentation kept exactly
    Evidence: Test output showing indentation

  Scenario: Internal multiple spaces compressed
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_internal_spaces_compress -- --nocapture
      2. Assert output contains "hello world"  # 4 spaces → 1 space
      3. Assert output does NOT contain "hello    world"
    Expected Result: Multiple consecutive spaces collapsed to single
    Evidence: Test output with compressed text

  Scenario: Mixed indentation and internal spaces handled
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_mixed_whitespace -- --nocapture
      2. Assert output line 1 is "    hello world"  # indent preserved
      3. Assert output line 2 is "    foo bar"     # indent preserved, internal compressed
    Expected Result: Both indentation and compression work correctly
    Evidence: Multi-line test output
  ```

  **Commit**: YES
  - Message: `fix(ocr_enhance): preserve indentation in normalize_whitespace`
  - Files: `crates/memflow-core/src/ocr_enhance.rs`

---

- [ ] 7. Implement preprocess_terminal_image

  **What to do**:
  - Remove TODO placeholder (lines 26-38)
  - Use `image` crate (already in dependencies)
  - Implement: grayscale conversion (Luma)
  - Implement: contrast enhancement (stretch histogram)
  - Implement: binarization (threshold for text)
  - Take `&[u8]` (PNG bytes), return `Vec<u8>` (processed PNG bytes)
  - Add benchmark test for latency (< 100ms per frame)

  **Must NOT do**:
  - Do NOT change function signature from `pub fn preprocess_terminal_image(image_data: &[u8]) -> Vec<u8>`
  - Do NOT add image resizing (handled by ocr_worker.rs already)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Reason**: Image processing requires using image crate APIs correctly
  > **Skills**: None needed - image crate usage

  **Parallelization**:
  - **Can Run In Parallel**: YES (Independent of other Wave 3 tasks)
  - **Parallel Group**: Wave 3 (with Tasks 5, 6, 8, 9)
  - **Blocks**: Task 9
  - **Blocked By**: Task 3

  **References**:

  **Pattern References** (existing code to follow):
  - `src-tauri/src/ocr_worker.rs:124-176` - Image preprocessing pattern (resize)
  - `crates/memflow-core/src/ocr_enhance.rs:26-38` - Current placeholder signature

  **API/Type References** (contracts to implement against):
  - `crates/memflow-core/src/ocr_enhance.rs:33` - `pub fn preprocess_terminal_image(image_data: &[u8]) -> Vec<u8>`

  **External References** (libraries and frameworks):
  - [image crate docs](https://docs.rs/image/latest/image/) - Image manipulation APIs
  - [imageops::filter](https://docs.rs/image/latest/image/imageops/struct.FilterType.html) - Resampling filters

  **Acceptance Criteria**:

  ```bash
  # Preprocessing produces grayscale output
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_preprocess_grayscale
  # Expected: Image converted to Luma (single channel)

  # Contrast enhancement applied
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_preprocess_contrast
  # Expected: Histogram stretched for better OCR

  # Binarization applied for text
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_preprocess_binarization
  # Expected: Threshold applied, pixels 0 or 255

  # Performance within budget
  cargo test --manifest-path crates/memflow-core/Cargo.toml test_preprocess_performance -- --nocapture
  # Expected: Processing time < 100ms for 1920x1080
  ```

  **Agent-Executed QA Scenarios**:

  ```bash
  Scenario: Image is converted to grayscale
    Tool: Bash (cargo test)
    Preconditions: Fixture image exists (tests/fixtures/ocr/clean_terminal.png)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_grayscale_conversion -- --nocapture
      2. Assert output contains "channels: 1" or "Luma"
      3. Assert output does NOT contain "RGB" or "3 channels"
    Expected Result: Image is grayscale (single channel)
    Evidence: Test output showing image format

  Scenario: Contrast is enhanced
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_contrast_enhancement -- --nocapture
      2. Assert output contains "min: 0, max: 255" or similar histogram stretch
    Expected Result: Contrast stretched to full range
    Evidence: Test output with pixel value range

  Scenario: Binarization creates black/white image
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_binarization_threshold -- --nocapture
      2. Assert output contains "threshold: 128" or similar
      3. Assert output shows "unique values: 2" (black and white only)
    Expected Result: Binary image suitable for OCR
    Evidence: Test output with threshold value

  Scenario: Preprocessing latency within budget
    Tool: Bash (cargo test)
    Preconditions: Benchmark test written
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml test_preprocess_latency -- --nocapture
      2. Parse output for "Preprocessing time: Xms"
      3. Assert X < 100  # 100ms budget
    Expected Result: Preprocessing completes within performance budget
    Evidence: Timing output in test logs
  ```

  **Commit**: YES
  - Message: `feat(ocr_enhance): implement preprocess_terminal_image with grayscale/contrast/binarization`
  - Files: `crates/memflow-core/src/ocr_enhance.rs`

---

- [ ] 8. Integrate enhancement functions into ocr_worker.rs

  **What to do**:
  - Add `use memflow_core::ocr_enhance::{preprocess_terminal_image, postprocess_terminal_text, is_likely_code};` to `ocr_worker.rs`
  - Call `preprocess_terminal_image()` before OCR (integrate with existing resize logic)
  - Call `postprocess_terminal_text()` after OCR result
  - Gate enhancement with `is_likely_code()` check (user requested code-detection gating)
  - Add logging: "OCR enhancement applied: {functions}"
  - Update integration test to verify end-to-end flow

  **Must NOT do**:
  - Do NOT modify `ocr/mod.rs` or `rapidocr.rs` (guardrail)
  - Do NOT change `process_image()` signature (guardrail)
  - Do NOT add new config options (use existing code detection)

  **Recommended Agent Profile**:
  > **Category**: `unspecified-high`
  > **Reason**: Integration requires understanding existing worker flow and safe insertion points
  > **Skills**: None needed - module integration

  **Parallelization**:
  - **Can Run In Parallel**: NO (Final integration task)
  - **Parallel Group**: Sequential (after all others)
  - **Blocks**: None (final task)
  - **Blocked By**: Tasks 4, 5, 6, 7, 8

  **References**:

  **Pattern References** (existing code to follow):
  - `src-tauri/src/ocr_worker.rs:112-176` - Image preprocessing location
  - `src-tauri/src/ocr_worker.rs:201-220` - OCR result handling location
  - `src-tauri/src/ocr/mod.rs:80-88` - Current OCR processing flow

  **Test References** (testing patterns to follow):
  - `crates/memflow-core/src/ocr_enhance.rs:247-254` - `evaluate_ocr_quality()` function for metrics

  **Documentation References** (specs and requirements):
  - `PROJECT_ARCHITECTURE.md` - OCR pipeline architecture (if exists)

  **Acceptance Criteria**:

  ```bash
  # Integration test passes
  cargo test --manifest-path src-tauri/Cargo.toml ocr_enhancement_integration
  # Expected: Full OCR worker flow with enhancements applied

  # Preprocess called before OCR
  cargo test --manifest-path src-tauri/Cargo.toml test_preprocess_called
  # Expected: Log contains "Preprocessing image with preprocess_terminal_image"

  # Postprocess called after OCR
  cargo test --manifest-path src-tauri/Cargo.toml test_postprocess_called
  # Expected: Log contains "Postprocessing with postprocess_terminal_text"

  # Code detection gates enhancement
  cargo test --manifest-path src-tauri/Cargo.toml test_code_detection_gate
  # Expected: Enhancement only applied when is_likely_code() returns true
  ```

  **Agent-Executed QA Scenarios**:

  ```bash
  Scenario: Enhancement functions called in OCR worker
    Tool: Bash (cargo test + grep)
    Preconditions: Integration test written
    Steps:
      1. cargo test --manifest-path src-tauri/Cargo.toml test_enhancement_integration -- --nocapture
      2. Assert output contains "preprocess_terminal_image called"
      3. Assert output contains "postprocess_terminal_text called"
      4. grep -n "use memflow_core::ocr_enhance" src-tauri/src/ocr_worker.rs
      5. Assert grep finds import line
    Expected Result: Enhancement module imported and functions called
    Evidence: Test output + grep result

  Scenario: Enhancement only applied to code-like content
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path src-tauri/Cargo.toml test_code_gating -- --nocapture
      2. Assert output contains "is_likely_code: true → enhancement applied"
      3. Assert output contains "is_likely_code: false → enhancement skipped"
    Expected Result: Code detection gate working
    Evidence: Test output with gating decisions

  Scenario: Integration doesn't break existing OCR flow
    Tool: Bash (cargo test)
    Preconditions: Existing OCR tests pass
    Steps:
      1. cargo test --manifest-path src-tauri/Cargo.toml ocr_worker -- --test-threads=1
      2. Assert all existing worker tests still pass
      3. Assert no new errors in output
    Expected Result: Integration is non-breaking
    Evidence: Full test suite output
  ```

  **Commit**: YES
  - Message: `integrate(ocr_worker): add enhancement preprocessing and postprocessing`
  - Files: `src-tauri/src/ocr_worker.rs`, `src-tauri/tests/ocr_enhancement_integration.rs`

---

### Wave 3: Verification

- [ ] 9. Verify CER improvement and finalize

  **What to do**:
  - Run CER improvement test on noisy fixtures
  - Verify 5%+ improvement achieved
  - Run full test suite (unit + integration + property tests)
  - Check for regressions on clean fixtures
  - Document results in `PROGRESS.md` or refactor notes
  - Verify performance envelope (preprocessing latency, OCR throughput)

  **Must NOT do**:
  - Do NOT modify implementation at this stage (verification only)
  - Do NOT add new features beyond scope

  **Recommended Agent Profile**:
  > **Category**: `quick`
  > **Reason**: Verification and documentation task
  > **Skills**: None needed - running tests and documenting

  **Parallelization**:
  - **Can Run In Parallel**: NO (Final verification)
  - **Parallel Group**: Sequential (final task)
  - **Blocks**: None (done)
  - **Blocked By**: All previous tasks

  **Acceptance Criteria**:

  ```bash
  # CER improvement verified
  cargo test --manifest-path crates/memflow-core/Cargo.toml cer_improvement -- --nocapture
  # Expected: "Before: X%, After: Y%, Improvement: Z%" where Z >= 5.0

  # All tests pass
  cargo test --manifest-path crates/memflow-core/Cargo.toml
  # Expected: All tests pass, no failures

  # No regressions
  cargo test --manifest-path src-tauri/Cargo.toml ocr_worker
  # Expected: All existing worker tests pass

  # Performance budget maintained
  cargo test --manifest-path crates/memflow-core/Cargo.toml preprocess_performance
  # Expected: Preprocessing < 100ms, throughput not degraded
  ```

  **Agent-Executed QA Scenarios**:

  ```bash
  Scenario: CER improvement >= 5% on noisy fixtures
    Tool: Bash (cargo test)
    Preconditions: Noisy OCR fixtures exist
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml cer_improvement -- --nocapture
      2. Parse output: grep "CER before:" | awk '{print $3}'
      3. Parse output: grep "CER after:" | awk '{print $3}'
      4. Assert ((before - after) / before * 100 >= 5.0)
    Expected Result: CER reduced by at least 5 percentage points
    Evidence: Test output with before/after CER values

  Scenario: No regression on clean fixtures
    Tool: Bash (cargo test)
    Preconditions: Clean OCR fixtures exist
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml cer_regression -- --nocapture
      2. Assert output contains "CER < 1.0%"
      3. Assert output does NOT contain "CER increased"
    Expected Result: High-quality OCR not degraded
    Evidence: Test output with low CER values

  Scenario: Full test suite passes
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml --test-threads=1
      2. Assert exit code is 0
      3. Assert output contains "test result: ok"
    Expected Result: All unit and property tests pass
    Evidence: Final test suite output

  Scenario: Performance within budget
    Tool: Bash (cargo test)
    Steps:
      1. cargo test --manifest-path crates/memflow-core/Cargo.toml performance_envelope -- --nocapture
      2. Parse "Preprocessing: Xms" → Assert X < 100
      3. Parse "Throughput: Yms/item" → Assert Y < 50
    Expected Result: No performance regression from enhancements
    Evidence: Timing metrics in test output
  ```

  **Commit**: NO (Verification task only)

---

## Commit Strategy

| After Task | Message | Files | Verification |
|--------------|-------------|--------|---------------|
| 1 | `test(ocr_enhance): add proptest dependency and test fixtures` | Cargo.toml, tests/fixtures/ocr/* | cargo test --help |
| 2 | `test(ocr_enhance): write RED tests for 8 issues` | ocr_enhance.rs | cargo test ocr_enhance (expects 8 failures) |
| 3 | `fix(ocr_enhance): implement context-aware symbol correction` | ocr_enhance.rs | cargo test test_correct_code_symbols_context_aware |
| 4 | `refactor(ocr_enhance): merge duplicate Levenshtein into generic implementation` | ocr_enhance.rs | cargo test test_levenshtein_generic_* |
| 5 | `fix(ocr_enhance): implement string literal detection in fix_bracket_pairs` | ocr_enhance.rs | cargo test test_string_literals_preserved |
| 6 | `fix(ocr_enhance): preserve indentation in normalize_whitespace` | ocr_enhance.rs | cargo test test_indentation_preserved |
| 7 | `feat(ocr_enhance): implement preprocess_terminal_image with grayscale/contrast/binarization` | ocr_enhance.rs | cargo test test_preprocess_grayscale |
| 8 | `integrate(ocr_worker): add enhancement preprocessing and postprocessing` | ocr_worker.rs, tests/ocr_enhancement_integration.rs | cargo test ocr_enhancement_integration |
| 9 | (no commit - verification only) | - | cargo test (all pass) |

---

## Success Criteria

### Verification Commands

```bash
# All unit tests pass
cargo test --manifest-path crates/memflow-core/Cargo.toml ocr_enhance

# Integration tests pass
cargo test --manifest-path src-tauri/Cargo.toml ocr_enhancement

# Property tests pass
cargo test --manifest-path crates/memflow-core/Cargo.toml prop

# CER improvement verified
cargo test --manifest-path crates/memflow-core/Cargo.toml cer_improvement -- --nocapture
# Expected: "Improvement: 5.2%" or >= 5.0%

# Performance envelope maintained
cargo test --manifest-path crates/memflow-core/Cargo.toml performance -- --nocapture
# Expected: Preprocessing < 100ms, throughput not degraded
```

### Final Checklist

- [ ] All P0 bugs fixed (symbol conflict, string literal breaking)
- [ ] P1 duplicate code removed (generic Levenshtein)
- [ ] P2 improvements made (is_likely_code, normalize_whitespace)
- [ ] P3 feature implemented (preprocess_terminal_image)
- [ ] Integration complete (ocr_worker.rs calls enhancement)
- [ ] Test coverage comprehensive (TDD RED-GREEN-REFACTOR cycle followed)
- [ ] Property tests added (proptest suite passes)
- [ ] CER improvement >= 5% on noisy fixtures
- [ ] No regression on clean fixtures
- [ ] Performance within budget (preprocessing < 100ms)
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
