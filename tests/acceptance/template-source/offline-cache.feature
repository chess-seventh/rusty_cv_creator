# Feature: Generating offline from a cached template (TS-04)
#
# Documentation SSOT mapped to concrete Rust tests (no cucumber-rust harness).
# See the traceability table in docs/feature/template-source/feature-delta.md
# (Wave: DISTILL). The reuse-vs-fetch-vs-abort decision is a pure `CacheAction`
# matrix; cache-key derivation is deterministic. All pending DELIVER (RED scaffolds).
#
# @in-memory: the decision is a pure function over (cache present, remote reachable);
#   no real network is contacted in these specs.

Feature: Generating offline from a cached template
  As Francesco on a flaky connection
  I want the last good template reused when a fetch fails
  So that a network hiccup never blocks an application

  @US-04 @error @in-memory @contract-shape:unbounded-preservation
  Scenario: Offline, the most recent cached template is reused with a warning
    Given a cached template exists for the requested repository and version
    And the remote is unreachable
    When the cache decision is made
    Then the cached template is reused

  @US-04 @error @in-memory @contract-shape:unbounded-preservation
  Scenario: With no cache and no network the run aborts without a partial CV
    Given no cached template exists for the requested repository and version
    And the remote is unreachable
    When the cache decision is made
    Then the run aborts and no partial CV is produced

  @US-04 @in-memory @contract-shape:bounded-change
  Scenario: A successful fetch refreshes the cache for next time
    Given a cached template exists and the remote is reachable
    When the cache decision is made
    Then the cache is refreshed by fetch and checkout

  @US-04 @property @contract-shape:pure-function
  Scenario: The reuse-or-fetch-or-abort decision is total over cache and network state
    Given any combination of cache presence and remote reachability
    When the cache decision is made
    Then exactly one of clone, fetch-checkout, reuse, or abort is chosen

  @US-04 @property @edge @contract-shape:pure-function
  Scenario: A repository and version map to one deterministic cache entry
    Given the same repository and version
    When the cache key is derived
    Then the same cache entry is always produced
