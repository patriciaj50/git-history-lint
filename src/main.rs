use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use git_history_lint::{config, lint, parser};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: git-history-lint <path>|-");
            eprintln!("  pipe git log output in, e.g.: git log | git-history-lint -");
            return ExitCode::from(2);
        }
    };

    let config = match config::load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("git-history-lint: {}", err);
            return ExitCode::from(2);
        }
    };
    if let Some(bad) = config
        .disabled_rules
        .iter()
        .find(|id| !lint::rule_ids().any(|known| known == id.as_str()))
    {
        eprintln!("git-history-lint: unknown rule in config: {}", bad);
        return ExitCode::from(2);
    }

    let input = match read_input(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("git-history-lint: {}: {}", path, err);
            return ExitCode::from(2);
        }
    };

    let commits = parser::parse(&input);
    let mut rule_counts: HashMap<&'static str, usize> = HashMap::new();

    for commit in &commits {
        for finding in lint::lint_commit(commit, &config) {
            *rule_counts.entry(finding.rule).or_insert(0) += 1;
            let short_hash = &finding.hash[..finding.hash.len().min(7)];
            println!(
                "{}:{}: {} [{}] ({})",
                path, finding.line, finding.message, finding.rule, short_hash
            );
        }
    }

    if rule_counts.is_empty() {
        ExitCode::SUCCESS
    } else {
        println!("{}", summary_line(&rule_counts));
        ExitCode::from(1)
    }
}

/// One line tallying findings per rule, in the linter's fixed rule order
/// so the breakdown reads the same across runs regardless of hash order.
fn summary_line(rule_counts: &HashMap<&'static str, usize>) -> String {
    let total: usize = rule_counts.values().sum();
    let breakdown: Vec<String> = lint::rule_ids()
        .filter_map(|id| rule_counts.get(id).map(|n| format!("{} ({})", id, n)))
        .collect();
    format!(
        "{} finding{} total: {}",
        total,
        if total == 1 { "" } else { "s" },
        breakdown.join(", ")
    )
}

fn read_input(path: &str) -> io::Result<String> {
    if path == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else {
        fs::read_to_string(path)
    }
}
