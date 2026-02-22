use crate::template::types::Template;

pub fn validate_sources(template: &Template) -> crate::Result<()> {
    let primaries: Vec<String> = template
        .sources
        .iter()
        .filter(|(_, cfg)| cfg.primary == Some(true))
        .map(|(name, _)| name.clone())
        .collect();

    match primaries.len() {
        0 => return Err(crate::MailnirError::NoPrimarySource),
        1 => {}
        _ => {
            let mut sorted = primaries;
            sorted.sort();
            return Err(crate::MailnirError::MultiplePrimarySource { namespaces: sorted });
        }
    }

    for (namespace, cfg) in &template.sources {
        let Some(join_map) = &cfg.join else {
            continue;
        };
        for (join_key, ref_value) in join_map {
            let parts: Vec<&str> = ref_value.splitn(2, '.').collect();
            let valid = parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty();
            if !valid {
                return Err(crate::MailnirError::InvalidJoinRef {
                    namespace: namespace.clone(),
                    join_key: join_key.clone(),
                    ref_value: ref_value.clone(),
                });
            }

            let ref_namespace = parts[0];

            if ref_namespace == namespace {
                return Err(crate::MailnirError::SelfJoin {
                    namespace: namespace.clone(),
                });
            }

            if !template.sources.contains_key(ref_namespace) {
                return Err(crate::MailnirError::UnknownJoinNamespace {
                    namespace: namespace.clone(),
                    join_key: join_key.clone(),
                    ref_namespace: ref_namespace.to_string(),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::parse::parse_template_str;

    fn make_template(yaml: &str) -> Template {
        parse_template_str(yaml).expect("fixture must parse")
    }

    #[test]
    fn test_validate_valid_single_source() {
        let t = make_template("sources:\n  p: {primary: true}\nto: a\nsubject: b\nbody: c");
        assert!(validate_sources(&t).is_ok());
    }

    #[test]
    fn test_validate_valid_with_joins() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  inst:\n    join:\n      class_id: classes.id\nto: a\nsubject: b\nbody: c",
        );
        assert!(validate_sources(&t).is_ok());
    }

    #[test]
    fn test_validate_valid_composite_join() {
        let t = make_template(
            "sources:\n  classes: {primary: true}\n  inst:\n    join:\n      class_id: classes.id\n      term: classes.term\nto: a\nsubject: b\nbody: c",
        );
        assert!(validate_sources(&t).is_ok());
    }

    #[test]
    fn test_validate_valid_global_source() {
        // global source: neither primary nor join
        let t = make_template(
            "sources:\n  primary: {primary: true}\n  global: {}\nto: a\nsubject: b\nbody: c",
        );
        assert!(validate_sources(&t).is_ok());
    }

    #[test]
    fn test_validate_no_primary() {
        let t = make_template("sources:\n  a: {}\n  b: {}\nto: x\nsubject: y\nbody: z");
        assert!(matches!(
            validate_sources(&t),
            Err(crate::MailnirError::NoPrimarySource)
        ));
    }

    #[test]
    fn test_validate_multiple_primaries() {
        let t = make_template(
            "sources:\n  a: {primary: true}\n  b: {primary: true}\nto: x\nsubject: y\nbody: z",
        );
        assert!(matches!(
            validate_sources(&t),
            Err(crate::MailnirError::MultiplePrimarySource { .. })
        ));
    }

    #[test]
    fn test_validate_invalid_join_ref_no_dot() {
        let t = make_template(
            "sources:\n  p: {primary: true}\n  s:\n    join:\n      key: justonepart\nto: a\nsubject: b\nbody: c",
        );
        assert!(matches!(
            validate_sources(&t),
            Err(crate::MailnirError::InvalidJoinRef { .. })
        ));
    }

    #[test]
    fn test_validate_invalid_join_ref_empty_parts() {
        // leading dot: empty namespace part
        let t = make_template(
            "sources:\n  p: {primary: true}\n  s:\n    join:\n      key: .field\nto: a\nsubject: b\nbody: c",
        );
        assert!(matches!(
            validate_sources(&t),
            Err(crate::MailnirError::InvalidJoinRef { .. })
        ));

        // trailing dot: empty field part
        let t2 = make_template(
            "sources:\n  p: {primary: true}\n  s:\n    join:\n      key: namespace.\nto: a\nsubject: b\nbody: c",
        );
        assert!(matches!(
            validate_sources(&t2),
            Err(crate::MailnirError::InvalidJoinRef { .. })
        ));
    }

    #[test]
    fn test_validate_unknown_join_namespace() {
        let t = make_template(
            "sources:\n  p: {primary: true}\n  s:\n    join:\n      key: missing.id\nto: a\nsubject: b\nbody: c",
        );
        assert!(matches!(
            validate_sources(&t),
            Err(crate::MailnirError::UnknownJoinNamespace { .. })
        ));
    }

    #[test]
    fn test_validate_self_join() {
        let t = make_template(
            "sources:\n  p: {primary: true}\n  s:\n    join:\n      key: s.id\nto: a\nsubject: b\nbody: c",
        );
        assert!(matches!(
            validate_sources(&t),
            Err(crate::MailnirError::SelfJoin { .. })
        ));
    }
}
