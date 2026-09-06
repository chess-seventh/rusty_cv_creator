# RCA — the tailored CV renders to three pages and the contract allows two (L327)

Investigated 2026-09-06 on **greensleeves**, the box that ran the hunt.
Reproduced against the **real** tailored corpus of run `2026-09-06T04-06-41Z`,
not against a fixture. Everything below is re-derivable: the scripts and their
output live in `repro/` beside this file.

This is the **second occurrence**. The first is
`docs/feature/fix-3-page-cv-overflow/rca.md` (2026-07-09), and reading it is
part of reading this one: the conclusion here is that the earlier fix worked
exactly as designed and was applied to one variant of four.

**Jobs are named by id only.** This repository is public; the employer each id
belongs to is not written down here. The 2026-07-09 RCA did the same, naming
"jobs 12/19/34". The ids alone prove every number below.

## What happened

`rusty_cv_creator` refused **ten** renders — `CV is 3 pages; the contract allows
at most 2`, one refusal per job, no retries. Each then reported `cv-branch open
failed` and `FAILED - isolated, continuing`, so the run exited 0 and the ten
jobs were lost silently. The Claude call for each had already been spent.

The run log accounts for itself exactly: **25 builds = 15 `Page-count contract
met` + 10 refusals.** (A `grep` for the page-count string returns twenty lines,
because each refusal prints twice — the logger record and the propagated
`Error:` line. Twenty is a line count, not a loss count.)

The ten, from the run log:

| id | variant |
| --- | --- |
| 7 | senior-platform-engineer |
| 9 | engineering-manager |
| 10 | engineering-manager |
| 11 | senior-platform-engineer |
| 12 | engineering-manager |
| 23 | engineering-manager |
| 25 | senior-platform-engineer |
| 36 | senior-platform-engineer |
| 37 | senior-platform-engineer |
| 49 | senior-platform-engineer |

That list is not typed: it is every distinct branch in
`cv-branch: render failed for …`, which appears exactly ten times.

## The corpus survived, so nothing here is synthetic

The per-job branches never reached the `cv` remote (D-96: the deploy key is
read-only) and the run's temporary workspace is gone. But the renderer caches
every pinned template tree under
`~/.cache/rusty-cv-creator/templates/<workspace>@<branch>`, and **all 25 trees
from that run are still there**, including all ten failures.

Re-rendered outside the pipeline, with `tectonic --print`, one copy per tree:

- **10/10 refusals reproduce at exactly 3 pages.**
- **15/15 successes reproduce at exactly 2 pages.**
- **4/4 untailored base variants render at 2 pages.**

Zero Claude calls were spent to obtain any of this. `repro/pages.tsv` is that
run.

## Root cause

### It is not the length of the tailored content

Byte and line deltas against the untailored base do not separate the two
groups. Job 20 **passes** at `aboutme +127 / experience +16 / skills +15` bytes.
Job 36 **fails** while being *smaller than base* in two of the three sections
(`+130 / -69 / -42`). By total source lines the two groups cross over as well:
job 23 fails at 709, job 20 passes at 719. `repro/measure2.sh` prints all of it.

### It is not the variant selection

`senior-platform-engineer` and `engineering-manager` each appear on both sides
of the split. All three `senior-sre` jobs passed. Corpus composition: 11
`senior-platform-engineer`, 11 `engineering-manager`, 3 `senior-sre`.

### It is the template's headroom, and the mechanism is already written down

`cv-main.tex` says it in its own comment:

> the experience table is one unbreakable box, so when page 2 overflows by even
> 1% the whole block jumps to page 3

and the 2026-07-09 RCA measured the slack:

> the template was tuned to exactly 2 pages; tailored content grew +0.6-1.1%
> and tipped it. jobs-forge's +15% char guard is a placebo (real slack < 1%).

So the discriminator is **rendered height against a quantized block**, not
source size — which is exactly why a section can lose bytes and still tip the
page, and why a longer job can fit while a shorter one does not.

### Why now and not before

The 2026-07-09 fix (`cv` commit `4e84a47`) bought headroom by giving the
overflowing driver `\def\cvcompact{}` plus `\setstretch{0.96}`. It was applied
to **`senior-sre` only** — the variant that was overflowing that day.
`engineering-manager` already had `\cvcompact`. **`senior-platform-engineer`
and `senior-devops` have never had it**, and `git log -S cvcompact` on those two
drivers returns nothing at all.

