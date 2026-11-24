#!/usr/bin/env bash
# Which green verify run already holds a commit's tree, so a tag does
# not run the whole matrix (about 85 minutes, 30 jobs) a second time.
#
#   verified.sh [REF]     # REF defaults to HEAD
#
# Prints the run's id and exits 0 when one of the recent verify runs
#   - passed on a commit with exactly REF's tree, or
#   - failed on `sizes` alone, on a commit whose tree differs from REF's
#     only in the size record, and REF's record is that run's own
#     `sizes-record` byte for byte (the record moves with every content
#     change, so a release's last round is often only that commit).
# Exits 1 when no run qualifies; the caller then runs verify itself.
#
# Needs `gh` with read access to the repository's actions and contents
# (GH_TOKEN), and REF's commit pushed. Used by release.yml's gate and
# before `git tag -s` (docs/06-cicd/03-release-process.md).
set -euo pipefail

ref="${1:-HEAD}"
sha="$(git rev-parse "${ref}^{commit}")"
tree="$(git rev-parse "${ref}^{tree}")"
repo="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"
record=(docs/05-testing/sizes.json docs/05-testing/SIZES.md)

tree_of() { gh api "repos/${repo}/commits/$1" --jq .commit.tree.sha; }

# The runs on any branch, newest first; a completed run is all that is read.
runs="$(gh api "repos/${repo}/actions/workflows/verify.yml/runs?status=completed&per_page=40" \
  --jq '.workflow_runs[] | "\(.id) \(.head_sha) \(.conclusion)"')"

while read -r id head conclusion; do
  [ -n "${id}" ] || continue
  if [ "${conclusion}" = success ] && [ "$(tree_of "${head}")" = "${tree}" ]; then
    echo "${id}"
    exit 0
  fi
  [ "${conclusion}" = failure ] || continue
  failed="$(gh api "repos/${repo}/actions/runs/${id}/jobs?per_page=100" \
    --jq '[.jobs[] | select(.conclusion == "failure") | .name] | join(",")')"
  [ "${failed}" = sizes ] || continue
  changed="$(gh api "repos/${repo}/compare/${head}...${sha}" --jq '[.files[].filename] | sort | join(",")')"
  [ "${changed}" = "$(printf '%s\n' "${record[@]}" | sort | paste -sd, -)" ] || continue
  dir="$(mktemp -d)"
  if gh run download "${id}" --repo "${repo}" --name sizes-record --dir "${dir}" >/dev/null 2>&1 &&
    cmp -s "${dir}/sizes.json" <(git show "${sha}:${record[0]}") &&
    cmp -s "${dir}/SIZES.md" <(git show "${sha}:${record[1]}"); then
    rm -rf "${dir}"
    echo "${id}"
    exit 0
  fi
  rm -rf "${dir}"
done <<< "${runs}"

echo "no verify run holds ${sha}'s tree" >&2
exit 1
