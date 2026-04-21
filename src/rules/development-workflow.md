# Development Workflow

## Core Loop

**Define Problem** → **Small Safe Change** → **Review Change** → **Refactor** — Repeat.

## Problem 1-Pager

Before you start coding, if the problem is complex or unclear, draft a **Problem 1-Pager** including the following items. If any items are ambiguous, request an interview to clarify.

* **Background:** Context and motivation for the change.
* **Problem:** What specific issue are we trying to solve?
* **Goal:** What is the definition of success (the "success state")?
* **Non-goals:** What is explicitly out of scope?
* **Constraints:** Mandatory technical or business constraints.

## Feature Implementation Workflow

1. **Plan First**
   - Use **planner** agent to create implementation plan
   - Identify dependencies and risks
   - Break down into phases

2. **TDD Approach**
   - Use **tdd-guide** agent
   - Write tests first (RED)
   - Implement to pass tests (GREEN)
   - Refactor (IMPROVE)
   - Verify 80%+ coverage

3. **Code Review**
   - Use **code-reviewer** agent immediately after writing code
   - Address CRITICAL and HIGH issues
   - Fix MEDIUM issues when possible

4. **Commit & Push**
   - Follow `commit-convention.md` for commit messages
   - Create PR per `pull-request-rules.md`
