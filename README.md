# git-history-lint

A linter for `git log` output. It reads the plain-text history you'd
normally scroll past and reports commit message problems with the exact
line number they're on, so you can jump straight to the offending commit.

## Why

Most commit message conventions (a short imperative subject, a blank
line before the body, no trailing punctuation) exist because someone
read them somewhere once and forgot within a week. Nobody checks them
by hand, and by the time a bad message lands it's permanent. This is a
small tool to catch the obvious ones before a merge, or to audit a
branch's history in bulk.

It works on the text `git log` prints, not the object database, so
there's nothing to link against and no repository format to keep up
with.

## Usage

Pipe a log straight in:

```sh
git log | git-history-lint -
```

Or lint a saved log file:

```sh
git log > history.txt
git-history-lint history.txt
```

By default it expects the layout plain `git log` prints (also what you get
from `--format=short/full/fuller`, which only add or drop header lines).
For `git log --oneline`, pass `--format=oneline`:

```sh
git log --oneline | git-history-lint --format=oneline -
```

`--format` can go before or after the path argument.

Output looks like:

```
history.txt:14: subject line should start with a capital letter [subject-not-capitalized] (a1b2c3d)
history.txt:41: missing blank line between subject and body [missing-blank-line] (4f9e21a)
history.txt:58: subject line is 91 characters, limit is 72 [subject-too-long] (7cddb02)
3 findings total: subject-too-long (1), subject-not-capitalized (1), missing-blank-line (1)
```

The summary line is only printed when something was flagged, and tallies
findings by rule in the order listed under "What it checks", not the
order they appeared in the log.

Exit status is 1 if anything was flagged, 0 if the history is clean, and
2 if the input couldn't be read at all.

## What it checks

- `empty-subject` — a commit with no message
- `subject-too-long` — subject line over the length limit (72 by default)
- `subject-trailing-period` — subject ends with `.`
- `subject-not-capitalized` — subject starts with a lowercase letter
- `subject-not-imperative` — subject starts with a past-tense or gerund
  verb ("Fixed", "Adding") instead of an imperative one ("Fix", "Add")
- `missing-blank-line` — body starts right after the subject, no gap
- `trailing-whitespace` — a message line has trailing spaces or tabs
- `body-line-too-long` — a body line over the length limit (72 by default)

Any rule can be disabled and the length limit adjusted, see Config below.

## Config

Drop a `.git-history-lint.conf` file in the directory you run the linter
from to change the line length limit or turn off individual rules:

```
max-line-len = 100
disable = subject-trailing-period
disable = trailing-whitespace
```

Blank lines and `#` comments are ignored. `disable` can appear more than
once. An unknown key, a bad `max-line-len` value, or an unrecognized rule
id in `disable` is an error, not a silent no-op. No config file means the
defaults apply: a 72 character limit and every rule enabled.

## Limitations

`git log -p` and `--stat` output will parse (their extra lines are just
skipped), but diffs and stat summaries aren't inspected.

`--format=oneline` is the only alternate layout supported. Since it has
no message body, the body-only rules (`missing-blank-line`,
`body-line-too-long`) never fire on it. Custom `--pretty=format:` strings
aren't supported: there's no way to know their layout in general, so the
parser only understands the handful of named formats above.

## Pre-push hook

Reject a push that would introduce a bad commit message by wiring the
linter into `.git/hooks/pre-push`. Git feeds this hook one line per ref
on stdin (`<local ref> <local sha1> <remote ref> <remote sha1>`), so the
hook can restrict linting to just the commits about to leave the
machine instead of the whole history:

```sh
#!/bin/sh
# .git/hooks/pre-push

zero="0000000000000000000000000000000000000000"

while read local_ref local_sha remote_ref remote_sha; do
    if [ "$local_sha" = "$zero" ]; then
        continue # a branch delete, nothing to lint
    fi
    if [ "$remote_sha" = "$zero" ]; then
        range="$local_sha" # new branch, lint its whole history
    else
        range="$remote_sha..$local_sha"
    fi
    if ! git log "$range" | git-history-lint -; then
        echo "pre-push: fix the commit message issues above, or use --no-verify" >&2
        exit 1
    fi
done
```

Make it executable with `chmod +x .git/hooks/pre-push`, and make sure
`git-history-lint` itself is on `PATH` (`cargo install --path .` puts it
in `~/.cargo/bin`). Hooks aren't committed to the repository, so anyone
who wants this needs to add it to their own `.git/hooks/`.

## Building

```sh
cargo build --release
```

No third-party dependencies, standard library only.

## License

MIT, see LICENSE.
