#!/usr/bin/env bash
set -euo pipefail

if ! git remote get-url upstream >/dev/null 2>&1; then
  echo "ERROR: remote 'upstream' is not configured." >&2
  exit 1
fi

echo "== Fork Hygiene Audit =="
echo

echo "[1/5] Master leak check"
master_sha="$(git rev-parse master)"
upstream_sha="$(git rev-parse upstream/master)"
if [[ "$master_sha" != "$upstream_sha" ]]; then
  echo "FAIL: local master diverges from upstream/master"
  echo "  master:         $master_sha"
  echo "  upstream/master:$upstream_sha"
else
  echo "PASS: master is clean and aligned"
fi

echo
echo "[2/5] Integration branch staleness"
if git show-ref --verify --quiet refs/heads/integration/pending-prs; then
  last_date="$(git log -1 --format=%cs integration/pending-prs)"
  echo "integration/pending-prs exists (last commit date: $last_date)"
else
  echo "integration/pending-prs not present (ok)"
fi

echo
echo "[3/5] Local branches without remote tracking"
for b in $(git for-each-ref --format='%(refname:short)' refs/heads | sort); do
  [[ "$b" == "master" ]] && continue
  if ! git rev-parse --abbrev-ref "$b@{upstream}" >/dev/null 2>&1; then
    echo "- $b (no upstream tracking)"
  fi
done

echo
echo "[4/5] Remote branches merged status"
echo "Merged into upstream/master:" 
git branch -r --merged upstream/master | sed 's/^/  /'
echo

echo "Not merged into upstream/master:" 
git branch -r --no-merged upstream/master | sed 's/^/  /'
echo

echo "[5/5] PR mapping hints"
if command -v gh >/dev/null 2>&1; then
  if gh auth status >/dev/null 2>&1; then
    echo "Open PR heads (via gh):"
    gh pr list --state open --json headRefName --jq '.[].headRefName' | sed 's/^/  /'
  else
    echo "gh installed but not authenticated; skipping PR mapping query"
  fi
else
  echo "gh not installed; manual PR mapping required"
fi

echo
echo "Audit complete."
