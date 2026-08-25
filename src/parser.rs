//! Parses the plain-text output of `git log` (the default format, not
//! `-p` or `--stat`) into individual commits, keeping track of which
//! source line each piece of the message came from. That's what lets the
//! linter point back at an exact line instead of just naming a commit.

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

/// Parses `input` into a list of commits. Anything before the first
/// `commit ` line is ignored, so callers can feed in a log with a
/// leading banner or pager artifacts without pre-cleaning it.
pub fn parse(input: &str) -> Vec<Commit> {
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
