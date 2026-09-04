#!/usr/bin/env bash
#
# Allow only known identities in the history of this repository.
#
# Every author, committer and Co-Authored-By email in the range must appear in
# the allowlist. Anything else fails: a work account, a personal address nobody
# registered, or an AI assistant. An allowlist is used rather than a list of
# banned domains because a banned list only catches what someone remembered to
# write down, and the exact spelling of a work domain is easy to get wrong.
#
# Usage:
#   check-commit-authorship.sh [<range>]
#
# <range> defaults to origin/dev..HEAD. In CI pass the PR range explicitly,
# e.g. "$BASE_SHA..$HEAD_SHA".
#
# Configuration (environment):
#   ALLOWED_EMAILS_FILE   path to the allowlist, default
#                         .github/allowed-commit-emails.txt next to this script
#   BLOCK_ALL_CO_AUTHORS  "true" to reject every Co-Authored-By trailer, even an
#                         allowlisted one. Default "false".

set -euo pipefail

RANGE="${1:-origin/dev..HEAD}"
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ALLOWED_EMAILS_FILE="${ALLOWED_EMAILS_FILE:-$SCRIPT_DIR/../allowed-commit-emails.txt}"
BLOCK_ALL_CO_AUTHORS="${BLOCK_ALL_CO_AUTHORS:-false}"

# Attribution footers some AI coding tools append to the commit body (a
# "Generated with [<tool>](<link>)" style line). This carries no email, so the
# allowlist cannot catch it — matched generically, without naming any tool.
AI_BODY_PATTERN='Generated (with|using|by) \[.*\]\(https?://'

violations=0

fail() {
    printf '  %s\n' "$1" >&2
    violations=$((violations + 1))
}

if [ ! -f "$ALLOWED_EMAILS_FILE" ]; then
    echo "check-commit-authorship: allowlist not found at $ALLOWED_EMAILS_FILE" >&2
    exit 2
fi

# Load the allowlist: exact addresses, plus "@domain" entries meaning any
# address on that domain. Comments and blank lines are ignored.
allowed_exact=""
allowed_domains=""
while IFS= read -r line || [ -n "$line" ]; do
    line=${line%%#*}
    line=$(printf '%s' "$line" | tr -d '[:space:]' | tr '[:upper:]' '[:lower:]')
    [ -z "$line" ] && continue
    case "$line" in
        @*) allowed_domains="$allowed_domains ${line#@}" ;;
        *)  allowed_exact="$allowed_exact $line" ;;
    esac
done < "$ALLOWED_EMAILS_FILE"

if [ -z "$allowed_exact$allowed_domains" ]; then
    echo "check-commit-authorship: allowlist $ALLOWED_EMAILS_FILE is empty" >&2
    exit 2
fi

is_allowed() {
    local email domain
    email=$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')
    [ -z "$email" ] && return 1
    for a in $allowed_exact; do
        [ "$email" = "$a" ] && return 0
    done
    for domain in $allowed_domains; do
        case "$email" in
            *"@$domain") return 0 ;;
        esac
    done
    return 1
}

if ! git rev-parse --verify --quiet "${RANGE%%..*}" >/dev/null 2>&1; then
    echo "check-commit-authorship: cannot resolve range '$RANGE'" >&2
    exit 2
fi

commits=$(git log --no-merges --format=%H "$RANGE")

if [ -z "$commits" ]; then
    echo "check-commit-authorship: no commits in range '$RANGE', nothing to check"
    exit 0
fi

echo "check-commit-authorship: scanning $(printf '%s\n' "$commits" | wc -l | tr -d ' ') commit(s) in '$RANGE'"
echo "  allowlist:            $ALLOWED_EMAILS_FILE"
echo "  allowed identities:   $(printf '%s' "$allowed_exact" | wc -w | tr -d ' ') address(es), $(printf '%s' "$allowed_domains" | wc -w | tr -d ' ') domain(s)"
echo "  block all co-authors: $BLOCK_ALL_CO_AUTHORS"
echo

for sha in $commits; do
    subject=$(git log -1 --format=%s "$sha")
    body=$(git log -1 --format=%B "$sha")

    commit_reported=0
    report_commit() {
        if [ "$commit_reported" -eq 0 ]; then
            printf '%s %s\n' "${sha:0:9}" "$subject" >&2
            commit_reported=1
        fi
    }

    for role in author committer; do
        case "$role" in
            author)    email=$(git log -1 --format=%ae "$sha"); name=$(git log -1 --format=%an "$sha") ;;
            committer) email=$(git log -1 --format=%ce "$sha"); name=$(git log -1 --format=%cn "$sha") ;;
        esac
        if ! is_allowed "$email"; then
            report_commit
            fail "$role is not on the allowlist: $name <$email>"
        fi
    done

    while IFS= read -r trailer; do
        [ -z "$trailer" ] && continue
        value=${trailer#*:}
        value=$(printf '%s' "$value" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
        trailer_email=$(printf '%s' "$value" | sed -n 's/.*<\(.*\)>.*/\1/p')

        if [ "$BLOCK_ALL_CO_AUTHORS" = "true" ]; then
            report_commit
            fail "Co-Authored-By trailers are not allowed here: $value"
        elif ! is_allowed "$trailer_email"; then
            report_commit
            fail "co-author is not on the allowlist: $value"
        fi
    done <<EOF
$(printf '%s\n' "$body" | grep -iE '^[[:space:]]*Co-Authored-By:' || true)
EOF

    while IFS= read -r line; do
        [ -z "$line" ] && continue
        report_commit
        fail "AI attribution line in commit body: $(printf '%s' "$line" | sed 's/^[[:space:]]*//')"
    done <<EOF
$(printf '%s\n' "$body" | grep -iE "$AI_BODY_PATTERN" || true)
EOF
done

if [ "$violations" -gt 0 ]; then
    cat >&2 <<MSG

check-commit-authorship: FAILED

Commits in this pull request carry an identity that is not on the allowlist.
See README.md ("Contributing") for the rules.

If the identity is an AI assistant, strip the attribution instead of allowing
it. Turn it off at the source, in your assistant's own settings, rather than
editing commits after the fact.

If a commit was made from the wrong account, fix the identity and rewrite:

  git config user.email "you@personal.example"
  git rebase <base> --exec 'git commit --amend --no-edit --reset-author'

If this is a genuine new contributor, add their address to
$ALLOWED_EMAILS_FILE in this pull request.
MSG
    exit 1
fi

echo "check-commit-authorship: OK, every identity is on the allowlist"
