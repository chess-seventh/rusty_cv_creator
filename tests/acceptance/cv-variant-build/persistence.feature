# Feature: Recording applications and reaching the records store
#
# RETROACTIVE backfill (LEAN). Documentation SSOT mapped to existing GREEN Rust
# tests (no cucumber-rust harness). See the traceability table in
# docs/feature/cv-variant-build/feature-delta.md (Wave: DISTILL).
#
# @in-memory: persistence is exercised against in-memory SQLite via the
#   backend-agnostic connection (DbConnection MultiConnection). Real-IO boundary:
#   in production the same query code runs against Postgres reached over the
#   secure network; that real backend is NOT exercised by these specs.

Feature: Recording applications and reaching the records store
  As Francesco, tracking where I have applied
  I want each application recorded and retrievable
  So that I avoid duplicate applications and keep an audit trail

  @US-04 @in-memory @contract-shape:bounded-change
  Scenario: An application is recorded when saving is opted in
    Given Francesco applies to "ACME" for a "Senior DevOps Engineer" role
    When the application is saved
    Then a record holds the job title, company, quote, PDF path and date

  @US-04 @edge @in-memory @contract-shape:bounded-change
  Scenario: Re-saving the same application keeps a single record
    Given an application to "ACME" for a "SRE" role is already recorded
    When the same application is saved again with a different PDF path
    Then the original record is kept and no duplicate is created

  @US-04 @in-memory @contract-shape:bounded-change
  Scenario: A motivational quote is stored with the application
    Given Francesco applies to "ACME" for a "Platform" role with a quote "stay hungry"
    When the application is saved
    Then the stored record carries the quote "stay hungry"

  @US-04 @in-memory @contract-shape:pure-function
  Scenario: Recorded applications can be listed back
    Given two applications have been recorded
    When the recorded applications are listed
    Then both recorded PDF paths appear in the list

  @US-04 @edge @in-memory @contract-shape:pure-function
  Scenario: Listing with no applications returns nothing
    Given no applications have been recorded
    When the recorded applications are listed
    Then the list is empty

  @US-04 @error @in-memory @contract-shape:pure-function
  Scenario: Database connectivity is confirmed when the secure network is up
    Given the secure network reports active connection details
    When database reachability is checked
    Then reachability is confirmed

  @US-04 @error @in-memory @contract-shape:pure-function
  Scenario: Database connectivity is reported down when logged out of the secure network
    Given the secure network reports being logged out
    When database reachability is checked
    Then reachability is reported as not connected

  @US-04 @error @in-memory @contract-shape:pure-function
  Scenario: The reachability check errors when the secure-network status fails
    Given the secure-network status command fails
    When database reachability is checked
    Then the reachability check returns an error

  @US-04 @error @in-memory @contract-shape:pure-function
  Scenario: The reachability check errors when the status command cannot run
    Given the secure-network status command cannot be run
    When database reachability is checked
    Then the reachability check returns an error
