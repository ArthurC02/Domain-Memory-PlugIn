use std::path::Path;

use crate::coverage::with_references;
use crate::evidence::Verifier;
use crate::json::{Object, Value};
use crate::registry_write::{read_records, write_records};
use crate::store::{ASSETS, Registry, has_registry};
use crate::transaction::mutate_registry;

/// One record whose citations moved, and the files they moved within.
struct Moved {
    asset: &'static str,
    record_id: String,
    citations: usize,
    paths: Vec<String>,
}

/// A citation this command cannot write: its excerpt is no longer where it
/// was and no single place in the file holds it, either because the text
/// changed or because it occurs more than once. Only a person can settle it.
struct Unresolved {
    asset: &'static str,
    record_id: String,
    path: String,
    reason: String,
}

/// Rewrites the stored lines of every `shifted` citation to where its excerpt
/// now sits, leaving `path` and `excerpt_sha256` as they are. A shifted
/// citation is reported only where the excerpt occurs exactly once in the
/// file, so the lines it moves to are the ones the Registry already cites.
fn refreshed(
    repo_root: &Path,
    record: &Object,
    mut unresolved: Option<&mut Vec<(String, String)>>,
) -> Result<(Object, usize, Vec<String>), String> {
    let mut verifier = Verifier::new(repo_root, false);
    let mut moved = 0;
    let mut paths: Vec<String> = Vec::new();
    let refreshed = with_references(
        &Value::Object(record.clone()),
        &mut |reference| {
            let Value::Object(object) = reference else {
                return Ok(reference.clone());
            };
            let verdict = verifier.check(reference)?;
            let status = verdict.get("status").and_then(Value::as_str);
            if status != Some("shifted") {
                if let (Some(found), Some("stale")) = (unresolved.as_deref_mut(), status) {
                    let text = |key: &str| {
                        verdict
                            .get(key)
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string()
                    };
                    found.push((text("path"), text("reason")));
                }
                return Ok(reference.clone());
            }
            let Some(lines @ Value::Object(_)) = verdict.get("lines") else {
                return Ok(reference.clone());
            };
            let mut refreshed = object.clone();
            refreshed.insert("lines", lines.clone());
            moved += 1;
            if let Some(path) = verdict.get("path").and_then(Value::as_str) {
                let path = path.to_string();
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
            Ok(Value::Object(refreshed))
        },
        0,
    )?;
    let Value::Object(refreshed) = refreshed else {
        return Ok((record.clone(), 0, paths));
    };
    paths.sort();
    Ok((refreshed, moved, paths))
}

fn moved_records(root: &Path, repo_root: &Path) -> Result<(Vec<Moved>, Vec<Unresolved>), String> {
    let registry = Registry::open(root)?;
    let mut found = Vec::new();
    let mut unresolved = Vec::new();
    for (asset, _, _) in ASSETS {
        for item in registry.records(asset)? {
            let Value::Object(record) = item else {
                continue;
            };
            let mut stale = Vec::new();
            let (_, citations, paths) = refreshed(repo_root, &record, Some(&mut stale))?;
            let record_id = record
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            for (path, reason) in stale {
                unresolved.push(Unresolved {
                    asset,
                    record_id: record_id.clone(),
                    path,
                    reason,
                });
            }
            if citations > 0 {
                found.push(Moved {
                    asset,
                    record_id,
                    citations,
                    paths,
                });
            }
        }
    }
    Ok((found, unresolved))
}

fn report_of(moved: &[Moved], unresolved: &[Unresolved]) -> Object {
    let mut records = Vec::new();
    let mut citations = 0;
    let mut files: Vec<String> = Vec::new();
    for found in moved {
        citations += found.citations;
        for path in &found.paths {
            if !files.contains(path) {
                files.push(path.clone());
            }
        }
        let mut row = Object::new();
        row.insert("asset", Value::Str(found.asset.to_string()));
        row.insert("id", Value::Str(found.record_id.clone()));
        row.insert("citations", Value::int(found.citations));
        row.insert(
            "paths",
            Value::Array(found.paths.iter().cloned().map(Value::Str).collect()),
        );
        records.push(Value::Object(row));
    }
    files.sort();
    let mut left = Vec::new();
    for found in unresolved {
        let mut row = Object::new();
        row.insert("asset", Value::Str(found.asset.to_string()));
        row.insert("id", Value::Str(found.record_id.clone()));
        row.insert("path", Value::Str(found.path.clone()));
        row.insert("reason", Value::Str(found.reason.clone()));
        left.push(Value::Object(row));
    }
    let mut report = Object::new();
    report.insert("format", Value::Str("domain-memory-refresh/v1".to_string()));
    report.insert("records", Value::Array(records));
    report.insert("citations", Value::int(citations));
    report.insert(
        "files",
        Value::Array(files.into_iter().map(Value::Str).collect()),
    );
    report.insert("unresolved", Value::Array(left));
    report
}

/// Re-cites every `shifted` citation in place: the excerpt is unchanged, so
/// only the lines that say where to read it move, and no record's status,
/// statement or evidence digest changes. Writing nothing when none shifted
/// keeps it idempotent and leaves no audit event behind.
pub fn refresh_citations(root: &Path, repo_root: &Path) -> Result<Object, String> {
    if !has_registry(root) {
        return Ok(report_of(&[], &[]));
    }
    let (moved, unresolved) = moved_records(root, repo_root)?;
    let report = report_of(&moved, &unresolved);
    if moved.is_empty() {
        return Ok(report);
    }
    let mut event = Object::new();
    event.insert("operation", Value::Str("refresh-citations".to_string()));
    event.insert(
        "citations",
        report.get("citations").cloned().unwrap_or(Value::Null),
    );
    event.insert(
        "records",
        Value::Array(
            moved
                .iter()
                .map(|found| Value::Str(format!("{}/{}", found.asset, found.record_id)))
                .collect(),
        ),
    );
    mutate_registry(
        root,
        repo_root,
        |staging| {
            for (asset, _, _) in ASSETS {
                let mut records = read_records(staging, asset)?;
                let mut touched = false;
                for record in &mut records {
                    let (refreshed, citations, _) = refreshed(repo_root, record, None)?;
                    if citations > 0 {
                        *record = refreshed;
                        touched = true;
                    }
                }
                if touched {
                    write_records(staging, asset, &records)?;
                }
            }
            Ok(())
        },
        || event,
    )?;
    Ok(report)
}
