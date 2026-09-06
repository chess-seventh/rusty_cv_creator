#!/usr/bin/env bash
# Ground-truth page counts for the 25 tailored trees of run 2026-09-06T04-06-41Z,
# plus the 4 untailored base variants. Renders a COPY; the cache is evidence.
#
# Emits the job ID only, never the employer. The cache directory names carry the
# company each job was tailored for, and this repository is PUBLIC - the
# 2026-07-09 RCA named its jobs "12/19/34" for the same reason. The id proves
# every number in ../rca.md on its own; the id-to-employer mapping stays on the
# box that ran the hunt.
set -euo pipefail
sp="${L327_WORK:?set L327_WORK to a scratch directory}"
cache="${CV_CACHE:-$HOME/.cache/rusty-cv-creator/templates}"
base="${CV_REPO:-$HOME/src/claude-src/repos/cv}"
work="$sp/render"
log="${JF_RUNS:-/var/lib/jobs-forge-automation/runs}"/2026-09-06T04-06-41Z.log
out="$sp/pages.tsv"
rm -rf "$work"; mkdir -p "$work"; : >"$out"

pages_of() { sed -nE 's/.*Output written on .*\(([0-9]+) pages?.*/\1/p' | tail -1; }

# The variant a tailored tree carries = the one variant directory differing from
# the untailored baseline. ASSERTED to be exactly one: taking the first match
# would silently answer for the wrong variant if the cv checkout ever drifted
# from what the run pinned, and the wrong answer would still look plausible.
variant_of() {
  local d="$1" hits n
  hits=$(cd "$d/cv-sections/variants" && for x in *; do
           diff -rq "$base/cv-sections/variants/$x" "$x" >/dev/null 2>&1 || echo "$x"
         done)
  n=$(printf '%s' "$hits" | grep -c . || true)
  [ "$n" -eq 1 ] || {
    echo "render-all: $d differs from base in $n variants [$hits], expected 1" >&2
    return 1
  }
  printf '%s' "$hits"
}

# 1. the four base variants, untailored
cp -r "$base" "$work/BASE"
for f in "$work"/BASE/PivaFrancesco-*.tex; do
  v=$(basename "$f" .tex); v=${v#PivaFrancesco-}
  p=$( (cd "$work/BASE" && tectonic --print "PivaFrancesco-$v.tex" 2>&1) | pages_of )
  printf 'BASE\t%s\t%s\tbase\n' "$v" "${p:-NONE}" >>"$out"
done

# 2. the 25 tailored trees
for d in "$cache"/*; do
  n=$(basename "$d"); ref=${n#*@}
  id=$(printf '%s' "$ref" | cut -d_ -f2)
  branch=$(printf '%s' "$ref" | sed 's/_/-/g; s/^cv-/cv\//')
  v=$(variant_of "$d")
  cp -r "$d" "$work/$id"
  p=$( (cd "$work/$id" && tectonic --print "PivaFrancesco-$v.tex" 2>&1) | pages_of )
  if grep -q "render failed for $branch" "$log"; then r=FAIL-3p; else r=ok-2p; fi
  printf '%s\t%s\t%s\t%s\n' "$id" "$v" "${p:-NONE}" "$r" >>"$out"
done
sort -k4,4 -k1,1n "$out"
