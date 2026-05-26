# TSSP Refactoring & Optimization Guide

## Overview
This document outlines the systematic approach to modernizing the TSSP codebase for maintainability, performance, and developer experience.

## Frontend Refactoring Strategy

### Phase 1: Modularization (HIGHEST PRIORITY)

#### Large Monolithic Views (Needs Breaking Down)
These files need to be split into smaller, focused components:

- **DriveView.svelte** (1300 lines)
  - DriveFileGrid.svelte - File grid display
  - DriveFileList.svelte - File list display  
  - DriveHeader.svelte - Filter and view mode controls
  - DriveSidebar.svelte - Folder tree and filters
  - DriveLensService.ts - Filter/lens logic extraction

- **OperationsView.svelte** (1102 lines)
  - SystemMonitor.svelte - System stats display
  - ProcessList.svelte - Running processes
  - LogViewer.svelte - System logs
  - HealthStatus.svelte - Health checks

- **HomeLauncher.svelte** (1047 lines)
  - QuickActions.svelte - Action tiles
  - RecentItems.svelte - Recent files/notes
  - Statistics.svelte - System stats
  - Shortcuts.svelte - Keyboard shortcuts

#### Service Layer Extractions (IN PROGRESS)
✅ appConfigService.ts - App metadata and routing
✅ keyboardService.ts - Global keyboard handling
✅ uploadEventService.ts - Upload event management

Still needed:
- driveService refactoring - Extract business logic from DriveView
- notesService refactoring - Separate note operations from UI
- workspaceService refactoring - Extract editor logic

### Phase 2: Dead Code Removal

#### Known Issues
- 660+ `todo!()`, `unimplemented!()`, `panic!()` instances in Rust
- Stub implementations in frontend services
- Unused imports and exports
- Test code in production files

#### TODO Comments to Address
- Block nesting (BlockEditor.svelte:116) - Needs implementation or removal
- Chunked upload placeholder (chunkedUploadService.ts:154) - Needs real implementation
- Store refactoring TODOs (ui/index.ts, shell/index.ts)

### Phase 3: Performance Optimization

#### Bundle Size Reduction
- Audit and remove unused dependencies
- Implement code splitting for large views
- Lazy load route components
- Optimize asset imports

#### Runtime Performance
- Implement virtual scrolling for large file lists
- Optimize store subscriptions (avoid unnecessary re-renders)
- Batch database operations
- Cache frequently accessed data

### Phase 4: Architecture Consistency

#### Store Organization
Current: Mixed organization with ui, shell, operations, data
Target: Consistent, predictable structure

#### API Integration
- Create unified API request wrapper
- Implement request deduplication
- Add caching layer
- Handle retries and timeouts consistently

## Backend Refactoring Strategy

### Phase 1: Module Organization

#### Large Files Needing Splitting
- **tsspd/src/workspaces.rs** (2237 lines)
  - Split into: models.rs, handlers.rs, store.rs, errors.rs
  
- **tsspd/src/upload.rs** (1231 lines)
  - Extract: chunk_manager.rs, session.rs, completion.rs

- **tssp-adapter-sqlite/src/lib.rs** (1155 lines)
  - Create modular subdirectory structure

### Phase 2: Error Handling

#### Replace Panics
Goal: Zero panics in production paths
- Replace `unwrap()` with proper error types
- Implement comprehensive error handling
- Add context to error messages

### Phase 3: Testing

#### Test Organization
- Move test code out of src/ files
- Create tests/ directory structure
- Implement integration tests
- Add property-based tests for critical paths

## Priority Order

1. **QUICK WINS** (< 1 hour each)
   - [✅] Enable disabled capabilities endpoint
   - [ ] Remove obvious dead code
   - [ ] Fix import organization
   - [ ] Update TODO/FIXME comments

2. **HIGH IMPACT** (1-2 hours)
   - [✅] Extract services from App.svelte
   - [ ] Break down DriveView
   - [ ] Modularize workspaces.rs
   - [ ] Optimize bundle imports

3. **MEDIUM EFFORT** (2-4 hours)
   - [ ] Complete block nesting feature
   - [ ] Fix chunked upload implementation
   - [ ] Refactor store organization
   - [ ] Add proper error handling

4. **LONG TERM** (4+ hours)
   - [ ] Complete modularization
   - [ ] Implement missing features
   - [ ] Comprehensive testing
   - [ ] Performance profiling

## Code Quality Metrics

### Target State
- Max 300-400 lines per component/file
- 0 panics in production code paths
- 100% error type coverage
- <50 ms first contentful paint
- <200 KB gzipped bundle

### Current State
- Max 2237 lines (workspaces.rs)
- 660+ panic/todo instances
- Inconsistent error handling
- Performance unoptimized
- Bundle size ~500+ KB estimated

## Tools & Approaches

### Frontend
- SvelteKit route-based code splitting
- Dynamic imports for large components
- Svelte store optimization
- Bundle analysis tools

### Backend
- Rust module system and visibility
- Error propagation patterns
- Async/await best practices
- Benchmark harness

## References
- Project CLAUDE.md for architecture rules
- Svelte 5 patterns and best practices
- Rust hexagonal architecture pattern
- Orange Pi resource constraints
