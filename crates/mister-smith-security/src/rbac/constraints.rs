//! ABAC policy constraints for attribute-based access control.
//!
//! [`PolicyConstraints`] layer additional conditions on top of role-based
//! permissions.  They are evaluated against a context map of string key-value
//! pairs supplied with each [`super::AuthorizationRequest`].
//!
//! Supported constraint dimensions:
//!
//! - **Time window** — restrict to business hours / specific weekdays.
//! - **IP ranges** — restrict to allowed CIDR ranges (IPv4/IPv6 aware).
//! - **Resource ownership** — require the caller to own the target resource.

use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr};
use tracing::warn;

// ---------------------------------------------------------------------------
// PolicyConstraints
// ---------------------------------------------------------------------------

/// Optional ABAC constraints attached to a [`super::permission::Permission`].
///
/// All present constraints must be satisfied (logical AND).  A field set to
/// `None` imposes no restriction on that dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConstraints {
    /// Restrict access to a time window (business hours, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_window: Option<TimeWindow>,
    /// Restrict access to specific IP CIDR ranges (e.g. `["10.0.0.0/8"]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_ranges: Option<Vec<String>>,
    /// When `true`, the caller must own the target resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_owner: Option<bool>,
}

impl PolicyConstraints {
    /// Evaluate all present constraints against the supplied `context`.
    ///
    /// Expected context keys:
    ///
    /// | Key | Description | Example |
    /// |-----|-------------|---------|
    /// | `hour` | Current hour (0-23) | `"14"` |
    /// | `day` | Weekday name (lowercase) | `"monday"` |
    /// | `ip` | Caller IP address | `"10.0.1.42"` |
    /// | `is_owner` | Whether caller owns the resource | `"true"` |
    ///
    /// Returns `true` if **all** present constraints are satisfied.  Missing
    /// context keys cause the corresponding constraint to fail (deny by
    /// default).
    pub fn evaluate(&self, context: &HashMap<String, String>) -> bool {
        if let Some(ref tw) = self.time_window {
            if !Self::evaluate_time_window(tw, context) {
                return false;
            }
        }

        if let Some(ref ranges) = self.ip_ranges {
            if !Self::evaluate_ip_ranges(ranges, context) {
                return false;
            }
        }

        if let Some(require_owner) = self.resource_owner {
            if require_owner && !Self::evaluate_resource_owner(context) {
                return false;
            }
        }

        true
    }

    // -- private helpers ----------------------------------------------------

    /// Check whether the current hour and day fall within the time window.
    fn evaluate_time_window(tw: &TimeWindow, context: &HashMap<String, String>) -> bool {
        // Hour check
        let hour = match context.get("hour").and_then(|h| h.parse::<u8>().ok()) {
            Some(h) => h,
            None => {
                warn!("time_window constraint: missing or invalid 'hour' in context");
                return false;
            }
        };

        let in_hours = if tw.start_hour <= tw.end_hour {
            // Normal range: e.g. 9..17
            hour >= tw.start_hour && hour < tw.end_hour
        } else {
            // Overnight range: e.g. 22..6 means 22-23 or 0-5
            hour >= tw.start_hour || hour < tw.end_hour
        };

        if !in_hours {
            return false;
        }

        // Day check (if days list is non-empty)
        if !tw.days.is_empty() {
            let day = match context.get("day") {
                Some(d) => d.to_lowercase(),
                None => {
                    warn!("time_window constraint: missing 'day' in context");
                    return false;
                }
            };

            let day_match = tw.days.iter().any(|d| d.to_lowercase() == day);
            if !day_match {
                return false;
            }
        }

        true
    }

    /// Check whether the caller's IP falls within one of the allowed ranges.
    ///
    /// Invalid caller IP or invalid CIDR entries are treated as denial. Invalid
    /// CIDRs also emit warning logs to help diagnose misconfiguration.
    fn evaluate_ip_ranges(ranges: &[String], context: &HashMap<String, String>) -> bool {
        let ip = match context.get("ip") {
            Some(ip) => ip,
            None => {
                warn!("ip_ranges constraint: missing 'ip' in context");
                return false;
            }
        };

        let parsed_ip = match ip.parse::<IpAddr>() {
            Ok(parsed) => parsed,
            Err(err) => {
                warn!(
                    ip = ip,
                    error = %err,
                    "ip_ranges constraint: invalid caller IP in context"
                );
                return false;
            }
        };

        let mut has_invalid_cidr = false;
        let mut matched = false;

        for cidr in ranges {
            match ip_in_cidr(parsed_ip, cidr) {
                Ok(is_match) => {
                    if is_match {
                        matched = true;
                    }
                }
                Err(err) => {
                    has_invalid_cidr = true;
                    warn!(
                        cidr = %cidr,
                        error = %err,
                        "ip_ranges constraint: invalid CIDR entry"
                    );
                }
            }
        }

        if has_invalid_cidr {
            return false;
        }

        matched
    }

    /// Check the resource ownership flag.
    fn evaluate_resource_owner(context: &HashMap<String, String>) -> bool {
        match context.get("is_owner") {
            Some(v) => v == "true",
            None => {
                warn!("resource_owner constraint: missing 'is_owner' in context");
                false
            }
        }
    }
}

