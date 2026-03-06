#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/check_fork_graph.sh [--branch <name>] [--stack-parent <name>] [--allow-path <prefix>]...

Checks:
- local master equals upstream/master
- branch base is upstream/master (independent) OR explicit stack parent is provided
- optional scope check: all changed files under allowed prefixes
USAGE
}

branch="$(git branch --show-current)"
stack_parent=""
allow_paths=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --branch)
      [[ $# -ge 2 ]] || { usage; exit 2; }
      branch="$2"
      shift 2
      ;;
    --stack-parent)
      [[ $# -ge 2 ]] || { usage; exit 2; }
      stack_parent="$2"
      shift 2
      ;;
    --allow-path)
      [[ $# -ge 2 ]] || { usage; exit 2; }
      allow_paths+=("$2")
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if ! git remote get-url upstream >/dev/null 2>&1; then
  echo "ERROR: remote 'upstream' is not configured." >&2
  exit 1
fi

if ! git show-ref --verify --quiet refs/heads/master; then
  echo "ERROR: local 'master' branch not found." >&2
  exit 1
fi

if ! git show-ref --verify --quiet refs/remotes/upstream/master; then
  echo "ERROR: upstream/master not found locally. Run: git fetch upstream" >&2
  exit 1
fi

master_sha="$(git rev-parse master)"
upstream_sha="$(git rev-parse upstream/master)"

if [[ "$master_sha" != "$upstream_sha" ]]; then
  echo "ERROR: local master is not aligned with upstream/master." >&2
  echo "  master:         $master_sha" >&2
  echo "  upstream/master:$upstream_sha" >&2
  exit 1
fi

echo "OK: master is aligned with upstream/master"

if [[ "$branch" == "master" ]]; then
  echo "INFO: checking master only; branch-specific checks skipped."
  exit 0
fi

if ! git show-ref --verify --quiet "refs/heads/$branch"; then
  echo "ERROR: branch '$branch' not found locally." >&2
  exit 1
fi

base_sha="$(git merge-base "$branch" upstream/master)"

if [[ "$base_sha" != "$upstream_sha" ]]; then
  if [[ -z "$stack_parent" ]]; then
    echo "ERROR: branch '$branch' is not based on latest upstream/master and no --stack-parent was provided." >&2
    echo "  merge-base(branch, upstream/master): $base_sha" >&2
    echo "  upstream/master:                     $upstream_sha" >&2
    exit 1
  fi

  if ! git show-ref --verify --quiet "refs/heads/$stack_parent"; then
    echo "ERROR: --stack-parent '$stack_parent' not found locally." >&2
    exit 1
  fi

  parent_sha="$(git rev-parse "$stack_parent")"
  if ! git merge-base --is-ancestor "$parent_sha" "$branch"; then
    echo "ERROR: stack parent '$stack_parent' is not an ancestor of '$branch'." >&2
    exit 1
  fi

  echo "OK: stacked branch dependency explicitly declared: $branch -> $stack_parent"
else
  if [[ -n "$stack_parent" ]]; then
    echo "WARN: --stack-parent was provided but branch is already independent on upstream/master."
  fi
  echo "OK: branch '$branch' is based on latest upstream/master"
fi

if (( ${#allow_paths[@]} > 0 )); then
  mapfile -t changed_files < <(git diff --name-only upstream/master..."$branch")

  outside=()
  for f in "${changed_files[@]}"; do
    matched=0
    for p in "${allow_paths[@]}"; do
      if [[ "$f" == "$p"* ]]; then
        matched=1
        break
      fi
    done
    if [[ $matched -eq 0 ]]; then
      outside+=("$f")
    fi
  done

  if (( ${#outside[@]} > 0 )); then
    echo "ERROR: branch scope contains files outside --allow-path constraints:" >&2
    printf '  %s\n' "${outside[@]}" >&2
    exit 1
  fi

  echo "OK: branch file scope matches allowed prefixes"
fi

echo "PASS: fork graph checks completed"
