# Implement Semantic Search Filters

## Backend (Rust)
1.  **Define Data Structures**:
    - In `src-tauri/src/ai/mod.rs`, define `FilterParams` struct to match the JSON output from LLM.
    - Fields: `app_name` (Option<String>), `keywords` (Vec<String>), `date_range` (Option<String>), `has_ocr` (Option<bool>).

2.  **Implement Intent Parser**:
    - In `src-tauri/src/ai/mod.rs`, implement `parse_query_intent(query: &str) -> Result<FilterParams>`.
    - Construct a prompt telling the LLM to extract filters from the user query.
    - Call `chat_with_openai` or `chat_with_anthropic` depending on config.
    - Parse the JSON response.

3.  **Expose Command**:
    - In `src-tauri/src/commands.rs`, add `#[tauri::command] fn parse_query_intent(...)`.
    - This command will call `ai::parse_query_intent`.

## Frontend (React)
1.  **Update Timeline Component**:
    - In `src/components/Timeline.tsx`:
        - Add a "Smart Search" button (using `Sparkles` icon) next to the search input.
        - When clicked (or on Enter if a specific prefix is used), call `parse_query_intent`.
        - Show a loading state while parsing.
        - On success, map the returned `FilterParams` to the component's state (`setAppName`, `setStartDate`, `setEndDate`, `setHasOcr`).
        - Automatically trigger `handleSearch`.

## Compilation
1.  Run `npm run tauri:build` or equivalent to verify compilation.
