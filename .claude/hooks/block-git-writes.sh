#!/usr/bin/env bash
# PreToolUse hook: lets Claude read git state but never change the repository,
# index, refs, branches, stash, worktrees or config.
#
# - Bash / PowerShell: every `git` invocation in the command must be a read-only
#   subcommand (allowlist below). Unknown subcommands are blocked by default.
#   Direct references to the .git directory are blocked too.
# - Write / Edit / NotebookEdit: blocked when the target path is inside .git/.
# - EnterWorktree / ExitWorktree / Agent with worktree isolation / desktop-app
#   branch sync: blocked, since they create or merge branches and worktrees.
#
# Matching is deliberately fail-closed: a command that merely mentions a `git`
# word (e.g. `echo git commit`) may be blocked. Rephrase or run it yourself.
#
# Portable bash (3.2+), no jq/node/python required.

input=$(cat)

deny() {
    local reason="Blocked by .claude/hooks/block-git-writes.sh: $1. Claude may only read git state in this repo; ask the user to run repository-changing git operations themselves."
    reason=${reason//\\/\\\\}
    reason=${reason//\"/\\\"}
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"%s"}}\n' "$reason"
    exit 0
}

# Extract a top-level-ish JSON string field and decode the common escapes.
json_field() {
    local re="\"$1\"[[:space:]]*:[[:space:]]*\"(([^\"\\\\]|\\\\.)*)\""
    if [[ $input =~ $re ]]; then
        local v=${BASH_REMATCH[1]}
        v=${v//\\\\/$'\001'}
        v=${v//\\\"/\"}
        v=${v//\\n/$'\n'}
        v=${v//\\t/$'\t'}
        v=${v//\\r/}
        v=${v//\\\//\/}
        v=${v//$'\001'/\\}
        printf '%s' "$v"
    fi
}

strip_quotes() {
    local t=$1
    t=${t//\"/}
    t=${t//\'/}
    printf '%s' "$t"
}

in_list() {
    local needle=$1
    shift
    local x
    for x in "$@"; do [[ $needle == "$x" ]] && return 0; done
    return 1
}

touches_dot_git() {
    local p=${1//\\//}
    [[ $p =~ (^|/)\.git(/|$) ]]
}

# Decide whether `git <args...>` is read-only. Echoes a reason and returns 1 if not.
check_git() {
    local -a a=("$@")
    local n=${#a[@]} i=0 sub=""

    # Global options before the subcommand.
    while ((i < n)); do
        local t=${a[i]}
        case $t in
            -c | --config-env | --config-env=* | -c*)
                echo "git $t (inline config can run arbitrary commands)"; return 1 ;;
            --exec-path=* | --exec-path)
                echo "git $t"; return 1 ;;
            -C | --git-dir | --work-tree | --namespace | --super-prefix)
                i=$((i + 2)); continue ;;
            --version | --help | -h | --html-path | --man-path | --info-path)
                return 0 ;;
            -*)
                i=$((i + 1)); continue ;;
            *)
                sub=$t; i=$((i + 1)); break ;;
        esac
    done

    [[ -z $sub ]] && return 0 # bare `git` prints help

    local -a rest=("${a[@]:i}")
    local -a pos=() flags=()
    local r
    for r in "${rest[@]}"; do
        if [[ $r == -* ]]; then flags+=("$r"); else pos+=("$r"); fi
    done
    local first=${pos[0]-}

    case $sub in
        status | diff | log | show | blame | annotate | ls-files | ls-tree | ls-remote | \
        rev-parse | rev-list | describe | shortlog | cat-file | for-each-ref | show-ref | \
        show-branch | name-rev | merge-base | check-ignore | check-attr | check-mailmap | \
        var | help | version | whatchanged | count-objects | cherry | range-diff | \
        diff-tree | diff-files | diff-index | verify-commit | verify-tag | grep)
            if [[ $sub == grep ]]; then
                for r in "${flags[@]}"; do
                    case $r in -O* | --open-files-in-pager*) echo "git grep $r (runs a program)"; return 1 ;; esac
                done
            fi
            return 0 ;;

        branch | tag)
            local -a bad
            if [[ $sub == branch ]]; then
                bad=(-d -D --delete -m -M --move -c -C --copy -f --force -u --unset-upstream --edit-description -t --track --no-track --create-reflog --recurse-submodules)
            else
                bad=(-d --delete -a --annotate -s --sign -u -f --force -m -F --file -e --edit --create-reflog)
            fi
            for r in "${flags[@]}"; do
                [[ $r == --set-upstream-to* || $r == --local-user* || $r == --message* ]] && { echo "git $sub $r"; return 1; }
                in_list "$r" "${bad[@]}" && { echo "git $sub $r"; return 1; }
            done
            ((${#pos[@]} == 0)) && return 0
            for r in "${flags[@]}"; do in_list "$r" -l --list && return 0; done
            echo "git $sub with a name (use --list to filter)"; return 1 ;;

        remote)
            [[ -z $first || $first == show || $first == get-url ]] && return 0
            echo "git remote $first"; return 1 ;;

        config)
            for r in "${flags[@]}"; do
                case $r in
                    --unset* | --add | --replace-all | --rename-section | --remove-section | -e | --edit)
                        echo "git config $r"; return 1 ;;
                esac
            done
            [[ $first == get || $first == list ]] && return 0
            for r in "${flags[@]}"; do
                case $r in --get | --get-all | --get-regexp | --get-urlmatch | --get-color | --get-colorbool | -l | --list) return 0 ;; esac
            done
            ((${#pos[@]} == 1)) && return 0
            echo "git config write"; return 1 ;;

        stash)
            [[ $first == list || $first == show ]] && return 0
            echo "git stash ${first:-push}"; return 1 ;;

        worktree)
            [[ $first == list ]] && return 0
            echo "git worktree ${first}"; return 1 ;;

        reflog)
            case $first in expire | delete | drop | write) echo "git reflog $first"; return 1 ;; esac
            return 0 ;;

        notes)
            [[ -z $first || $first == list || $first == show ]] && return 0
            echo "git notes $first"; return 1 ;;

        submodule)
            [[ $first == status || $first == summary ]] && return 0
            echo "git submodule ${first:-update}"; return 1 ;;

        *)
            echo "git $sub"; return 1 ;;
    esac
}

