# Sections

This file defines all sections, their ordering, impact levels, and descriptions.
The section ID, shown after the heading, is the filename prefix used to group rules.

---

## 1. Component Architecture, prefix `architecture`

**Impact:** HIGH  
**Description:** Fundamental patterns for structuring components to avoid prop
proliferation and enable flexible composition.

## 2. State Management, prefix `state`

**Impact:** MEDIUM  
**Description:** Patterns for lifting state and managing shared context across
composed components.

## 3. Implementation Patterns, prefix `patterns`

**Impact:** MEDIUM  
**Description:** Specific techniques for implementing compound components and
context providers.

## 4. React 19 APIs, prefix `react19`

**Impact:** MEDIUM  
**Description:** React 19+ only. Don't use `forwardRef`; use `use()` instead of `useContext()`.
