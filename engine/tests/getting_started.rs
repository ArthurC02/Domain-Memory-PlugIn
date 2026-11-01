//! Runs `references/getting-started.md` against `examples/getting-started`: every
//! `$ domain-memory` line in a `console` block runs in order, and the lines after it
//! must be its output (stdout, then stderr), where `...` stands for any lines. A
//! `<!-- write: <path> -->` comment writes the block after it, and `<!-- exit: N -->`
//! sets the exit code the next block's last command must end with (0 otherwise).

use std::path::{Path, PathBuf};
use std::process::Command;

fn plugin() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn git(repo: &Path, args: &[&str]) {
    let done = Command::new("git")
        .args([
            "-c",
            "core.autocrlf=false",
            "-c",
            "user.name=Reader",
            "-c",
            "user.email=reader@example.com",
        ])
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        done.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&done.stderr)
    );
}

/// Splits a command line on spaces, keeping a double-quoted span as one word.
fn words(line: &str) -> Vec<String> {
    let (mut words, mut word, mut quoted, mut started) = (Vec::new(), String::new(), false, false);
    for c in line.chars() {
        match c {
            '"' => (quoted, started) = (!quoted, true),
            ' ' if !quoted => {
                if started {
                    words.push(std::mem::take(&mut word));
                }
                started = false;
            }
            _ => (word.push(c), started = true).1,
        }
    }
    if started {
        words.push(word);
    }
    words
}

/// Whether `actual` is `expected` with each `...` line standing for any lines.
fn matches(expected: &[&str], actual: &[&str]) -> bool {
    match expected.split_first() {
        None => actual.is_empty(),
        Some((&"...", rest)) => (0..=actual.len()).any(|skip| matches(rest, &actual[skip..])),
        Some((line, rest)) => actual.first() == Some(line) && matches(rest, &actual[1..]),
    }
}

fn run_block(repo: &Path, block: &[&str], exit: i32) {
    let mut commands: Vec<(&str, Vec<&str>)> = Vec::new();
    for line in block {
        match line.strip_prefix("$ ") {
            Some(command) => commands.push((command, Vec::new())),
            None => commands
                .last_mut()
                .expect("output before a command")
                .1
                .push(line),
        }
    }
    let last = commands.len() - 1;
    for (at, (command, expected)) in commands.iter().enumerate() {
        let words = words(command);
        assert_eq!("domain-memory", words[0], "{command}");
        let done = Command::new(env!("CARGO_BIN_EXE_domain-memory"))
            .args(&words[1..])
            .current_dir(repo)
            .output()
            .unwrap();
        let printed = format!(
            "{}{}",
            String::from_utf8_lossy(&done.stdout),
            String::from_utf8_lossy(&done.stderr)
        );
        let actual: Vec<&str> = printed.lines().collect();
        assert!(
            matches(expected, &actual),
            "$ {command}\nexpected:\n{}\nprinted:\n{printed}",
            expected.join("\n")
        );
        let wanted = if at == last { exit } else { 0 };
        assert_eq!(Some(wanted), done.status.code(), "$ {command}\n{printed}");
    }
}

#[test]
fn the_getting_started_page_runs_as_written() {
    let page =
        std::fs::read_to_string(plugin().join("references").join("getting-started.md")).unwrap();
    let repo = std::env::temp_dir().join(format!("dm-getting-started-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&repo);
    copy_tree(&plugin().join("examples").join("getting-started"), &repo);
    git(&repo, &["init", "-q"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "Library loans"]);
    let lines: Vec<&str> = page.lines().collect();
    let (mut write, mut exit, mut ran) = (None, 0, 0);
    let mut at = 0;
    while at < lines.len() {
        let line = lines[at];
        if let Some(path) = line
            .strip_prefix("<!-- write: ")
            .and_then(|rest| rest.strip_suffix(" -->"))
        {
            write = Some(path);
        } else if let Some(code) = line
            .strip_prefix("<!-- exit: ")
            .and_then(|rest| rest.strip_suffix(" -->"))
        {
            exit = code.parse().unwrap();
        } else if line.starts_with("```") && line.len() > 3 {
            let end = at
                + 1
                + lines[at + 1..]
                    .iter()
                    .position(|line| *line == "```")
                    .unwrap();
            let block = &lines[at + 1..end];
            if let Some(path) = write.take() {
                std::fs::write(repo.join(path), block.join("\n") + "\n").unwrap();
            } else if line == "```console" {
                run_block(&repo, block, exit);
                exit = 0;
                ran += 1;
            }
            at = end;
        }
        at += 1;
    }
    let _ = std::fs::remove_dir_all(&repo);
    assert_eq!(7, ran, "every console block ran");
}
