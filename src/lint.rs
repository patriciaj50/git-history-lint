use crate::config::Config;
use crate::parser::{Commit, Line};

#[derive(Debug, Clone)]
pub struct Finding {
    pub line: usize,
    pub rule: &'static str,
    pub message: String,
    pub hash: String,
}

type CheckFn = fn(&Commit, &Config) -> Vec<(usize, String)>;

struct Rule {
    id: &'static str,
    check: CheckFn,
}

// Adding a rule means adding one entry here and one function below.
const RULES: &[Rule] = &[
    Rule { id: "empty-subject", check: check_empty_subject },
    Rule { id: "subject-too-long", check: check_subject_too_long },
    Rule { id: "subject-trailing-period", check: check_subject_trailing_period },
    Rule { id: "subject-not-capitalized", check: check_subject_capitalized },
    Rule { id: "subject-not-imperative", check: check_subject_imperative_mood },
    Rule { id: "missing-blank-line", check: check_missing_blank_line },
    Rule { id: "trailing-whitespace", check: check_trailing_whitespace },
    Rule { id: "body-line-too-long", check: check_body_line_too_long },
];

/// All rule ids the linter knows about, for validating a config's
/// `disable` entries against typos.
pub fn rule_ids() -> impl Iterator<Item = &'static str> {
    RULES.iter().map(|r| r.id)
}

pub fn lint_commit(commit: &Commit, config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();
    for rule in RULES {
        if config.disabled_rules.contains(rule.id) {
            continue;
        }
        for (line, message) in (rule.check)(commit, config) {
            findings.push(Finding {
                line,
                rule: rule.id,
                message,
                hash: commit.hash.clone(),
            });
        }
    }
    findings.sort_by_key(|f| f.line);
    findings
}

fn check_empty_subject(commit: &Commit, _config: &Config) -> Vec<(usize, String)> {
    if commit.subject.is_none() {
        vec![(commit.header_line, "commit has no message".to_string())]
    } else {
        Vec::new()
    }
}

fn check_subject_too_long(commit: &Commit, config: &Config) -> Vec<(usize, String)> {
    match &commit.subject {
        Some(s) if s.text.chars().count() > config.max_line_len => {
            let len = s.text.chars().count();
            vec![(s.number, format!("subject line is {} characters, limit is {}", len, config.max_line_len))]
        }
        _ => Vec::new(),
    }
}

fn check_subject_trailing_period(commit: &Commit, _config: &Config) -> Vec<(usize, String)> {
    match &commit.subject {
        Some(s) if s.text.trim_end().ends_with('.') => {
            vec![(s.number, "subject line ends with a period".to_string())]
        }
        _ => Vec::new(),
    }
}

fn check_subject_capitalized(commit: &Commit, _config: &Config) -> Vec<(usize, String)> {
    match &commit.subject {
        Some(s) => match s.text.chars().next() {
            Some(c) if c.is_lowercase() => {
                vec![(s.number, "subject line should start with a capital letter".to_string())]
            }
            _ => Vec::new(),
        },
        None => Vec::new(),
    }
}

// Catching every non-imperative subject would need real grammar analysis,
// so this just flags the past-tense and gerund forms of the verbs people
// actually reach for in commit subjects ("Fixed the bug" instead of "Fix
// the bug"). It only looks at the first word, so it won't catch every case
// and won't touch subjects that already read as commands.
const NON_IMPERATIVE_FIRST_WORDS: &[&str] = &[
    "Fixed", "Added", "Changed", "Removed", "Updated", "Deleted", "Created",
    "Implemented", "Refactored", "Improved", "Renamed", "Merged", "Reverted",
    "Bumped", "Cleaned", "Moved", "Replaced",
    "Fixing", "Adding", "Changing", "Removing", "Updating", "Deleting",
    "Creating", "Implementing", "Refactoring", "Improving", "Renaming",
    "Merging", "Reverting", "Bumping", "Cleaning", "Moving", "Replacing",
];

fn check_subject_imperative_mood(commit: &Commit, _config: &Config) -> Vec<(usize, String)> {
    match &commit.subject {
        Some(s) => {
            let first_word = s.text.split_whitespace().next().unwrap_or("");
            if NON_IMPERATIVE_FIRST_WORDS.contains(&first_word) {
                vec![(
                    s.number,
                    format!("subject line should use imperative mood, not \"{}\"", first_word),
                )]
            } else {
                Vec::new()
            }
        }
        None => Vec::new(),
    }
}

fn check_missing_blank_line(commit: &Commit, _config: &Config) -> Vec<(usize, String)> {
    match commit.body.first() {
        Some(l) if !l.text.is_empty() => {
            vec![(l.number, "missing blank line between subject and body".to_string())]
        }
        _ => Vec::new(),
    }
}

fn check_trailing_whitespace(commit: &Commit, _config: &Config) -> Vec<(usize, String)> {
    let mut findings = Vec::new();
    let lines: Vec<&Line> = commit.subject.iter().chain(commit.body.iter()).collect();
    for l in lines {
        if !l.text.is_empty() && l.text != l.text.trim_end() {
            findings.push((l.number, "line has trailing whitespace".to_string()));
        }
    }
    findings
}

fn check_body_line_too_long(commit: &Commit, config: &Config) -> Vec<(usize, String)> {
    commit
        .body
        .iter()
        .filter(|l| l.text.chars().count() > config.max_line_len)
        .map(|l| {
            let len = l.text.chars().count();
            (l.number, format!("body line is {} characters, limit is {}", len, config.max_line_len))
        })
        .collect()
}
