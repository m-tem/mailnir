use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::template::{SourceConfig, Template};
use crate::MailnirError;

/// Build one merged context per primary source entry.
///
/// Each context is a JSON object keyed by namespace name:
/// - primary namespace → the entry object
/// - global namespaces → the full source array
/// - secondary namespaces → matched object (1:1) or array of objects (1:N)
pub fn build_contexts(
    template: &Template,
    sources: &HashMap<String, Value>,
) -> crate::Result<Vec<Map<String, Value>>> {
    build_contexts_lenient(template, sources)?
        .into_iter()
        .collect()
}

/// Like [`build_contexts`], but continues past per-entry join errors.
///
/// The outer `Result` fails only on structural errors (no primary source,
/// malformed secondary source shape). The inner `Result` for each entry
/// is `Ok(context)` on join success or `Err(JoinMissingMatch | JoinAmbiguousMatch)`
/// on per-entry failure.
pub fn build_contexts_lenient(
    template: &Template,
    sources: &HashMap<String, Value>,
) -> crate::Result<Vec<crate::Result<Map<String, Value>>>> {
    let primary_name = template
        .sources
        .iter()
        .find(|(_, cfg)| cfg.primary == Some(true))
        .map(|(name, _)| name.as_str())
        .ok_or(MailnirError::NoPrimarySource)?;

    let primary_array = sources
        .get(primary_name)
        .and_then(Value::as_array)
        .ok_or_else(|| MailnirError::InvalidDataShape {
            path: std::path::PathBuf::from(primary_name),
            message: "primary source must be an array".into(),
        })?;

    let global_names: Vec<&str> = template
        .sources
        .iter()
        .filter(|(name, cfg)| {
            name.as_str() != primary_name && cfg.primary != Some(true) && cfg.join.is_none()
        })
        .map(|(name, _)| name.as_str())
        .collect();

    let secondary_sources_cfg: Vec<(&str, &SourceConfig)> = template
        .sources
        .iter()
        .filter(|(name, cfg)| name.as_str() != primary_name && cfg.join.is_some())
        .map(|(name, cfg)| (name.as_str(), cfg))
        .collect();

    // Pre-validate secondary source shapes — structural failure, not per-entry.
    let secondary_sources: Vec<(&str, &SourceConfig, &[Value])> = secondary_sources_cfg
        .iter()
        .map(|&(ns_name, ns_cfg)| {
            let array = sources
                .get(ns_name)
                .and_then(Value::as_array)
                .ok_or_else(|| MailnirError::InvalidDataShape {
                    path: std::path::PathBuf::from(ns_name),
                    message: "secondary source must be an array".into(),
                })?;
            Ok((ns_name, ns_cfg, array.as_slice()))
        })
        .collect::<crate::Result<_>>()?;

    let mut results = Vec::with_capacity(primary_array.len());

    for (entry_index, primary_entry) in primary_array.iter().enumerate() {
        results.push(build_single_context(
            primary_name,
            primary_entry,
            entry_index,
            &global_names,
            &secondary_sources,
            sources,
        ));
    }

    Ok(results)
}

/// Build a merged context for a single primary source entry.
///
/// Returns `Ok(context)` or `Err(JoinMissingMatch | JoinAmbiguousMatch)`.
/// Secondary source arrays must be pre-validated before calling.
fn build_single_context(
    primary_name: &str,
    primary_entry: &Value,
    entry_index: usize,
    global_names: &[&str],
    secondary_sources: &[(&str, &SourceConfig, &[Value])],
    sources: &HashMap<String, Value>,
) -> crate::Result<Map<String, Value>> {
    let mut ctx: Map<String, Value> = Map::new();

    ctx.insert(primary_name.to_string(), primary_entry.clone());

    for &global_name in global_names {
        if let Some(data) = sources.get(global_name) {
            ctx.insert(global_name.to_string(), data.clone());
        }
    }

    for &(ns_name, ns_cfg, secondary_array) in secondary_sources {
        let join_map = ns_cfg.join.as_ref().expect("secondary always has join");

        let matches: Vec<&Value> = secondary_array
            .iter()
            .filter(|row| predicates_match(row, join_map, &ctx))
            .collect();

        if ns_cfg.many == Some(true) {
            ctx.insert(
                ns_name.to_string(),
                Value::Array(matches.into_iter().cloned().collect()),
            );
        } else {
            match matches.len() {
                0 => {
                    return Err(MailnirError::JoinMissingMatch {
                        namespace: ns_name.to_string(),
                        entry_index,
                    })
                }
                1 => {
                    ctx.insert(ns_name.to_string(), matches[0].clone());
                }
                n => {
                    return Err(MailnirError::JoinAmbiguousMatch {
                        namespace: ns_name.to_string(),
                        entry_index,
                        match_count: n,
                    })
                }
            }
        }
    }

    Ok(ctx)
}

