use git_history_lint::config::Config;
use git_history_lint::{lint, parser};

struct Case {
    name: &'static str,
    input: String,
    expected: Vec<(&'static str, usize)>,
}

fn run(input: &str) -> Vec<(&'static str, usize)> {
    let config = Config::default();
    let mut findings = Vec::new();
    for commit in parser::parse(input) {
        for f in lint::lint_commit(&commit, &config) {
            findings.push((f.rule, f.line));
        }
    }
    findings.sort_by_key(|&(_, line)| line);
    findings
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "clean commit produces no findings",
            input: vec![
                "commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(), // 1
                "Author: Jane Doe <jane@example.com>".to_string(),            // 2
                "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),          // 3
                String::new(),                                                // 4
                "    Fix off-by-one error in range check".to_string(),        // 5
                String::new(),                                                // 6
                "    The loop previously iterated one past the end of the slice".to_string(), // 7
                "    when the start index was zero.".to_string(),             // 8
            ]
            .join("\n"),
            expected: vec![],
        },
        Case {
            name: "subject exactly at the length limit is fine",
            input: header_block("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", &"A".repeat(72)),
            expected: vec![],
        },
        Case {
            name: "subject one character over the limit is flagged",
            input: header_block("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", &"A".repeat(73)),
            expected: vec![("subject-too-long", 5)],
        },
        Case {
            name: "subject ending with a period is flagged",
            input: header_block("cccccccccccccccccccccccccccccccccccccccc", "Fix the bug."),
            expected: vec![("subject-trailing-period", 5)],
        },
        Case {
            name: "subject starting lowercase is flagged",
            input: header_block("dddddddddddddddddddddddddddddddddddddddd", "fix the bug"),
            expected: vec![("subject-not-capitalized", 5)],
        },
        Case {
            name: "subject starting with a digit is not a false positive",
            input: header_block("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee", "3 bugs fixed in this patch"),
            expected: vec![],
        },
        Case {
            name: "unicode subject within the character limit is not flagged by byte length",
            // 40 chars, 80 bytes: would trip a byte-length check but not a char-count one.
            input: header_block("ffffffffffffffffffffffffffffffffffffffff", &"\u{c9}".repeat(40)),
            expected: vec![],
        },
        Case {
            name: "missing blank line between subject and body is flagged",
            input: vec![
                "commit 1111111111111111111111111111111111111111".to_string(), // 1
                "Author: Jane Doe <jane@example.com>".to_string(),             // 2
                "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),           // 3
                String::new(),                                                 // 4
                "    Add feature X".to_string(),                               // 5
                "    This line should have been separated by a blank line.".to_string(), // 6
            ]
            .join("\n"),
            expected: vec![("missing-blank-line", 6)],
        },
        Case {
            name: "trailing whitespace on a body line is flagged",
            input: vec![
                "commit 2222222222222222222222222222222222222222".to_string(), // 1
                "Author: Jane Doe <jane@example.com>".to_string(),             // 2
                "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),           // 3
                String::new(),                                                 // 4
                "    Add feature X".to_string(),                               // 5
                String::new(),                                                 // 6
                format!("    Body line with trailing space{}", "   "),         // 7
            ]
            .join("\n"),
            expected: vec![("trailing-whitespace", 7)],
        },
        Case {
            name: "a whitespace-only line counts as trailing whitespace, not a blank separator",
            input: vec![
                "commit 3333333333333333333333333333333333333333".to_string(), // 1
                "Author: Jane Doe <jane@example.com>".to_string(),             // 2
                "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),           // 3
                String::new(),                                                 // 4
                "    Subject line here".to_string(),                          // 5
                String::new(),                                                 // 6
                format!("    {}", "   "),                                      // 7: 4-space indent + 3 spaces
            ]
            .join("\n"),
            expected: vec![("trailing-whitespace", 7)],
        },
        Case {
            name: "a commit with no message at all is flagged once, at its header line",
            input: vec![
                "commit 4444444444444444444444444444444444444444".to_string(), // 1
                "Author: Jane Doe <jane@example.com>".to_string(),             // 2
                "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),           // 3
                String::new(),                                                 // 4
            ]
            .join("\n"),
            expected: vec![("empty-subject", 1)],
        },
        Case {
            name: "a merge commit's extra header line is skipped correctly",
            input: vec![
                "commit 5555555555555555555555555555555555555555".to_string(), // 1
                "Merge: aaaa111 bbbb222".to_string(),                          // 2
                "Author: Jane Doe <jane@example.com>".to_string(),             // 3
                "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),           // 4
                String::new(),                                                 // 5
                "    Merge branch 'feature' into main".to_string(),           // 6
            ]
            .join("\n"),
            expected: vec![],
        },
        Case {
            name: "line numbers keep counting correctly across multiple commits",
            input: vec![
                "commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(), // 1
                "Author: Jane Doe <jane@example.com>".to_string(),            // 2
                "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),          // 3
                String::new(),                                                // 4
                "    Fix off-by-one error in range check".to_string(),        // 5
                String::new(),                                                // 6
                "    The loop previously iterated one past the end of the slice".to_string(), // 7
                "    when the start index was zero.".to_string(),             // 8
                String::new(),                                                // 9
                "commit 6666666666666666666666666666666666666666".to_string(), // 10
                "Author: Jane Doe <jane@example.com>".to_string(),            // 11
                "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),          // 12
                String::new(),                                                // 13
                "    fix things".to_string(),                                 // 14
            ]
            .join("\n"),
            expected: vec![("subject-not-capitalized", 14)],
        },
        Case {
            name: "CRLF line endings are handled the same as LF",
            input: vec![
                "commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                "Author: Jane Doe <jane@example.com>".to_string(),
                "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),
                String::new(),
                "    Fix off-by-one error in range check".to_string(),
            ]
            .join("\r\n"),
            expected: vec![],
        },
    ]
}

fn header_block(hash: &str, subject: &str) -> String {
    vec![
        format!("commit {}", hash),
        "Author: Jane Doe <jane@example.com>".to_string(),
        "Date:   Mon Jan 5 10:00:00 2026 +0000".to_string(),
        String::new(),
        format!("    {}", subject),
    ]
    .join("\n")
}

#[test]
fn table_driven_rule_checks() {
    for case in cases() {
        let actual = run(&case.input);
        assert_eq!(actual, case.expected, "case failed: {}", case.name);
    }
}

#[test]
fn disabled_rule_produces_no_finding_for_it() {
    let input = header_block("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "fix the bug");
    let mut config = Config::default();
    config.disabled_rules.insert("subject-not-capitalized".to_string());

    let commit = parser::parse(&input).into_iter().next().unwrap();
    let findings: Vec<&'static str> = lint::lint_commit(&commit, &config)
        .into_iter()
        .map(|f| f.rule)
        .collect();

    assert!(!findings.contains(&"subject-not-capitalized"));
}

#[test]
fn custom_max_line_len_shortens_the_allowed_subject() {
    let input = header_block("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "Short subject line");
    let mut config = Config::default();
    config.max_line_len = 10;

    let commit = parser::parse(&input).into_iter().next().unwrap();
    let findings = lint::lint_commit(&commit, &config);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule, "subject-too-long");
}
