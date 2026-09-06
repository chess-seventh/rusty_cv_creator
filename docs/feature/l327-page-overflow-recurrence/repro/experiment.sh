#!/usr/bin/env bash
# Diagnosis experiment over the ten refusals only. Works on copies under
# $L327_WORK. This script writes nothing outside it: the cv checkout is only
# READ, and the template cache is never touched. (Entering the cv repo devenv,
# which is how you get tectonic, does rebuild that repo's gitignored PDFs -
# that is the shell, not this script.)
#   A: \cvcompact on whichever driver lacks it (the 4e84a47 senior-sre precedent).
#   B: A, plus \setstretch 0.96 -> 0.94 in the compact block.
# Needs render-all.sh to have run first: it reads the trees it left in
# $L327_WORK/render. Emits job ids only, never employers - see render-all.sh.
set -euo pipefail
sp="${L327_WORK:?set L327_WORK to a scratch directory}"
work=$sp/render
basev="${CV_REPO:-$HOME/src/claude-src/repos/cv}/cv-sections/variants"
fails="7 9 10 11 12 23 25 36 37 49"
pages_of() { sed -nE 's/.*Output written on .*\(([0-9]+) pages?.*/\1/p' | tail -1; }

# Every edit below asserts it landed. A \setstretch or \input renamed upstream
# would make each sed a silent no-op, and the arm would then report page counts
# for an UNPATCHED corpus - a wrong answer that looks exactly like a right one.
assert_in() { grep -qF -- "$1" "$2" || { echo "experiment: $2 lacks $1" >&2; exit 1; }; }

run_arm() {
  local arm="$1" id v dst drv p
  for id in $fails; do
    dst="$sp/arm$arm/$id"
    rm -rf "$dst"; mkdir -p "$sp/arm$arm"; cp -r "$work/$id" "$dst"
    v=$(cd "$dst/cv-sections/variants" && for x in *; do
          diff -rq "$basev/$x" "$x" >/dev/null 2>&1 || echo "$x"; done)
    [ "$(printf '%s' "$v" | grep -c . || true)" -eq 1 ] ||
      { echo "experiment: job $id differs from base in [$v], expected 1" >&2; exit 1; }
    drv="$dst/PivaFrancesco-$v.tex"
    if ! grep -q '\\def\\cvcompact' "$drv"; then
      sed -i 's|^\\input{cv-main.tex}|\\def\\cvcompact{}\n\\input{cv-main.tex}|' "$drv"
      assert_in '\def\cvcompact' "$drv"
    fi
    if [ "$arm" = B ]; then
      sed -i 's|\\setstretch{0.96}|\\setstretch{0.94}|' "$dst/cv-main.tex"
      assert_in '\setstretch{0.94}' "$dst/cv-main.tex"
    fi
    p=$( (cd "$dst" && tectonic --print "PivaFrancesco-$v.tex" 2>&1) | pages_of )
    printf 'arm%s\t%s\t%s\t%s\n' "$arm" "$id" "$v" "${p:-NONE}"
    # Drop the copy once its page count is read - see the note in armC.sh.
    rm -rf "$dst"
  done
}
run_arm A | tee "$sp/armA.tsv"
run_arm B | tee "$sp/armB.tsv"
