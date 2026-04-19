# E2E Verification — Phase 0.5 Complete

## Setup (pre-launch)

```bash
# 1. Ensure clean state
pkill -f "tauri dev"; pkill -f vite; pkill -9 app
rm -rf ~/Library/Application\ Support/com.viboai.app*

# 2. Prelaunch cleanup + launch
cd /Users/cristianovb/Desktop/zettel-spark-flow
bash scripts/dev-prelaunch.sh
bun run tauri:dev
```

## Expected on first launch (before user action)

- Tauri window opens (native, not localhost) 
- WebKit origin: `tauri://localhost/` (no localhost dev server)
- Bundle identifier in use: `com.viboai.app.dev` (check: file system should be `~/Library/Application Support/com.viboai.app.dev/`)
- Data root on disk: `viboai/myspace/`, `viboai/database/` both exist
- SQLite at: `viboai/database/vibo.db` (NOT at root)
- `viboai/myspace/` contains all 9 reserved dirs: `notes/`, `tasks/`, `agents/`, `skills/`, `roles/`, `providers/`, `tools/`, `mcp/`, `plugin/`
- Onboarding wizard appears (fresh bundle) with "Welcome" step

## 8-Step Verification Flow

### Step 1: Onboarding wizard → complete
- Fill: User name = "Test User", Model = "instruct-pack", Cloud fallback = "none", Auth = "biometrics"
- Click: "Get started" → wizard closes
- Result: localStorage key `zettel-onboarding-done` = `"true"`

-> app opened like expected , no old notes or task or folders where there like expected on a reset. I reminded of my onboarding process so I didnt had to do it. ( check if this is what was supposed to happen)

### Step 2: Create note with special characters
- Click: `+` floating button → "New note" dialog opens
- Title: `Hello "world" & <tags>` (test HTML/JSON safety)
- Content: 
  ```
  # Heading
  
  - [ ] Task item in note
  ```
- Click: Save → note appears in Notebook view under "notes" folder
- File check: `ls ~/Library/Application Support/com.viboai.app.dev/viboai/myspace/notes/{uuid}.md` exists
- YAML inspection: open file in editor, verify YAML frontmatter (`id`, `type: note`, `status: inbox`, `folder: notes`, `created`, `modified`)

-> created a new note like requested and with tag and text like request.
Pls you do verify the following: file check  and yaml inspections.


### Step 3: Create folder "Research"
- Click: folder icon in Notebook sidebar (or create-folder button in new note dialog)
- Name: "Research"
- Result: folder appears in sidebar under "notes"
- File check: `ls -d ~/Library/Application Support/com.viboai.app.dev/viboai/myspace/Research/` exists
- SQL check: `sqlite3 vibo.db "SELECT name FROM folders"` shows only `Research` (NOT `notes`, `tasks`, or other reserved names)

-> folder created
Sql check need to be you 

### Step 4: Move note to Research folder
- Right-click (or drag) the "Hello world" note in Notebook sidebar
- Change folder: "notes" → "Research"
- Result: note moves into Research section; no orphan left in `myspace/notes/`
- File check: file now at `~/Library/Application Support/com.viboai.app.dev/viboai/myspace/Research/{uuid}.md`
- File check: `myspace/notes/{uuid}.md` is GONE (not present)
- YAML: `folder: Research` in file

-> done , ui seems to have done all the intended
Need to confirm sql - is notes not on research folder? If true it changed correctly, did it delete the previous copy from notes folder when it moved to research folder? If yes we can assume autosaves are also qorking like intended


### Step 5: Create task in Kanban
- Click: bottom nav → Tasks (Kanban view)
- Click: `+` in Inbox column
- Title: "Write test case"
- Content: 
  ```
  ## Task
  Write a test case for Phase 0.5
  - [ ] Test folder creation
  - [ ] Test note movement
  ```
- Click: Save → task appears in Inbox column
- File check: `ls ~/Library/Application Support/com.viboai.app.dev/viboai/myspace/tasks/{uuid}.md` exists
- YAML: `type: task`, `folder: tasks`, `column_id: inbox`