check_command() {
    local cmd=$1

    if [[ $cmd =~ GIT_[A-Za-z_]+[[:space:]]*= ]] && [[ $cmd == *git* ]]; then
        deny "setting GIT_* environment variables"
    fi

    # Split into simple commands on shell/PowerShell separators.
    local seps=$'\n'
    cmd=${cmd//;/$seps}
    cmd=${cmd//|/$seps}
    cmd=${cmd//&/$seps}
    cmd=${cmd//(/$seps}
    cmd=${cmd//)/$seps}
    cmd=${cmd//\{/$seps}
    cmd=${cmd//\}/$seps}
    cmd=${cmd//\`/$seps}

    local line
    while IFS= read -r line; do
        local -a toks=()
        read -ra toks <<<"$line"
        local k=0 m=${#toks[@]}
        while ((k < m)); do
            local t
            t=$(strip_quotes "${toks[k]}")
            if touches_dot_git "$t"; then
                deny "direct access to the .git directory ($t)"
            fi
            local base=${t//\\//}
            base=${base##*/}
            base=$(printf '%s' "$base" | tr '[:upper:]' '[:lower:]')
            if [[ $base == git || $base == git.exe ]]; then
                local -a args=()
                local j
                for ((j = k + 1; j < m; j++)); do args+=("$(strip_quotes "${toks[j]}")"); done
                local why
                if ! why=$(check_git ${args[@]+"${args[@]}"}); then
                    deny "$why"
                fi
            fi
            k=$((k + 1))
        done
    done <<<"$cmd"
}

tool=$(json_field tool_name)

case $tool in
    Bash | PowerShell)
        check_command "$(json_field command)" ;;
    Write | Edit | NotebookEdit)
        path=$(json_field file_path)
        [[ -z $path ]] && path=$(json_field notebook_path)
        touches_dot_git "$path" && deny "editing files inside .git ($path)" ;;
    EnterWorktree | ExitWorktree)
        deny "$tool creates or removes git worktrees and branches" ;;
    Agent)
        [[ $(json_field isolation) == worktree ]] && deny "worktree isolation creates a git branch" ;;
    mcp__ccd_host__sync_with_base_branch)
        deny "syncing merges the base branch into this one" ;;
esac

exit 0
