//! Parses the plain-text output of `git log` (not `-p` or `--stat`) into
//! individual commits, keeping track of which source line each piece of
//! the message came from. That's what lets the linter point back at an
//! exact line instead of just naming a commit.

#[derive(Debug, Clone)]
pub struct Line {
    pub number: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub header_line: usize,
    pub subject: Option<Line>,
    pub body: Vec<Line>,
}

/// Which `git log` layout the input is in. Formats that already put each
/// commit's message on its own blank-line-terminated, four-space-indented
/// block (`medium`, `short`, `full`, `fuller`) all parse the same way and
/// don't need a variant here; only layouts with a genuinely different
/// shape do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// `commit <hash>` line, header lines, a blank line, then the
    /// four-space-indented message. This is what plain `git log` prints,
    /// and also `--format=short/full/fuller`.
    Medium,
    /// `git log --oneline`: one commit per line, `<hash> <subject>`, no
    /// body and no separate header block.
    Oneline,
}

/// Parses `input` into a list of commits, assuming the default `git log`
/// layout. Use `parse_with_format` for other formats.
pub fn parse(input: &str) -> Vec<Commit> {
    parse_with_format(input, Format::Medium)
}

pub fn parse_with_format(input: &str, format: Format) -> Vec<Commit> {
    match format {
        Format::Medium => parse_medium(input),
        Format::Oneline => parse_oneline(input),
    }
}

/// Anything before the first `commit ` line is ignored, so callers can
/// feed in a log with a leading banner or pager artifacts without
/// pre-cleaning it.
fn parse_medium(input: &str) -> Vec<Commit> {
    let lines: Vec<&str> = input.lines().collect();
    let mut commits = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if !lines[i].starts_with("commit ") {
            i += 1;
            continue;
        }

        let hash = lines[i]["commit ".len()..].trim().to_string();
        let header_line = i + 1;
        i += 1;

        // Header lines (Author, Date, Merge, GPG status, ...) run until
        // the blank line that separates them from the message body.
        while i < lines.len() && !lines[i].is_empty() && !lines[i].starts_with("commit ") {
            i += 1;
        }
        if i < lines.len() && lines[i].is_empty() {
            i += 1;
        }

        // The message region is every blank line or four-space-indented
        // line that follows, matching how `git log` prints the message.
        let mut message: Vec<Line> = Vec::new();
        while i < lines.len() {
            let raw = lines[i];
            if raw.is_empty() {
                message.push(Line { number: i + 1, text: String::new() });
                i += 1;
            } else if let Some(text) = raw.strip_prefix("    ") {
                message.push(Line { number: i + 1, text: text.to_string() });
                i += 1;
            } else {
                break;
            }
        }
        while message.last().map_or(false, |l| l.text.is_empty()) {
            message.pop();
        }

        let mut iter = message.into_iter();
        let subject = loop {
            match iter.next() {
                Some(l) if l.text.is_empty() => continue,
                other => break other,
            }
        };
        let body: Vec<Line> = iter.collect();

        commits.push(Commit { hash, header_line, subject, body });
    }

    commits
}

/// Each non-empty line is one commit: the hash, a single space, then the
/// subject. A line with no space (or nothing after it) is a commit with
/// an empty message, matching how the medium parser treats an all-blank
/// message.
fn parse_oneline(input: &str) -> Vec<Commit> {
    let mut commits = Vec::new();

    for (i, raw) in input.lines().enumerate() {
        if raw.is_empty() {
            continue;
        }
        let line_number = i + 1;
        let (hash, rest) = match raw.split_once(' ') {
            Some((hash, rest)) => (hash, Some(rest)),
            None => (raw, None),
        };
        let subject = match rest {
            Some(text) if !text.is_empty() => Some(Line { number: line_number, text: text.to_string() }),
            _ => None,
        };
        commits.push(Commit {
            hash: hash.to_string(),
            header_line: line_number,
            subject,
            body: Vec::new(),
        });
    }

    commits
}
