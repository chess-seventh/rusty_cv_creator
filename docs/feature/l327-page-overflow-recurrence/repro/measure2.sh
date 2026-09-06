#!/usr/bin/env bash
# Per-section byte deltas AND total source lines of the tailored content vs the
# untailored base, split by the verdict the run log recorded. Shows that source
# size does NOT separate the two groups: a passing job can be larger in every
# section than a failing one, and the line totals cross over too.
#
# Needs no scratch directory and renders nothing. Emits job ids only, never
# employers - see render-all.sh for why.
set -euo pipefail
base="${CV_REPO:-$HOME/src/claude-src/repos/cv}/cv-sections/variants"
cache="${CV_CACHE:-$HOME/.cache/rusty-cv-creator/templates}"
log="${JF_RUNS:-/var/lib/jobs-forge-automation/runs}"/2026-09-06T04-06-41Z.log
printf '%-5s %-26s %8s %8s %8s %6s %6s %s\n' \
  id variant d_about d_exp d_skills b_ln t_ln verdict
for d in "$cache"/*; do
  n=$(basename "$d"); ref=${n#*@}
  branch=$(printf '%s' "$ref" | sed 's/_/-/g; s/^cv-/cv\//')
  id=$(printf '%s' "$ref" | cut -d_ -f2)
  v=$(cd "$d/cv-sections/variants" && for x in *; do
        diff -rq "$base/$x" "$x" >/dev/null 2>&1 || echo "$x"; done)
  [ "$(printf '%s' "$v" | grep -c . || true)" -eq 1 ] ||
    { echo "measure2: job $id differs from base in [$v], expected 1" >&2; exit 1; }
  out=""; bl=0; tl=0
  for f in aboutme.tex experience.tex skills.tex; do
    b=$(wc -c < "$base/$v/$f"); t=$(wc -c < "$d/cv-sections/variants/$v/$f")
    bl=$((bl + $(wc -l < "$base/$v/$f")))
    tl=$((tl + $(wc -l < "$d/cv-sections/variants/$v/$f")))
    out="$out $(printf '%+8d' $((t - b)))"
  done
  if grep -q "render failed for $branch" "$log"; then verdict=FAIL-3p; else verdict=ok-2p; fi
  printf '%-5s %-26s%s %6s %6s %s\n' "$id" "$v" "$out" "$bl" "$tl" "$verdict"
done | sort -k8,8 -k1,1n
