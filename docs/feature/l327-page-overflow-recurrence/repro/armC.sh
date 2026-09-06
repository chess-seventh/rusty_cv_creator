#!/usr/bin/env bash
# Arm C - the WHOLE proposed template change against the WHOLE corpus:
#   1. \def\cvcompact{} on the senior-platform-engineer and senior-devops drivers
#   2. \setstretch{0.96} -> {0.90} in cv-main.tex's compact block
# Applied to all 25 tailored trees + the 4 untailored base variants. Every one
# must render <= 2 pages: the 10 that failed must now pass, and the 15 that
# passed plus the 4 bases must not regress.
#
# Needs render-all.sh to have run first: it reads the trees it left in
# $L327_WORK/render. Emits job ids only, never employers - see render-all.sh.
set -euo pipefail
sp="${L327_WORK:?set L327_WORK to a scratch directory}"
work=$sp/render; dstroot=$sp/armC
basev="${CV_REPO:-$HOME/src/claude-src/repos/cv}/cv-sections/variants"
pages_of() { sed -nE 's/.*Output written on .*\(([0-9]+) pages?.*/\1/p' | tail -1; }

# Every edit asserts it landed. A \setstretch or \input renamed upstream would
# make each sed a silent no-op, and this arm would then report 29 page counts
# for an UNPATCHED corpus - a wrong answer indistinguishable from a right one.
assert_in() { grep -qF -- "$1" "$2" || { echo "armC: $2 lacks $1" >&2; exit 1; }; }

patch_tree() {
  local t="$1" d
  for d in senior-platform-engineer senior-devops; do
    [ -f "$t/PivaFrancesco-$d.tex" ] || continue
    if ! grep -q '\\def\\cvcompact' "$t/PivaFrancesco-$d.tex"; then
      sed -i 's|^\\input{cv-main.tex}|\\def\\cvcompact{}\n\\input{cv-main.tex}|' \
        "$t/PivaFrancesco-$d.tex"
      assert_in '\def\cvcompact' "$t/PivaFrancesco-$d.tex"
    fi
  done
  sed -i 's|\\setstretch{0.96}|\\setstretch{0.90}|' "$t/cv-main.tex"
  assert_in '\setstretch{0.90}' "$t/cv-main.tex"
}

rm -rf "$dstroot"; mkdir -p "$dstroot"
: >"$sp/armC.tsv"
for src in "$work"/*; do
  id=$(basename "$src"); dst="$dstroot/$id"
  cp -r "$src" "$dst"; patch_tree "$dst"
  if [ "$id" = BASE ]; then
    for f in "$dst"/PivaFrancesco-*.tex; do
      v=$(basename "$f" .tex); v=${v#PivaFrancesco-}
      p=$( (cd "$dst" && tectonic --print "PivaFrancesco-$v.tex" 2>&1) | pages_of )
      printf 'BASE\t%s\t%s\n' "$v" "${p:-NONE}" >>"$sp/armC.tsv"
    done
  else
    v=$(cd "$dst/cv-sections/variants" && for x in *; do
          diff -rq "$basev/$x" "$x" >/dev/null 2>&1 || echo "$x"; done)
    [ "$(printf '%s' "$v" | grep -c . || true)" -eq 1 ] ||
      { echo "armC: job $id differs from base in [$v], expected 1" >&2; exit 1; }
    p=$( (cd "$dst" && tectonic --print "PivaFrancesco-$v.tex" 2>&1) | pages_of )
    printf '%s\t%s\t%s\n' "$id" "$v" "${p:-NONE}" >>"$sp/armC.tsv"
  fi
  # Drop the copy as soon as its page count is read. A template tree is ~42MB
  # and there are 29 of them: keeping them all costs 1.2GB for nothing, and a
  # first run of this was killed for memory pressure with three of these
  # running side by side. The page count is the only output that matters.
  rm -rf "$dst"
done
awk -F'\t' '{c[$3]++} END {for (k in c) printf "pages=%s\tcount=%s\n", k, c[k]}' "$sp/armC.tsv"
