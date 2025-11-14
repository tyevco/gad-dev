---
description: Pick and complete a single work item from work_items.md
---

# Work Item Development Protocol

You are an agent tasked with completing a **SINGLE** work item from the project backlog.

## Instructions

### 1. Read the Work Items List
- Read `/home/user/gad-dev/work_items.md` to see all available work items
- Identify items marked with `- [ ]` (not started)
- Avoid items marked with `[~]` (in progress by another agent) or `[x]` (completed)

### 2. Select ONE Work Item
- Choose the **NEXT** uncompleted item from the highest priority phase
- Phases should be completed sequentially (Phase 1 → 2 → 3, etc.)
- Within a phase, work items should generally be completed top-to-bottom
- **IMPORTANT:** Only select ONE item per session. Do not work on multiple items.

### 3. Mark Item as In Progress
- Before starting work, update `work_items.md`
- Change `- [ ]` to `- [~]` for the item you selected
- Add your session identifier or timestamp to track who is working on it
- Commit this change: `git add work_items.md && git commit -m "Start work on: [item description]"`

### 4. Complete the Work
- Focus ONLY on the selected work item
- Read relevant code files mentioned in the work item location
- Implement the feature following best practices:
  - Write clean, documented Rust code
  - Follow existing code style and patterns
  - Add error handling
  - Test your implementation if possible
- Stay within the scope of the single work item

### 5. Mark Item as Complete
- After successful implementation, update `work_items.md`
- Change `- [~]` to `- [x]` for the completed item
- Commit your changes:
  ```bash
  git add .
  git commit -m "Complete: [brief description of work item]"
  ```

### 6. Push Your Work
- Push to the designated branch: `claude/repo-state-analysis-01Et4TisJ7E4Kwz6tJNoxYfk`
- Use: `git push -u origin claude/repo-state-analysis-01Et4TisJ7E4Kwz6tJNoxYfk`
- If push fails due to network, retry up to 4 times with exponential backoff (2s, 4s, 8s, 16s)

## Scope Constraints

**DO:**
- ✅ Pick ONE item only
- ✅ Mark item as in-progress before starting
- ✅ Stay focused on the specific task
- ✅ Mark as complete when done
- ✅ Commit and push your changes
- ✅ Ask questions if the work item is unclear

**DON'T:**
- ❌ Work on multiple items in one session
- ❌ Start work without marking the item as in-progress
- ❌ Expand scope beyond the single work item
- ❌ Skip items to work on "more interesting" tasks
- ❌ Leave items marked as in-progress without completing them
- ❌ Forget to commit and push when complete

## Example Workflow

1. Agent runs `/work-item` command
2. Agent reads `work_items.md`
3. Agent identifies first uncompleted item: "Implement complete Asset struct with metadata fields"
4. Agent marks it `[~]` in work_items.md and commits
5. Agent reads `rust/src/asset_library/asset.rs`
6. Agent implements the metadata fields (author, version, description, tags, etc.)
7. Agent tests compilation: `cd rust && cargo build`
8. Agent marks item `[x]` in work_items.md
9. Agent commits all changes with descriptive message
10. Agent pushes to branch
11. Agent reports completion to user

## File Locations Reference

Key files you may need to work with:
- **Asset structures:** `rust/src/asset_library/asset.rs`
- **Asset Manager:** `rust/src/asset_library/asset_manager.rs`
- **Config Manager:** `rust/src/asset_library/config_manager.rs`
- **Main Plugin:** `rust/src/asset_library_extension.rs`
- **GUI Components:** `rust/src/gui/asset_library_gui.rs`
- **Dependencies:** `rust/Cargo.toml`

## Testing Your Work

After implementing a work item:
1. Build the project: `cd rust && cargo build`
2. Fix any compilation errors
3. If possible, test in Godot editor (open `godot/project.godot`)
4. Verify your changes don't break existing functionality

## Reporting

When complete, provide a brief report:
- ✅ Work item completed: [description]
- 📁 Files modified: [list]
- 🔨 Changes made: [brief summary]
- ✅ Build status: [success/failure]
- 📝 Commit hash: [hash]
- 🚀 Pushed to: [branch]

---

**Remember:** Quality over quantity. Complete ONE item well rather than rushing through multiple items.
