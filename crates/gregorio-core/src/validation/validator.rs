//! Document validator: orchestrates rules and aggregates errors.

use super::rules::{all_validation_rules, ValidationRule};
use crate::parser::types::{ParseError, ParsedDocument};

pub struct DocumentValidator {
    rules: Vec<&'static ValidationRule>,
}

impl Default for DocumentValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentValidator {
    pub fn new() -> Self {
        Self {
            rules: all_validation_rules(),
        }
    }

    pub fn with_rules(rules: Vec<&'static ValidationRule>) -> Self {
        Self { rules }
    }

    pub fn validate(&self, doc: &ParsedDocument) -> Vec<ParseError> {
        let mut errors = doc.errors.clone();
        for rule in &self.rules {
            errors.extend((rule.validate)(doc));
        }
        errors
    }

    pub fn set_rule_enabled(&mut self, name: &str, enabled: bool) {
        if !enabled {
            self.rules.retain(|r| r.name != name);
        } else if !self.rules.iter().any(|r| r.name == name) {
            if let Some(rule) = all_validation_rules().into_iter().find(|r| r.name == name) {
                self.rules.push(rule);
            }
        }
    }

    pub fn available_rules(&self) -> Vec<&'static str> {
        all_validation_rules().iter().map(|r| r.name).collect()
    }
}