// ---------------------------------------------------------------------------
// TimeWindow
// ---------------------------------------------------------------------------

/// A recurring time window for constraining permission evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Start hour (inclusive), 0-23.
    pub start_hour: u8,
    /// End hour (exclusive), 0-23. If `end_hour < start_hour`, the window
    /// wraps past midnight.
    pub end_hour: u8,
    /// IANA timezone identifier (informational — actual comparison uses the
    /// `hour` value from the context map).
    pub timezone: String,
    /// Active weekday names (e.g. `["monday", "tuesday", ...]`).
    /// An empty list means every day.
    pub days: Vec<String>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// CIDR match helper based on `ipnet` parsing.
///
/// Returns an error when the CIDR is malformed.
fn ip_in_cidr(ip: IpAddr, cidr: &str) -> Result<bool, ipnet::AddrParseError> {
    let network: IpNet = cidr.parse()?;
    Ok(network.contains(&ip))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // -- TimeWindow --------------------------------------------------------

    #[test]
    fn time_window_within_hours() {
        let tw = TimeWindow {
            start_hour: 9,
            end_hour: 17,
            timezone: "UTC".to_string(),
            days: vec![],
        };
        let constraints = PolicyConstraints {
            time_window: Some(tw),
            ip_ranges: None,
            resource_owner: None,
        };
        assert!(constraints.evaluate(&ctx(&[("hour", "10"), ("day", "monday")])));
    }

    #[test]
    fn time_window_outside_hours() {
        let tw = TimeWindow {
            start_hour: 9,
            end_hour: 17,
            timezone: "UTC".to_string(),
            days: vec![],
        };
        let constraints = PolicyConstraints {
            time_window: Some(tw),
            ip_ranges: None,
            resource_owner: None,
        };
        assert!(!constraints.evaluate(&ctx(&[("hour", "20"), ("day", "monday")])));
    }

    #[test]
    fn time_window_overnight_range() {
        let tw = TimeWindow {
            start_hour: 22,
            end_hour: 6,
            timezone: "UTC".to_string(),
            days: vec![],
        };
        let constraints = PolicyConstraints {
            time_window: Some(tw),
            ip_ranges: None,
            resource_owner: None,
        };
        // 23:00 is within 22-6
        assert!(constraints.evaluate(&ctx(&[("hour", "23")])));
        // 3:00 is within 22-6
        assert!(constraints.evaluate(&ctx(&[("hour", "3")])));
        // 10:00 is outside 22-6
        assert!(!constraints.evaluate(&ctx(&[("hour", "10")])));
    }

    #[test]
    fn time_window_day_filter() {
        let tw = TimeWindow {
            start_hour: 0,
            end_hour: 23,
            timezone: "UTC".to_string(),
            days: vec!["monday".to_string(), "wednesday".to_string()],
        };
        let constraints = PolicyConstraints {
            time_window: Some(tw),
            ip_ranges: None,
            resource_owner: None,
        };
        assert!(constraints.evaluate(&ctx(&[("hour", "10"), ("day", "monday")])));
        assert!(!constraints.evaluate(&ctx(&[("hour", "10"), ("day", "tuesday")])));
    }

    #[test]
    fn time_window_missing_hour() {
        let tw = TimeWindow {
            start_hour: 9,
            end_hour: 17,
            timezone: "UTC".to_string(),
            days: vec![],
        };
        let constraints = PolicyConstraints {
            time_window: Some(tw),
            ip_ranges: None,
            resource_owner: None,
        };
        // No "hour" in context => deny
        assert!(!constraints.evaluate(&ctx(&[])));
    }

    // -- IP ranges ---------------------------------------------------------

    #[test]
    fn ip_range_match() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: Some(vec!["10.0.0.0/8".to_string()]),
            resource_owner: None,
        };
        assert!(constraints.evaluate(&ctx(&[("ip", "10.1.2.3")])));
        assert!(!constraints.evaluate(&ctx(&[("ip", "192.168.1.1")])));
    }

    #[test]
    fn ip_range_multiple() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: Some(vec![
                "10.0.0.0/8".to_string(),
                "192.168.0.0/16".to_string(),
            ]),
            resource_owner: None,
        };
        assert!(constraints.evaluate(&ctx(&[("ip", "10.1.2.3")])));
        assert!(constraints.evaluate(&ctx(&[("ip", "192.168.1.1")])));
        assert!(!constraints.evaluate(&ctx(&[("ip", "172.16.0.1")])));
    }

    #[test]
    fn ip_range_missing_ip() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: Some(vec!["10.0.0.0/8".to_string()]),
            resource_owner: None,
        };
        assert!(!constraints.evaluate(&ctx(&[])));
    }

    // -- Resource owner ----------------------------------------------------

    #[test]
    fn resource_owner_required_and_is_owner() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: None,
            resource_owner: Some(true),
        };
        assert!(constraints.evaluate(&ctx(&[("is_owner", "true")])));
    }

    #[test]
    fn resource_owner_required_but_not_owner() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: None,
            resource_owner: Some(true),
        };
        assert!(!constraints.evaluate(&ctx(&[("is_owner", "false")])));
    }

    #[test]
    fn resource_owner_not_required() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: None,
            resource_owner: Some(false),
        };
        // resource_owner = false means no ownership check
        assert!(constraints.evaluate(&ctx(&[])));
    }

    #[test]
    fn resource_owner_missing_context() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: None,
            resource_owner: Some(true),
        };
        assert!(!constraints.evaluate(&ctx(&[])));
    }

    // -- Combined ----------------------------------------------------------

    #[test]
    fn all_constraints_pass() {
        let constraints = PolicyConstraints {
            time_window: Some(TimeWindow {
                start_hour: 9,
                end_hour: 17,
                timezone: "UTC".to_string(),
                days: vec!["monday".to_string()],
            }),
            ip_ranges: Some(vec!["10.0.0.0/8".to_string()]),
            resource_owner: Some(true),
        };
        let context = ctx(&[
            ("hour", "10"),
            ("day", "monday"),
            ("ip", "10.1.2.3"),
            ("is_owner", "true"),
        ]);
        assert!(constraints.evaluate(&context));
    }

    #[test]
    fn one_constraint_fails() {
        let constraints = PolicyConstraints {
            time_window: Some(TimeWindow {
                start_hour: 9,
                end_hour: 17,
                timezone: "UTC".to_string(),
                days: vec![],
            }),
            ip_ranges: Some(vec!["10.0.0.0/8".to_string()]),
            resource_owner: None,
        };
        // IP doesn't match
        let context = ctx(&[("hour", "10"), ("ip", "192.168.1.1")]);
        assert!(!constraints.evaluate(&context));
    }

    #[test]
    fn no_constraints_always_passes() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: None,
            resource_owner: None,
        };
        assert!(constraints.evaluate(&ctx(&[])));
    }

    // -- IP range edge cases -----------------------------------------------

    #[test]
    fn ip_range_overlapping_cidrs() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: Some(vec![
                "10.0.0.0/8".to_string(),
                "10.1.0.0/16".to_string(),
            ]),
            resource_owner: None,
        };
        assert!(constraints.evaluate(&ctx(&[("ip", "10.1.2.3")])));
        assert!(!constraints.evaluate(&ctx(&[("ip", "11.1.2.3")])));
    }

    #[test]
    fn ip_range_ipv6_support() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: Some(vec!["2001:db8::/32".to_string()]),
            resource_owner: None,
        };
        assert!(constraints.evaluate(&ctx(&[("ip", "2001:db8::1")])));
        assert!(!constraints.evaluate(&ctx(&[("ip", "2001:db9::1")])));
    }

    #[test]
    fn ip_range_malformed_cidr_denies() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: Some(vec![
                "10.0.0.0/8".to_string(),
                "not-a-cidr".to_string(),
            ]),
            resource_owner: None,
        };

        assert!(!constraints.evaluate(&ctx(&[("ip", "10.1.2.3")])));
    }

    #[test]
    fn ip_range_boundary_addresses_ipv4() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: Some(vec!["192.168.1.0/24".to_string()]),
            resource_owner: None,
        };

        assert!(constraints.evaluate(&ctx(&[("ip", "192.168.1.0")])));
        assert!(constraints.evaluate(&ctx(&[("ip", "192.168.1.255")])));
        assert!(constraints.evaluate(&ctx(&[("ip", "192.168.1.42")])));
        assert!(!constraints.evaluate(&ctx(&[("ip", "192.168.2.1")])));
    }

    #[test]
    fn ip_range_boundary_addresses_ipv6() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: Some(vec!["2001:db8::/126".to_string()]),
            resource_owner: None,
        };

        assert!(constraints.evaluate(&ctx(&[("ip", "2001:db8::")])));
        assert!(constraints.evaluate(&ctx(&[("ip", "2001:db8::1")])));
        assert!(constraints.evaluate(&ctx(&[("ip", "2001:db8::3")])));
        assert!(!constraints.evaluate(&ctx(&[("ip", "2001:db8::4")])));
    }

    #[test]
    fn ip_range_invalid_request_ip_denies() {
        let constraints = PolicyConstraints {
            time_window: None,
            ip_ranges: Some(vec!["10.0.0.0/8".to_string()]),
            resource_owner: None,
        };

        assert!(!constraints.evaluate(&ctx(&[("ip", "999.999.999.999")])));
    }

    #[test]
    fn serde_roundtrip() {
        let constraints = PolicyConstraints {
            time_window: Some(TimeWindow {
                start_hour: 9,
                end_hour: 17,
                timezone: "America/New_York".to_string(),
                days: vec!["monday".to_string(), "friday".to_string()],
            }),
            ip_ranges: Some(vec!["10.0.0.0/8".to_string()]),
            resource_owner: Some(true),
        };
        let json = serde_json::to_string(&constraints).unwrap();
        let back: PolicyConstraints = serde_json::from_str(&json).unwrap();
        assert_eq!(back.time_window.as_ref().unwrap().start_hour, 9);
        assert!(back.resource_owner.unwrap());
    }
}
