# Phase 2 — Join Engine

## Tasks

1. Primary source iteration — yield one context per entry
2. 1:1 join resolution: for each primary entry, look up matching record in secondary source by join keys
3. 1:N join resolution: same, but collect all matches into a list
4. Global sources: inject entire dataset into every context
5. Build merged context object per primary entry with all namespaces resolved

## Exit Criteria

- Unit: 1:1 join — 3 classes, 3 instructors, assert each context has correct instructor
- Unit: 1:N join — class with 5 students, assert `{{#each}}` iterates 5 times
- Unit: Composite join — match on 2 keys simultaneously
- Unit: Global source — config values accessible in every context
- Unit: Missing join match → structured error with primary entry index + namespace
- Unit: Ambiguous 1:1 join (multiple matches) → error
