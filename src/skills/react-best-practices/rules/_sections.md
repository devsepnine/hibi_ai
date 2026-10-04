# Sections

This file defines all sections, their ordering, impact levels, and descriptions.
The section ID, shown after each heading as its prefix, is the filename prefix used to group rules.

---

## 1. Eliminating Waterfalls, prefix `async`

**Impact:** CRITICAL  
**Description:** Waterfalls are the #1 performance killer. Each sequential await adds full network latency. Eliminating them yields the largest gains.

## 2. Bundle Size Optimization, prefix `bundle`

**Impact:** CRITICAL  
**Description:** Reducing initial bundle size improves Time to Interactive and Largest Contentful Paint.

## 3. Server-Side Performance, prefix `server`

**Impact:** HIGH  
**Description:** Optimizing server-side rendering and data fetching eliminates server-side waterfalls and reduces response times.

## 4. Client-Side Data Fetching, prefix `client`

**Impact:** MEDIUM-HIGH  
**Description:** Automatic deduplication and efficient data fetching patterns reduce redundant network requests.

## 5. Re-render Optimization, prefix `rerender`

**Impact:** MEDIUM  
**Description:** Reducing unnecessary re-renders minimizes wasted computation and improves UI responsiveness.

## 6. Rendering Performance, prefix `rendering`

**Impact:** MEDIUM  
**Description:** Optimizing the rendering process reduces the work the browser needs to do.

## 7. JavaScript Performance, prefix `js`

**Impact:** LOW-MEDIUM  
**Description:** Micro-optimizations for hot paths can add up to meaningful improvements.

## 8. Advanced Patterns, prefix `advanced`

**Impact:** LOW  
**Description:** Advanced patterns for specific cases that require careful implementation.
