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
    let mut found_any = false;

    for commit in &commits {
        for finding in lint::lint_commit(commit, &config) {
            found_any = true;
            let short_hash = &finding.hash[..finding.hash.len().min(7)];
            println!(
                "{}:{}: {} [{}] ({})",
                path, finding.line, finding.message, finding.rule, short_hash
            );
        }
    }

    if found_any {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
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
