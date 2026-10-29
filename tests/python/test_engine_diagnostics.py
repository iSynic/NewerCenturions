import unittest

from tools.engine_diagnostics import scan


class DiagnosticOwnershipTests(unittest.TestCase):
    def test_import_ownership_and_parameterized_builders_preserve_distinct_errors(self):
        raw = {
            'crates/core/src/ammunition.rs': '''
                fn unsupported(dependency: &'static str) -> RuleError {
                    RuleError::Unsupported { action: "ammunition", dependency }
                }
            ''',
            'crates/core/src/equipment.rs': 'use record_rules::{self as rules, unsupported as record_rules, unsupported};',
            'crates/core/src/equipment/record_rules.rs': '''
                fn unsupported(dependency: &'static str) -> RuleError {
                    RuleError::Unsupported { action: "equipment", dependency }
                } pub use super::record_rules as again;
            ''',
            'crates/core/src/equipment/modifiers.rs': '''
                //! Item use for equipment.
                use super::{rules as current, record_rules::unsupported};
                fn run() { unsupported("global rules"); current::again::again::unsupported("module import"); }
            ''',
            'crates/core/src/equipment/transactions.rs': '''
                use super::{unsupported as fail};
                fn run() { fail("crate in combat"); }
            ''',
            'crates/core/src/payment.rs': '''
                fn unavailable(action: &'static str, dependency: &'static str) -> RuleError {
                    RuleError::Unsupported { action, dependency }
                }
                fn run() { unavailable("inventory.gold", "exhausted tokens"); }
                fn racial(action: &'static str) -> Result<(),RuleError> {
                    Err(RuleError::Unsupported { action, dependency: "racial mask" })
                }
                fn another() { racial("spell"); racial("support"); }
            ''',
        }
        pairs, unresolved = scan(raw)
        self.assertEqual(set(pairs), {
            ('equipment','global rules'), ('equipment','crate in combat'), ('equipment','module import'),
            ('inventory.gold','exhausted tokens'), ('spell','racial mask'), ('support','racial mask'),
        })
        self.assertFalse(unresolved)
