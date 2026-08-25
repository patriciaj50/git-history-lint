use crate::parser::{Commit, Line};

const MAX_LINE_LEN: usize = 72;

#[derive(Debug, Clone)]
pub struct Finding {
    pub line: usize,
    pub rule: &'static str,
    pub message: String,
    pub hash: String,
}

type CheckFn = fn(&Commit) -> Vec<(usize, String)>;

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
    Rule { id: "missing-blank-line", check: check_missing_blank_line },
    Rule { id: "trailing-whitespace", check: check_trailing_whitespace },
    Rule { id: "body-line-too-long", check: check_body_line_too_long },
];

pub fn lint_commit(commit: &Commit) -> Vec<Finding> {
    let mut findings = Vec::new();
    for rule in RULES {
        for (line, message) in (rule.check)(commit) {
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

fn check_empty_subject(commit: &Commit) -> Vec<(usize, String)> {
    if commit.subject.is_none() {
        vec![(commit.header_line, "commit has no message".to_string())]
    } else {
        Vec::new()
    }
}

fn check_subject_too_long(commit: &Commit) -> Vec<(usize, String)> {
    match &commit.subject {
        Some(s) if s.text.chars().count() > MAX_LINE_LEN => {
            let len = s.text.chars().count();
            vec![(s.number, format!("subject line is {} characters, limit is {}", len, MAX_LINE_LEN))]
        }
        _ => Vec::new(),
    }
}

fn check_subject_trailing_period(commit: &Commit) -> Vec<(usize, String)> {
    match &commit.subject {
        Some(s) if s.text.trim_end().ends_with('.') => {
            vec![(s.number, "subject line ends with a period".to_string())]
        }
        _ => Vec::new(),
    }
}

fn check_subject_capitalized(commit: &Commit) -> Vec<(usize, String)> {
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

fn check_missing_blank_line(commit: &Commit) -> Vec<(usize, String)> {
    match commit.body.first() {
        Some(l) if !l.text.is_empty() => {
            vec![(l.number, "missing blank line between subject and body".to_string())]
        }
        _ => Vec::new(),
    }
}

fn check_trailing_whitespace(commit: &Commit) -> Vec<(usize, String)> {
    let mut findings = Vec::new();
    let lines: Vec<&Line> = commit.subject.iter().chain(commit.body.iter()).collect();
    for l in lines {
        if !l.text.is_empty() && l.text != l.text.trim_end() {
            findings.push((l.number, "line has trailing whitespace".to_string()));
        }
    }
    findings
}

fn check_body_line_too_long(commit: &Commit) -> Vec<(usize, String)> {
    commit
        .body
        .iter()
        .filter(|l| l.text.chars().count() > MAX_LINE_LEN)
        .map(|l| {
            let len = l.text.chars().count();
            (l.number, format!("body line is {} characters, limit is {}", len, MAX_LINE_LEN))
        })
        .collect()
}
