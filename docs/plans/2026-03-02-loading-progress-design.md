# Loading Progress Indicator Design

**Date:** 2026-03-02
**Status:** Approved

## Context

On startup, `init_heroku` runs three sequential steps — check CLI installed, verify authentication, fetch apps — all while showing a static "Loading..." message. This gives no feedback about progress or why startup is slow.

## Design

### Data model

Extend `ConnectionStatus::Loading` to carry a `LoadingStep` value:

```rust
pub enum ConnectionStatus {
    Loading(LoadingStep),
    // ...existing variants unchanged
}

pub enum LoadingStep {
    CheckingCli,
    VerifyingAuth,
    FetchingApps,
}
```

`init_heroku` emits intermediate statuses as it progresses:

```
Loading(CheckingCli) → Loading(VerifyingAuth) → Loading(FetchingApps) → Ready / NotAuthenticated / Error
```

### Visual

A `LoadingProgress` component replaces the "Loading..." text in `StatusIndicator`. It renders all three steps inline, each in one of three visual states:

- **Done** — bright text, `✓` prefix
- **Active** — warning colour, subtle CSS pulse animation
- **Pending** — dim/muted text

```
✓ Checking CLI  →  ● Verifying auth  →  ○ Fetching apps
  (done)             (active, pulsing)    (pending, dim)
```

The pulse is a simple CSS `@keyframes` opacity animation on the active step label.

### Files changed

- `src/desktop/components/status_indicator.rs` — add `LoadingStep` enum, update `Loading` variant, add `LoadingProgress` component, update `StatusIndicator` match arm
- `src/desktop/main.rs` — update `init_heroku` to emit `Loading(step)` at each stage, update any `ConnectionStatus::Loading` match arms to handle the new payload
- Theme CSS — add `@keyframes pulse-opacity` and `.loading-step-active` class
