## 2025-05-15 - [N+1 query in SQLite tag loading]
**Learning:** Identified a classic N+1 query pattern where `list_files` and `list_notes` were fetching tags for each record in a loop, significantly slowing down listings as results grew.
**Action:** Implemented `load_tags_batch` and `load_note_tags_batch` using SQL `IN` clauses and `HashMap` grouping. This reduced O(n) queries to O(1) for tag retrieval across all listing and search endpoints.
