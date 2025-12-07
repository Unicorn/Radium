# Braingrid Workflow Guide

> **Braingrid (PROJ-14) is the single source of truth for all requirements and tasks.**

## Overview

Braingrid consolidates all project requirements (REQs) and tasks in one place, providing structured task breakdown, dependency tracking, and progress monitoring. Local markdown files are deprecated in favor of Braingrid as the authoritative source.

## Core Principles

1. **Braingrid is the source of truth** - All REQs and tasks should be in Braingrid
2. **Local files are temporary** - Local markdown files should be deleted once REQs are fully ported to Braingrid with complete task breakdowns
3. **Task breakdown is required** - REQs in Braingrid must have full task breakdowns before local copies are removed
4. **Reference Braingrid** - Documentation should reference Braingrid REQs, not local files

## Porting New REQs to Braingrid

### Step 1: Create the REQ in Braingrid

Use one of these methods:

**Option A: Using `braingrid specify` (Recommended)**
```bash
braingrid specify -p PROJ-14 --prompt "Add user authentication with OAuth2"
```

**Option B: Manual creation via Braingrid UI**
1. Go to https://app.braingrid.ai
2. Navigate to PROJ-14
3. Create new requirement
4. Fill in all required fields

### Step 2: Have Braingrid Analyze and Break into Tasks

Braingrid will automatically analyze the requirement and break it down into tasks. You can:

1. **Review the auto-generated tasks** - Braingrid creates tasks based on the requirement content
2. **Refine tasks** - Edit, add, or remove tasks as needed
3. **Set dependencies** - Link tasks that depend on each other
4. **Set acceptance criteria** - Define what "done" means for each task

### Step 3: Verify Task Completeness

Before deleting local markdown copies, verify:

- [ ] REQ has all required fields (title, description, acceptance criteria)
- [ ] Tasks are fully broken down (no high-level tasks that need further breakdown)
- [ ] Task dependencies are defined
- [ ] Acceptance criteria are clear for each task
- [ ] Status tracking is accurate

### Step 4: Update Local References

Update all local documentation to reference Braingrid:

```markdown
### REQ-163: Context Files ([Braingrid](https://app.braingrid.ai/requirements/overview?id=958cb2e0-8d3d-4253-8083-64f0c799905f))
```

### Step 5: Delete Local Markdown Copies

Once verified complete:
- Delete local REQ markdown files (e.g., `BG-REQ-11-*.md`)
- Remove outdated status tracking from `PROGRESS.md`
- Keep only high-level roadmap summaries if needed

## Working with Braingrid REQs

### Viewing REQs

```bash
# List all REQs
braingrid requirement list -p PROJ-14

# View specific REQ
braingrid requirement show REQ-163 -p PROJ-14

# List tasks for a REQ
braingrid task list -r REQ-163 -p PROJ-14
```

### Updating Task Status

```bash
# Start working on a task
braingrid task update TASK-1 -p PROJ-14 --status IN_PROGRESS

# Complete a task
braingrid task update TASK-1 -p PROJ-14 --status COMPLETED \
  --notes "Completed in commit abc123. Implements feature X."
```

### Creating New REQs

For substantial new work:
1. Check if a related REQ already exists
2. If not, create new REQ using `braingrid specify`
3. Let Braingrid analyze and break into tasks
4. Review and refine the task breakdown

## Handling Duplicates

When duplicates are found:

1. **Compare robustness** - Which REQ has:
   - More complete task breakdown?
   - Better task dependencies?
   - More detailed acceptance criteria?
   - Accurate status tracking?

2. **Favor the robust version** - Keep the more complete REQ

3. **Delete the duplicate** - Remove the less robust duplicate from Braingrid

4. **Update references** - Update all local references to point to the kept REQ

## REQs Missing Task Breakdowns

Many REQs are marked COMPLETED but have 0 tasks. These need task breakdowns:

1. **Review the REQ content** - Understand what was implemented
2. **Break down into tasks** - Create tasks that represent the work done
3. **Mark tasks as COMPLETED** - Update task statuses to reflect completion
4. **Update REQ status** - Ensure REQ status matches task completion

## Integration with Local Progress

### PROGRESS.md Updates

When updating `PROGRESS.md`:

- Reference Braingrid REQs: `REQ-163: Context Files ([Braingrid](URL))`
- Link to Braingrid for current status
- Remove detailed task lists (Braingrid is source of truth)
- Keep high-level summaries only

### Commit Messages

Reference Braingrid REQs in commits:

```bash
git commit -m "feat(context): implement context files [REQ-163]"
```

## Verification Checklist

Before considering a REQ "complete" in Braingrid:

- [ ] REQ exists in Braingrid with all required fields
- [ ] Tasks are fully broken down (no vague tasks)
- [ ] Task dependencies are defined
- [ ] Acceptance criteria are clear
- [ ] Status tracking is accurate
- [ ] Local markdown copies are deleted
- [ ] All references point to Braingrid
- [ ] Documentation is consistent

## Examples

### Example 1: Porting a Completed Feature

1. REQ-163 (Context Files) was completed locally
2. Ported to Braingrid with full task breakdown (17 tasks)
3. All tasks marked COMPLETED
4. Local files `BG-REQ-11-*.md` deleted
5. `PROGRESS.md` updated to reference REQ-163

### Example 2: Creating a New REQ

1. New feature needed: "Add OAuth authentication"
2. Run: `braingrid specify -p PROJ-14 --prompt "Add OAuth authentication"`
3. Braingrid creates REQ with task breakdown
4. Review and refine tasks
5. Start working on tasks, updating status in Braingrid

## Troubleshooting

### REQ not found in Braingrid

- Check if it was created with a different name
- Search by content/keywords
- May need to be ported from local files

### Tasks seem incomplete

- Review REQ content for missing functionality
- Break down high-level tasks further
- Add missing tasks based on implementation

### Duplicate REQs

- Compare both REQs for robustness
- Keep the more complete one
- Delete the duplicate
- Update all references

## Best Practices

1. **Always check Braingrid first** - Before starting work, check for existing REQs
2. **Update Braingrid immediately** - Update task status as you work
3. **Reference in commits** - Include REQ/TASK IDs in commit messages
4. **Keep Braingrid current** - Don't let local files get out of sync
5. **Delete local copies** - Remove local markdown once Braingrid is complete

## Resources

- **Braingrid UI**: https://app.braingrid.ai
- **Project**: PROJ-14
- **CLI Help**: `braingrid --help`
- **Agent Rules**: See `docs/rules/AGENT_RULES.md` for agent workflow

