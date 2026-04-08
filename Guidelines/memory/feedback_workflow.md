---
name: Execution Workflow Rules
description: Read→Diagnose→Propose→Approve→Execute; never act without verification; one task at a time
type: feedback
---

Always follow this sequence before touching any file or running any command:
1. **Read** the relevant files
2. **Diagnose** root cause (not symptoms)
3. **Propose** one targeted fix with rationale
4. **Wait for explicit approval**
5. **Execute** single task only
6. **Audit** result before moving on

**Why:** Multiple incidents of acting before understanding — adding .gitignore to home dir without diagnosing VS Code auto-detection, jumping to solutions before reading the actual state. Caused wasted tokens and user frustration.

**How to apply:** If tempted to "just fix it," stop. Read first. The diagnosis step is non-negotiable. If the cause is unclear after reading, ask — don't guess.

Never write to paths outside `/Users/cristianovb/Desktop/zettel-spark-flow-main/` without explicit per-session permission.

Token efficiency rule (RPT — ROI per token): prefer small context agents (haiku) for research; save main context for decisions and architecture.