/// Returns true if all join predicates hold for `row` against `ctx`.
///
/// Each predicate: `row[join_key] == ctx[ref_ns][ref_field]`
fn predicates_match(
    row: &Value,
    join_map: &HashMap<String, String>,
    ctx: &Map<String, Value>,
) -> bool {
    join_map.iter().all(|(join_key, ref_value)| {
        let Some((ref_ns, ref_field)) = ref_value.split_once('.') else {
            return false;
        };
        let expected = ctx.get(ref_ns).and_then(|ns| ns.get(ref_field));
        let actual = row.get(join_key);
        matches!((expected, actual), (Some(e), Some(a)) if e == a)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_template(yaml: &str) -> Template {
        crate::template::parse_template_str(yaml).expect("fixture must parse")
    }

    fn make_sources(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn test_one_to_one_join() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  inst:\n    join:\n      class_id: classes.id\nto: a\nsubject: b\nbody: c",
        );
        let sources = make_sources(&[
            (
                "classes",
                json!([
                    {"id": 1, "name": "Math"},
                    {"id": 2, "name": "Science"},
                    {"id": 3, "name": "History"},
                ]),
            ),
            (
                "inst",
                json!([
                    {"class_id": 2, "name": "Dr. Smith"},
                    {"class_id": 1, "name": "Prof. Jones"},
                    {"class_id": 3, "name": "Ms. Brown"},
                ]),
            ),
        ]);

        let ctxs = build_contexts(&t, &sources).expect("should succeed");
        assert_eq!(ctxs.len(), 3);

        assert_eq!(ctxs[0]["inst"]["name"], json!("Prof. Jones"));
        assert_eq!(ctxs[1]["inst"]["name"], json!("Dr. Smith"));
        assert_eq!(ctxs[2]["inst"]["name"], json!("Ms. Brown"));
    }

    #[test]
    fn test_one_to_n_join() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  students:\n    join:\n      class_id: classes.id\n    many: true\nto: a\nsubject: b\nbody: c",
        );
        let sources = make_sources(&[
            ("classes", json!([{"id": 10, "name": "Algebra"}])),
            (
                "students",
                json!([
                    {"class_id": 10, "name": "Alice"},
                    {"class_id": 10, "name": "Bob"},
                    {"class_id": 10, "name": "Carol"},
                    {"class_id": 10, "name": "Dan"},
                    {"class_id": 10, "name": "Eve"},
                ]),
            ),
        ]);

        let ctxs = build_contexts(&t, &sources).expect("should succeed");
        assert_eq!(ctxs.len(), 1);
        let students = ctxs[0]["students"].as_array().expect("must be array");
        assert_eq!(students.len(), 5);
        assert_eq!(students[2]["name"], json!("Carol"));
    }

    #[test]
    fn test_composite_join() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  rooms:\n    join:\n      building: classes.building\n      floor: classes.floor\nto: a\nsubject: b\nbody: c",
        );
        let sources = make_sources(&[
            (
                "classes",
                json!([
                    {"id": 1, "building": "A", "floor": 2},
                    {"id": 2, "building": "B", "floor": 1},
                ]),
            ),
            (
                "rooms",
                json!([
                    {"building": "B", "floor": 1, "capacity": 30},
                    {"building": "A", "floor": 2, "capacity": 50},
                ]),
            ),
        ]);

        let ctxs = build_contexts(&t, &sources).expect("should succeed");
        assert_eq!(ctxs.len(), 2);
        assert_eq!(ctxs[0]["rooms"]["capacity"], json!(50));
        assert_eq!(ctxs[1]["rooms"]["capacity"], json!(30));
    }

    #[test]
    fn test_global_source() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  cfg: {}\nto: a\nsubject: b\nbody: c",
        );
        let sources = make_sources(&[
            (
                "classes",
                json!([
                    {"id": 1},
                    {"id": 2},
                    {"id": 3},
                ]),
            ),
            (
                "cfg",
                json!([{"smtp_host": "mail.example.com", "from": "admin@example.com"}]),
            ),
        ]);

        let ctxs = build_contexts(&t, &sources).expect("should succeed");
        assert_eq!(ctxs.len(), 3);
        for ctx in &ctxs {
            let cfg = ctx["cfg"].as_array().expect("cfg must be array");
            assert_eq!(cfg[0]["smtp_host"], json!("mail.example.com"));
        }
    }

    #[test]
    fn test_missing_join_match() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  inst:\n    join:\n      class_id: classes.id\nto: a\nsubject: b\nbody: c",
        );
        let sources = make_sources(&[
            ("classes", json!([{"id": 1}, {"id": 99}])),
            ("inst", json!([{"class_id": 1, "name": "Prof. Jones"}])),
        ]);

        let err = build_contexts(&t, &sources).expect_err("should fail");
        assert!(matches!(
            err,
            MailnirError::JoinMissingMatch { namespace, entry_index: 1 }
            if namespace == "inst"
        ));
    }

    #[test]
    fn test_ambiguous_one_to_one() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  inst:\n    join:\n      class_id: classes.id\nto: a\nsubject: b\nbody: c",
        );
        let sources = make_sources(&[
            ("classes", json!([{"id": 5}])),
            (
                "inst",
                json!([
                    {"class_id": 5, "name": "Prof. A"},
                    {"class_id": 5, "name": "Prof. B"},
                ]),
            ),
        ]);

        let err = build_contexts(&t, &sources).expect_err("should fail");
        assert!(matches!(
            err,
            MailnirError::JoinAmbiguousMatch { namespace, entry_index: 0, match_count: 2 }
            if namespace == "inst"
        ));
    }

    // --- build_contexts_lenient tests ---

    #[test]
    fn test_lenient_collects_per_entry_join_failure() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  inst:\n    join:\n      class_id: classes.id\nto: a\nsubject: b\nbody: c",
        );
        let sources = make_sources(&[
            ("classes", json!([{"id": 1}, {"id": 99}])),
            ("inst", json!([{"class_id": 1, "name": "Prof. Jones"}])),
        ]);

        let results = build_contexts_lenient(&t, &sources).expect("outer should succeed");
        assert_eq!(results.len(), 2);
        // Entry 0 (id=1) matches
        assert!(results[0].is_ok());
        // Entry 1 (id=99) has no match
        let err = results[1].as_ref().expect_err("entry 1 should fail");
        assert!(matches!(
            err,
            MailnirError::JoinMissingMatch { namespace, entry_index: 1 }
            if namespace == "inst"
        ));
    }

    #[test]
    fn test_lenient_structural_failure_propagates() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  inst:\n    join:\n      class_id: classes.id\nto: a\nsubject: b\nbody: c",
        );
        // "inst" source is not an array — structural failure
        let sources = make_sources(&[
            ("classes", json!([{"id": 1}])),
            ("inst", json!({"class_id": 1})),
        ]);

        let err = build_contexts_lenient(&t, &sources).expect_err("outer should fail structurally");
        assert!(matches!(err, MailnirError::InvalidDataShape { .. }));
    }

    #[test]
    fn test_lenient_all_ok_when_no_failures() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  inst:\n    join:\n      class_id: classes.id\nto: a\nsubject: b\nbody: c",
        );
        let sources = make_sources(&[
            ("classes", json!([{"id": 1}, {"id": 2}])),
            (
                "inst",
                json!([
                    {"class_id": 1, "name": "A"},
                    {"class_id": 2, "name": "B"},
                ]),
            ),
        ]);

        let results = build_contexts_lenient(&t, &sources).expect("should succeed");
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}
