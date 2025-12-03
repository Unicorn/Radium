# Radium Roadmap

> **Organized roadmap for achieving legacy system feature parity**

## 📚 Quick Navigation

### Main Documents

1. **[00-project-overview.md](./00-project-overview.md)** - Project vision, architecture, and business model
2. **[01-completed.md](./01-completed.md)** - ✅ All completed work (Milestones 1-5)
3. **[02-now-next-later.md](./02-now-next-later.md)** - 🎯 Prioritized features (Now/Next/Later)
4. **[03-implementation-plan.md](./03-implementation-plan.md)** - 📋 0-10 step implementation plan

### Reference Documents

- **[legacy-system-feature-backlog.md](./legacy-system-feature-backlog.md)** - Complete feature catalog from legacy system
- **[gemini-cli-enhancements.md](../features/gemini-cli-enhancements.md)** - Features and patterns from gemini-cli integrated into Radium

### Architecture Documents

- **[architecture-backend.md](./architecture-backend.md)** - Backend architecture details (current state)
- **[architecture-cli-tui.md](./architecture-cli-tui.md)** - CLI and TUI design (current state)
- **[architecture-web-desktop.md](./architecture-web-desktop.md)** - Web and desktop app architecture (current state)
- **[04-milestones-and-timeline.md](./04-milestones-and-timeline.md)** - Original milestone timeline (reference)

### Progress Tracking

- **[PROGRESS.md](./PROGRESS.md)** - Detailed task-level progress tracker (active work, blockers, test status)
- **[01-completed.md](./01-completed.md)** - High-level summary of completed milestones

---

## 🎯 Current Status

**Completed**: All core infrastructure (Milestones 1-5) ✅

**Current Focus**: Step 0 - Workspace System (see [02-now-next-later.md](./02-now-next-later.md))

**Goal**: Achieve complete legacy system feature parity in 10 steps (5-8 weeks), including enhancements from gemini-cli

---

## 📖 How to Use This Roadmap

### For Planning

1. Start with **[02-now-next-later.md](./02-now-next-later.md)** to see prioritized features
2. Review **[03-implementation-plan.md](./03-implementation-plan.md)** for detailed step-by-step tasks
3. Reference **[legacy-system-feature-backlog.md](./legacy-system-feature-backlog.md)** for complete feature details
4. Check **[gemini-cli-enhancements.md](../features/gemini-cli-enhancements.md)** for features integrated from gemini-cli

### For Implementation

1. Check **[01-completed.md](./01-completed.md)** to see what's already done
2. Follow **[03-implementation-plan.md](./03-implementation-plan.md)** step-by-step
3. Reference feature backlog for specific implementation details

### For Understanding

1. Read **[00-project-overview.md](./00-project-overview.md)** for project vision
2. Review architecture documents for design decisions
3. Check completed work to understand current state

---

## 🗂️ Document Organization

### Main Roadmap (Primary)

- **00-project-overview.md** - Project vision and overview
- **01-completed.md** - What's been done
- **02-now-next-later.md** - Prioritized roadmap
- **03-implementation-plan.md** - Detailed implementation steps

### Reference (Secondary)

- **legacy-system-feature-backlog.md** - Complete feature catalog from legacy system
- **gemini-cli-enhancements.md** - Features and patterns from gemini-cli integrated into Radium
- **PROGRESS.md** - Detailed task-level progress (see note below)

### Architecture & Strategy (Reference)

- **architecture-backend.md** - Backend architecture (updated to current state)
- **architecture-cli-tui.md** - CLI/TUI architecture (updated to current state)
- **architecture-web-desktop.md** - Web/Desktop architecture (updated to current state)
- **UI_STRATEGY.md** - UI implementation strategy (Tamagui, Expo, @unicorn-love)
- **CLAUDE_RULES.md** - Agent rules and guidelines for Claude Code
- **04-milestones-and-timeline.md** - Original milestone timeline (reference)

### Progress Tracking

**PROGRESS.md vs 01-completed.md:**
- **PROGRESS.md**: Detailed task-level tracking, active work, blockers, test status, architecture decisions
- **01-completed.md**: High-level summary of completed milestones, what's been built, key achievements

---

## 🚀 Getting Started

If you're starting work on Radium:

1. **Read** [00-project-overview.md](./00-project-overview.md) for context
2. **Check** [01-completed.md](./01-completed.md) to see what exists
3. **Review** [02-now-next-later.md](./02-now-next-later.md) for priorities
4. **Follow** [03-implementation-plan.md](./03-implementation-plan.md) for tasks
5. **Reference** [legacy-system-feature-backlog.md](./legacy-system-feature-backlog.md) for details

---

## 📝 Notes

- All documents are living and updated as work progresses
- The 0-10 step plan integrates the complete feature backlog
- Now/Next/Later approach helps prioritize work
- Reference documents provide detailed context when needed

---

## 🧪 Test Coverage Status

**Current Coverage**: ~37.61% (2080/5531 lines)  
**Target Coverage**: 100%  
**Coverage Gap**: 62.39% (3,451 lines)

**Critical Test Gaps**:
- **CLI Commands**: 0% coverage (~1,200 lines) - 🔴 Critical
- **Server/gRPC**: 0% coverage (~167 lines) - 🔴 Critical  
- **TUI Application**: 0% coverage (~500 lines) - 🟡 Medium
- **Workflow Service**: Partial coverage (~70%) - 🟡 Medium

**See [TEST_COVERAGE_REPORT.md](./project/TEST_COVERAGE_REPORT.md) for detailed coverage analysis and test requirements.**

---

**Last Updated**: 2025-12-03 (includes test coverage report and gemini-cli enhancements integration)

