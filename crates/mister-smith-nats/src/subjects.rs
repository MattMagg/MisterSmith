//! NATS subject routing helpers.
//!
//! Maps `SubjectTaxonomy` patterns to NATS subject strings and provides
//! NATS-specific subject validation and wildcard construction.

use mister_smith_transport::TransportError;

/// Maximum NATS subject length.
const MAX_SUBJECT_LENGTH: usize = 4096;

/// Validate a NATS subject string.
pub fn validate_nats_subject(subject: &str) -> Result<(), TransportError> {
    if subject.is_empty() {
        return Err(TransportError::SubjectInvalid(
            "NATS subject must not be empty".into(),
        ));
    }
    if subject.len() > MAX_SUBJECT_LENGTH {
        return Err(TransportError::SubjectInvalid(format!(
            "NATS subject exceeds max length of {MAX_SUBJECT_LENGTH}: {}",
            subject.len()
        )));
    }
    if subject.contains(' ') {
        return Err(TransportError::SubjectInvalid(format!(
            "NATS subject must not contain spaces: {subject}"
        )));
    }
    Ok(())
}

/// Convert a `SubjectTaxonomy` subject to a NATS subject string.
///
/// The taxonomy already produces NATS-compatible dot-separated subjects,
/// so this is primarily a validation pass.
pub fn to_nats_subject(subject: &str) -> Result<String, TransportError> {
    validate_nats_subject(subject)?;
    Ok(subject.to_string())
}

/// Build a wildcard subscription subject for all subjects under a prefix.
///
/// Uses NATS `>` wildcard for multi-level matching.
pub fn wildcard_all(prefix: &str) -> Result<String, TransportError> {
    if prefix.is_empty() {
        return Ok(">".to_string());
    }
    let subject = format!("{prefix}.>");
    validate_nats_subject(&subject)?;
    Ok(subject)
}

/// Build a single-level wildcard subscription subject.
///
/// Uses NATS `*` wildcard for single token matching.
pub fn wildcard_single(prefix: &str, suffix: &str) -> Result<String, TransportError> {
    let subject = if suffix.is_empty() {
        format!("{prefix}.*")
    } else {
        format!("{prefix}.*.{suffix}")
    };
    validate_nats_subject(&subject)?;
    Ok(subject)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_nats_subject() {
        assert!(validate_nats_subject("agents.worker-1.status").is_ok());
        assert!(validate_nats_subject("tasks.code-review.assignment").is_ok());
        assert!(validate_nats_subject("system.health").is_ok());
    }

    #[test]
    fn rejects_empty_subject() {
        assert!(validate_nats_subject("").is_err());
    }

    #[test]
    fn rejects_spaces() {
        assert!(validate_nats_subject("my subject").is_err());
    }

    #[test]
    fn to_nats_subject_passthrough() {
        let subject = to_nats_subject("agents.worker-1.commands.execute").unwrap();
        assert_eq!(subject, "agents.worker-1.commands.execute");
    }

    #[test]
    fn wildcard_all_subjects() {
        assert_eq!(wildcard_all("agents").unwrap(), "agents.>");
        assert_eq!(wildcard_all("").unwrap(), ">");
    }

    #[test]
    fn wildcard_single_level() {
        assert_eq!(
            wildcard_single("tasks", "assignment").unwrap(),
            "tasks.*.assignment"
        );
        assert_eq!(wildcard_single("agents", "").unwrap(), "agents.*");
    }
}
