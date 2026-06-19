# Sections

This file defines all sections, their ordering, impact levels, and descriptions.
The section ID (in parentheses) is the filename prefix used to group rules.

---

## 1. Complexity & Context (Cynefin) (complexity)

**Impact:** HIGH  
**Description:** Match the coupling strategy to how well the problem is
understood, using the Cynefin domains to decide between up-front structure and
deferred decisions.

## 2. Coupling Types & Threat Ranking (coupling)

**Impact:** CRITICAL  
**Description:** Identify and reduce harmful coupling; modern threat order is
Control > External > Common > Contents > Stamp > Data.

## 3. Dependency Direction & Structure (dependency)

**Impact:** CRITICAL  
**Description:** Keep dependencies unidirectional, acyclic, and isolated by
change-rate so volatile parts cannot drag stable parts with them.

## 4. Abstraction & Module Boundary (abstraction)

**Impact:** HIGH  
**Description:** Publish consistent, minimal, abstracted knowledge across module
boundaries so callers depend on intent rather than implementation.

## 5. Layered & Monorepo Architecture (architecture)

**Impact:** MEDIUM  
**Description:** Apply layered structure and a Turbo monorepo with one-way
dependency flow from apps to shared packages.

## 6. AI-Friendly Ownership (ai)

**Impact:** MEDIUM  
**Description:** Structure code so an AI agent can own and modify isolated parts
within a limited context window.
