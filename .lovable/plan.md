

## Plan: Agents Default Count, OAuth-style Cloud Providers, Chat Model Selector

### 1. Default to 2 Agents from Onboarding Package

The `loadAgents()` function in `AgentsView.tsx` currently creates 1 starter agent. Update it to create 2 agents based on the selected onboarding package (e.g., Instruct Pack creates "LFM Instruct" + "LFM Compact" agents; Thinking Pack creates "LFM Thinking" + "LFM Compact").

**File**: `src/components/AgentsView.tsx` -- update `loadAgents()` to read the selected package from `getAiConfig()` and initialize 2 agents matching the 2 models in that package.

### 2. Cloud Providers: OAuth-style Connect Flow (Settings)

Replace the current API key paste UI in `CloudProvidersSection.tsx` with an OAuth-style "Connect" button pattern:

- Each provider shows a card with a "Connect" button (simulates OAuth -- since real OAuth requires backend/Rust later, this will open a dialog asking for the key but styled as a connection flow)
- Once connected, the card shows a green "Connected" status and a model selector dropdown
- A "Disconnect" button to remove the key
- The API key input is hidden behind the "Connect" action rather than always visible

This is a UI/UX improvement only -- actual OAuth will be wired in Rust later. The underlying storage (`setCloudKey`, `getCloudKeys`) stays the same.

**File**: `src/components/settings/CloudProvidersSection.tsx` -- redesign `ProviderField` to show Connect/Connected state with model selector toggle bar.

### 3. Inline Chat: Provider + Model Selector

The `ModelSwitcher` in `ChatAssistant.tsx` already exists but needs refinement:

- Show provider name and selected model clearly
- Allow switching provider first, then model within that provider
- Ensure both local and all connected cloud providers appear
- Make it more prominent/accessible in the chat header

**File**: `src/components/ChatAssistant.tsx` -- refine `ModelSwitcher` to show a two-level selector (provider, then model) with clearer labeling.

---

### Technical Details

**AgentsView.tsx**:
- Read `getAiConfig().localModel` to determine package (`"instruct-pack"` or `"thinking-pack"`)
- Map to 2 agents: e.g., `[{name: "LFM Instruct", role: "General Assistant"}, {name: "LFM Compact", role: "Quick Tasks"}]`

**CloudProvidersSection.tsx**:
- New `ProviderField` states: disconnected (shows "Connect" button) vs connected (shows model selector + "Disconnect")
- "Connect" button opens a small inline form/dialog for the API key, then saves and transitions to connected state
- Connected state shows a toggle/dropdown bar for model selection from `provider.models`

**ChatAssistant.tsx**:
- Restructure `ModelSwitcher` dropdown: group by provider headers, show models under each connected provider
- Display current selection as "Provider / Model" format

