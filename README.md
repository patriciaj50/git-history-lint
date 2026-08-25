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
- `subject-too-long` — subject line over 72 characters
- `subject-trailing-period` — subject ends with `.`
- `subject-not-capitalized` — subject starts with a lowercase letter
- `missing-blank-line` — body starts right after the subject, no gap
- `trailing-whitespace` — a message line has trailing spaces or tabs
- `body-line-too-long` — a body line over 72 characters

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
