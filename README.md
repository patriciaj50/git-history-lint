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

Output looks like:

```
history.txt:14: subject line should start with a capital letter [subject-not-capitalized] (a1b2c3d)
history.txt:41: missing blank line between subject and body [missing-blank-line] (4f9e21a)
history.txt:58: subject line is 91 characters, limit is 72 [subject-too-long] (7cddb02)
```

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

Only the default `git log` format is understood right now. `git log -p`
and `--stat` output will parse (their extra lines are just skipped),
but diffs and stat summaries aren't inspected. `--format` with a custom
pretty-print string isn't supported yet.

## Building

```sh
cargo build --release
```

No third-party dependencies, standard library only.

## License

MIT, see LICENSE.