That is the measured half. Two things then changed on 2026-09-06, and they are
of different evidential weight — said here rather than blurred together:

1. **Measured.** `engineering-manager`'s tailored content has now grown past
   even the *compact* headroom: **4 of its 11 overflowed** despite `\cvcompact`
   and `\setstretch{0.96}`. That variant was treated and still failed.
2. **Inferred, not measured.** 11 of the 25 jobs took the untreated
   `senior-platform-engineer` variant and **6 of those 11 tipped**. It is
   plausible that this was the first run to tailor that variant at volume — L326
   had just restored the hunt — but the box keeps only two run logs and the
   older one rendered nothing, so there is no comparative baseline here to prove
   it. What *is* proven is that the variant was never given the headroom, and
   that six of its eleven jobs needed it.

## Which repository owns the fix — `cv` (`_cv_template`), not this one

**`rusty_cv_creator`'s page gate is behaving exactly as designed.** The
2026-07-09 RCA states its purpose in this repo's own words: *"This repo's
assertion is the second layer: catch ANY future overflow at render time
regardless of variant/content."* It caught ten. It is the detector, not the
defect, and no change to `src/` makes a 3-page document into a 2-page one.

Measured, on the real corpus:

| change (all in `cv`) | of the ten |
| --- | --- |
| `\def\cvcompact{}` on the `senior-platform-engineer` driver | **6 fixed** — all six SPE jobs render 2 pages |
| the above + `\setstretch{0.96}` -> `{0.94}` | **9 fixed** — job 12 still renders 3 |
| the above + `\setstretch` -> `{0.90}` | **10 fixed** — job 12 needs a 6% line-spacing squeeze |

The first row is the direct precedent of `4e84a47` applied to the variant that
was left out, and it is uncontroversial. The last row is not: `\setstretch` in
the compact block applies to **every** compact CV, so buying job 12's fit costs
a visible typographic squeeze on all of them. `{0.93}`, `{0.92}` and `{0.91}`
all leave job 12 at three pages; `{0.90}` is its actual threshold.

The last row was then swept over the **whole** corpus — all 25 tailored trees
plus the 4 untailored base variants, one render each — to check that the ten
are bought without losing any of the fifteen:

    pages=2  count=29

**29 of 29 at two pages.** No regression among the fifteen that already passed,
none on the four base variants, and all ten failures fixed. So a template-only
fix is viable; what it costs is the squeeze, not correctness.

**That is the finding worth carrying over from the numbers: layout tightening
is a treadmill.** It has now been reached for twice, each time for the variant
that happened to overflow, and each round buys less. The tailored content is
not bounded by anything — jobs-forge's character guard was measured a placebo
in 2026-07 and nothing replaced it.

## Must-prove 4 — can the page check cost nothing when it fails?

**Not in its present position, and the honest answer is that it cannot be moved
before the call.** The page count is a property of the *rendered* document, and
the document is what the Claude call produces. Nothing knowable before the call
predicts it: this RCA's own measurement is that source size does not.

What *can* be removed is the waste after it. Two routes, neither of them this
lane's to take alone:

1. **Bound the content at tailor time** (`jobs-forge` / `career-ops`): give the
   tailoring step a real budget rather than a character guard already measured
   to be a placebo. This is the only route that stops the treadmill.
2. **Escalate instead of discard** (`cv` then `rusty_cv_creator`): the renderer
   already passes a Justfile variable override (`page_count_probe`,
   `tectonic=tectonic --print`). If the template exposed the compact knob the
   same way, the renderer could retry once, tighter, on a contract violation.
   Across the run's 25 builds a render took a **median of 4 seconds and 12 at
   worst**, so the retry costs seconds and **no** Claude call. The template
   change must land first; a retry against a knob that does not exist is
   nothing.

Today neither happens: the renderer returns `Err`, `cv-branch` deletes the
working tree, the job is never recorded as tailored, and it is re-tailored at a
fresh Claude call on the next run.

## What is explicitly not proposed

**Raising `max_pages` to 3.** The contract was set deliberately after a
three-page CV shipped (L18, 2026-07-10). Widening it is Franci's decision with
the loss written down, not a route to green.

## Reproduction

`repro/` holds the scripts, their prerequisites and the exact output quoted
above. The corpus is `~/.cache/rusty-cv-creator/templates/` on greensleeves; it
is evidence, and nothing in this investigation wrote to it.