-> done added a tag  and a second using "#tag" resulting in a ##tag to test 


### Step 6: Reject reserved folder name
- Create folder named "notes" (attempt)
- Expected: error message "Cannot create folder: 'notes' is reserved"
- (Verify same for: tasks, agents, skills, roles, providers, tools, mcp, plugin)

-> on notes view using the quick button to add a folder on the top right, it rejected creating "notes" folder like expected, but ui should give a message like "this folder already exist at: [.......]
Using the FAB also add the same result wich is good and also missed to send an elegant instruction error/notification for the ui.

I have also tried to create a tasks folder and was rejected liked expected and well done, missed also a message on the ui
I have also moved our task to research folder.

### Step 7: Relaunch app
- Close Tauri window (Cmd+Q)
- `sleep 2`
- `bun run tauri:dev` again
- Expected: Onboarding wizard does NOT appear (localStorage survived; silent-reuse applies)
- Workspace loads with note + task visible
- Sidebar shows "Research" folder with note inside
- Kanban shows task in Inbox

-> app quit and relaunched like expected, no onboarding wizard and remembers my choices.
Workspace loads with note and task visible
Sidebar shows research folder
Kanban sows task in the " Inbox" 
 

### Step 8: Open vault in Obsidian
- Open Obsidian (or init a new vault)
- Point Obsidian to: `~/Library/Application Support/com.viboai.app.dev/viboai/myspace/`
- Expected: Obsidian reads all `.md` files, YAML renders correctly
- Folder structure appears as-is (Research, notes, tasks, agents, etc.)
- Note + task YAML render natively in Obsidian

-> I was not able to point obsidian to '~/Library/Application Support/com.viboai.app.dev/viboai/myspace/`
I was not able too visually  find the folder Library/Application Support/com.viboai.app.dev

So I made "obsidian vault test" on users/cristianovb/desktop
I will create an obsidian test note "Obsidian test note"
Osidian has an option wich can select any folder and turn it into a vault, in our case a "myspace" 
They also allow you to select in wich directory/place you want to create the new folder 

To finish this we should move the file I created on obsidian to vibo -> myspace/notes
Then send the Hello “world" note we created and moved to research folder on vibo to obsidian vault test (vault/folder)

......

To note for the future:

We should have a 3 dot button/option on note that pops a small menu for this type of folder management.
Moving a folder moves everything it has inside with it to avoid lost files.
Lets think based on our ui ,ux and backend an easy way to implement this elegantly and without to much effort.

When we build we make elements and components that can be used repetitively in many cases like " system bubble messages/erros) tauri events on top menu like a small slider
/this are phase 10 improvements but to have in mind today for how we build and think about things


## Success Criteria


## -> important note before reading under
I was not able to physically find them or verify you need to smoke test this with an subagent


- ✅ All 9 reserved dirs created on first launch - 
- ✅ DB at `viboai/database/vibo.db`, not root 
- ✅ No reserved names in SQL `folders` table after any operation
- ✅ Note/task creation, folder creation, move operations all work end-to-end
- ✅ No orphan files left after moves
- ✅ Onboarding silent-reuse works (wizard doesn't re-appear on relaunch with existing data)
- ✅ Obsidian cross-compat: YAML renders, folder structure visible
- ✅ Tauri native window (not localhost preview)

## Troubleshooting

| Issue | Check |
|---|---|
| Onboarding re-appears after relaunch | localStorage `zettel-onboarding-done` key exists; snapshot has notes/folders; silent-reuse logic fires |
| Folder appears in sidebar but not creatable | Reserved-name guard in `db::create_folder` is active; test with non-reserved name |
| Note moves but old copy remains | Orphan cleanup post-transaction is running; check logs for warnings |
| DB file at root, not under `database/` | Migration in `lib.rs` setup handler didn't run; check if legacy DB exists and `target` dir permissions |

---

Generated 2026-04-18 post-Phase 0.5.