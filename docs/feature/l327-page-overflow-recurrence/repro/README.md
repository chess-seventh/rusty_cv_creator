# Reproduction — L327 page overflow

Everything in `../rca.md` was measured with these. They read the **real**
tailored corpus of run `2026-09-06T04-06-41Z`; nothing here is a fixture, and
nothing here writes to the corpus.

**Jobs are identified by id only.** The corpus directory names carry the
employer each CV was tailored for, and this repository is public, so no script
here emits that column and no document here records the mapping. The ids prove
every number in `../rca.md` on their own.

## Prerequisites

- The box is **greensleeves** — it holds both inputs:
  - `~/.cache/rusty-cv-creator/templates/` — the 25 tailored template trees the
    renderer pinned that run. This is the evidence; treat it read-only.
  - `/var/lib/jobs-forge-automation/runs/2026-09-06T04-06-41Z.log` — the run
    log, which is what says which jobs the renderer refused.
- A checkout of the `cv` template repo at `~/src/claude-src/repos/cv`, for the
  untailored baseline each tailored tree is diffed against.
- `tectonic` and `just`, which come from that repo's `devenv`.
- `L327_WORK` pointing at a scratch directory — the scripts `rm -rf` under it,
  so do not point it at anything you want to keep. Budget **~1.2GB**: a
  template tree is about 42MB and `render-all.sh` keeps all 29, because the
  arms render from them. The arms delete each of their own copies as soon as
  its page count is read, so nothing accumulates on top of that.

## Running them

**`devenv shell` must be entered from the `cv` repo, and the scripts referred
to by absolute path.** `devenv` resolves its environment from the working
directory: run it from this directory instead and it fails with `File devenv.nix
does not exist`.

```bash
export L327_WORK="$(mktemp -d)"
export REPRO=~/src/claude-worktrees/rusty_cv_creator/L327/docs/feature/l327-page-overflow-recurrence/repro
cd ~/src/claude-src/repos/cv

# 1. Source size does not separate pass from fail. Renders nothing; needs
#    no devenv and no scratch dir.
bash "$REPRO/measure2.sh"

# 2. Ground truth: render all 25 tailored trees + the 4 base variants.
#    Writes $L327_WORK/pages.tsv and leaves the trees in $L327_WORK/render,
#    which both later scripts build on. ~5 minutes.
devenv shell -- bash "$REPRO/render-all.sh"

# 3. Arms A and B over the ten failures only: \cvcompact alone, then
#    \cvcompact + \setstretch{0.94}.
devenv shell -- bash "$REPRO/experiment.sh"

# 4. Arm C: the whole proposed template change over the whole corpus,
#    checking the ten are bought without losing any of the fifteen.
devenv shell -- bash "$REPRO/armC.sh"
```

Entering that `devenv shell` rebuilds the four `PivaFrancesco-*.pdf` in the `cv`
checkout. They are gitignored, so nothing is polluted, but they are not this
lane's output.

## What was measured (the `.tsv` files are the output of the above)

| file | columns | result |
| --- | --- | --- |
| `pages.tsv` | id, variant, pages, verdict | 10 refusals reproduce at 3 pages, 15 successes at 2, 4 bases at 2 |
| `armA.tsv` | arm, id, variant, pages | `\cvcompact` on senior-platform-engineer: 6 of the 10 fixed |
| `armB.tsv` | arm, id, variant, pages | above + `\setstretch{0.94}`: 9 of the 10 — job 12 still 3 pages |
| `armC.tsv` | id, variant, pages | above at `\setstretch{0.90}`, whole corpus: **29 of 29 at 2 pages** |

`armC.sh` patches `\setstretch` to `{0.90}` because that is the value job 12
needs. `{0.93}`, `{0.92}` and `{0.91}` were each tried by hand with the same
method and all three leave it at three pages; only `{0.90}` reaches two. The
cost of `{0.90}` is a 6% line-spacing squeeze on **every** compact CV, which is
why `../rca.md` does not recommend it as the durable answer.

Every script asserts the edit it makes actually landed, and that each tailored
tree differs from the baseline in exactly one variant directory. Both would
otherwise fail silently and produce a plausible wrong answer: a renamed
`\setstretch` would leave the corpus unpatched, and a drifted `cv` checkout
would change which variant is measured.

## Scope

These scripts prove a fix that belongs in the `cv` template repository. They
live here because this is where the failure was observed and where the previous
occurrence is written up (`../../fix-3-page-cv-overflow/rca.md`); no line of
the fix itself lands in `src/`.
