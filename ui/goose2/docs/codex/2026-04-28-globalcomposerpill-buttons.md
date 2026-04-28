# Codex task: wire the GlobalComposerPill toolbar buttons

**Status:** ready to implement
**Owner of the design intent:** Tulsi (Design Fellow)
**Repo:** `aaif-goose`, working directory `ui/goose2/`
**Branch:** `tulsi/visual-design`

## Mission

The `GlobalComposerPill` is the floating "Start a conversation" pill at bottom-right that appears on every non-chat view (home, agents, skills, sessions, projects). It currently expands on hover/focus to reveal a toolbar row (Plus, model picker, project picker, mic, send). The expanded toolbar's buttons except Send are inert visual placeholders. **Wire them up.**

The collapsed state (input + inline mic + send) already works and is intentional — leave it functional and untouched.

## Visual reference

Figma frame for the expanded state: <https://www.figma.com/design/e43a6gyBVn1SdARFkZpN0N/Goose-Brand-%E2%80%94-Q2-2026?node-id=114-1358>

The existing implementation (file you'll edit) is `src/shared/ui/GlobalComposerPill.tsx`. It already has the right visual structure (collapsed top row + max-height-animated toolbar row); your job is functional wiring.

## What each button must do

### 1. Plus button — file attachment
- On click, open a Tauri native file dialog (use `import { open } from "@tauri-apps/plugin-dialog"`, same pattern as `src/features/chat/ui/ChatInput.tsx:312`).
- Accept multiple files. No directories needed.
- Show selected file paths as small chips ABOVE the input row inside the pill (one chip per file, with filename only — strip the directory). Chips have a small × to remove individual files.
- On send, the new chat session must be created with these files pre-loaded as attachments.
- Keep this lightweight — do NOT replicate ChatInput's full attachment system (drag-drop, paste, glob expansion). Just file picker → chips → forward to new session.

### 2. Model picker — provider + model selection
- Click opens a Popover (use `@/shared/ui/popover`, same pattern as ChatInput's AgentModelPicker).
- Populated from:
  - `useProviderInventoryStore` for the list of providers
  - `useAgentStore` for the currently-selected provider (read `selectedProvider`)
- Display options grouped by provider, model name as the leaf.
- On select, store locally in component state (do NOT mutate `agentStore.selectedProvider` — global pill is a per-compose override, not a global setting).
- The button label updates to show the selected model. Default label "Claude Opus 4.7" should be replaced with whatever the actual current/selected model is.
- On send, pass the selected provider + model to AppShell so the new session uses them.

### 3. Project picker — project selection
- Click opens a Popover with the list of projects from `useProjectStore`.
- "No project" is a valid option (default).
- On select, store locally; button label updates.
- On send, pass the selected project ID to AppShell.

### 4. Mic — voice dictation
- Use the existing `useVoiceDictation` hook from `src/features/chat/hooks/useVoiceDictation.ts`. Look at how `ChatInput.tsx` integrates it (around line 108).
- Dictate into the pill's input. While recording, replace the placeholder with the existing recording/transcribing translations (`t("toolbar.voiceInputRecording")`, etc.) — but the pill is in `src/shared/ui/`, not chat namespace. You may need to introduce new shared keys OR pass simple string literals; mirror whatever the rest of `shared/ui/` does for placeholder localization.
- Mic button visually toggles state (active when recording).
- Voice + send: clicking send while recording should follow ChatInput's pattern (`stopRecording({ flushPending: false })` then send what's already transcribed).

## AppShell side: extend `handleGlobalCompose`

Current signature (in `src/app/AppShell.tsx`):

```tsx
const handleGlobalCompose = useCallback(
  async (text: string) => {
    const session = await createNewTab(DEFAULT_CHAT_TITLE);
    chatStore.setPendingFirstMessage(session.id, text);
  },
  [createNewTab, chatStore],
);
```

Extend to accept an options bag:

```tsx
interface GlobalComposeOptions {
  providerId?: string;
  modelId?: string;
  modelName?: string;
  projectId?: string | null;
  attachments?: ChatAttachmentDraft[]; // import from @/shared/types/messages
}
```

`createNewTab` already accepts a `ProjectInfo` for project context — extend or wrap so the explicit options take precedence over the defaults from `agentStore.selectedProvider`. For attachments, look at how `ChatInput` queues them on send and mirror that shape into the pendingFirstMessage flow (you may need to add `pendingFirstAttachments` to chatStore similar to `pendingFirstMessage`).

The `onSend` prop on `GlobalComposerPill` should accept a second optional parameter:

```tsx
onSend: (text: string, options?: GlobalComposeOptions) => void;
```

Update the prop type and AppShell's invocation accordingly.

## Hard constraints (durable, apply to every task on this branch)

These come from `~/.claude/projects/-Users-tulsi/memory/goose2-alex-redesign-state.md`:

1. **Production code, not hack-week.** Asset hygiene matters; doc accuracy matters; surfaces are surface-level (tokens, classNames). Don't introduce architectural changes.
2. **Functionality preserved.** Stores/hooks/APIs untouched UNLESS scope is explicitly expanded. Adding `pendingFirstAttachments` to chatStore IS an explicit scope expansion for this task — it's needed and approved.
3. **Commit only with explicit approval.** Do NOT create a commit. Tulsi will review the staged diff and say "commit" herself.
4. **Work only in `ui/goose2/`.** Do NOT touch `ui/desktop/` or any other workspace package.
5. **Stash@{0} off-limits.** Don't pop/drop/apply.
6. **No `--no-verify`, no `--amend`, no force pushes.**
7. **File-size caps.** `scripts/check-file-sizes.mjs` runs in pre-commit. Per-file caps are defined there. If wiring all four buttons pushes `GlobalComposerPill.tsx` past its current cap, bump the cap narrowly with a 1-line comment justifying the bump rather than splitting into a sub-component just to dodge the cap. ChatInput itself has a 510-line cap as precedent.

## Acceptance criteria

A complete implementation must satisfy all of these:

1. **All four expanded-toolbar buttons are functional**:
   - Plus opens file dialog, selected files appear as chips, chips have × to remove, attachments survive into the new chat session
   - Model picker opens popover, selecting a model updates the button label and overrides the default for this compose
   - Project picker opens popover, "No project" is a default selectable, selection updates label and overrides default
   - Mic toggles voice dictation, transcribes into input, send-during-recording works
2. **Existing tests still pass**: `pnpm vitest run src/shared/ui/GlobalComposerPill.test.tsx` returns 3/3 passing. The test queries `getByRole("button", { name: /send/i })` expects exactly one match — preserve the aria-hidden pattern that gates which set of buttons is in the accessibility tree.
3. **Typecheck clean**: `pnpm typecheck` returns 0 errors.
4. **Biome clean**: `pnpm exec biome check src/` returns 0 errors after formatting.
5. **Visual proportions** roughly match Figma 114:1358 — pill is ~143px tall when expanded, ~68px when collapsed. The closed-state pill must NOT have residual height from the (collapsed) toolbar.
6. **Collapsed state unchanged**: The default state (no hover, no focus, no text, no attachments) must still show input + inline mic + send, identical to current behavior.

## What NOT to do

- Do NOT touch `src/features/chat/` files (ChatInput.tsx, ChatView.tsx, etc.). The in-chat composer has its own pickers; replicate the *patterns* but don't share code paths.
- Do NOT add new npm dependencies. Everything you need (`@tauri-apps/plugin-dialog`, `lucide-react`, popover, voice hook) is already installed.
- Do NOT implement drag-and-drop or paste-attach. Plus button → file dialog is the only entry point for attachments.
- Do NOT modify `--surface-chrome`, `--surface-button`, or any other Alex design tokens. They were finalized in commit `da9bc5c2ad`.
- Do NOT create new shared UI primitives "for reuse later" — keep the wiring inside `GlobalComposerPill.tsx`. Future chat-input-without-session work can extract.
- Do NOT change `useVoiceDictation`'s API. Adapt to it; if it really doesn't fit, surface that as a question rather than refactoring the hook.

## Verification routine

Before considering the task done, run all of these from `ui/goose2/`:

```bash
pnpm exec biome format --write src/shared/ui/GlobalComposerPill.tsx src/app/AppShell.tsx src/features/chat/stores/chatStore.ts
pnpm exec biome check src/
pnpm typecheck
pnpm vitest run src/shared/ui/GlobalComposerPill.test.tsx
```

Then have a human visually verify the four buttons in the running Tauri dev window. Tulsi will do this — leave the dev server alone if it's already running.

## When you're done

- Stage the changes (`git add` the modified files explicitly — never `-A`, never `.`)
- Show Tulsi the staged diff stat (`git diff --cached --stat`)
- Wait for her to say "commit" before creating any commit
- Do NOT push
