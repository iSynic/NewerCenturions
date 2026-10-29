//! Assertions against independent original-instruction receipts.
use super::{NativeCase, NativeScalar, bytes, record_bytes};
use serde_json::Value;

#[track_caller]
pub fn expect<T: NativeScalar>(actual: T, expected: &Value) {
    assert_eq!(actual, T::from_native(expected));
}

#[track_caller]
pub fn expect_array<T: NativeScalar>(actual: &T, expected: &Value) {
    assert_eq!(actual, &T::from_native(expected));
}

pub fn native_value<T: NativeScalar>(_: &T, value: &Value) -> T {
    T::from_native(value)
}

pub fn buffer(fixture: &Value, value: &Value) -> Vec<u8> {
    let encoded = value.as_str().unwrap_or_else(|| {
        fixture["shared_rule_buffers"][value.text("shared_buffer_sha256")]
            .as_str()
            .unwrap()
    });
    let decoded = bytes(encoded);
    if value.is_object() {
        expect(decoded.len(), &value["byte_length"]);
        assert_eq!(
            centurions_content::digest(&decoded),
            value.text("shared_buffer_sha256")
        );
    }
    decoded
}

pub fn replay_rng(random: &mut centurions_core::RandomState, receipts: &[Value]) {
    for receipt in receipts {
        expect(random.bound(receipt.n::<i16>("bound")), &receipt["result"]);
    }
}

pub fn advance_rng<'a>(
    random: &mut centurions_core::RandomState,
    receipts: &mut impl Iterator<Item = &'a Value>,
    draws: u64,
) {
    while random.draws < draws {
        let receipt = receipts.next().unwrap();
        expect(random.bound(receipt.n::<i16>("bound")), &receipt["result"]);
    }
    assert_eq!(random.draws, draws);
}

pub fn compare_party(actual: &[centurions_content::Warrior], snapshot: &Value) {
    compare_party_named(actual, snapshot, "party");
}

pub fn compare_party_named(
    actual: &[centurions_content::Warrior],
    snapshot: &Value,
    context: &str,
) {
    assert_bytes(&record_bytes(actual), &bytes(&snapshot["party"]), context);
}

#[track_caller]
pub fn assert_rng(actual: &centurions_core::RandomState, seed: &Value, draws: &Value) {
    assert_eq!(
        actual.seed,
        seed.as_u64().unwrap() as u32,
        "native RNG seed"
    );
    assert_eq!(actual.draws, draws.as_u64().unwrap(), "native RNG draws");
}

macro_rules! native_expect {
    ($($kind:ident $actual:expr => $expected:expr, $context:expr;)*) => {
        $(crate::support::native_expect!(@$kind $actual, $expected, $context);)*
    };
    (@rng $actual:expr, $expected:expr, $context:expr) => {
        {
            let (seed, draws) = $expected;
            crate::support::native_expect!(@scalar $actual.seed, seed, $context);
            crate::support::native_expect!(@scalar $actual.draws, draws, $context);
        }
    };
    (@scalar $actual:expr, $expected:expr, $context:expr) => {
        { let actual = &$actual; assert_eq!(actual, &crate::support::native_value(actual, $expected), "{}", $context); }
    };
    (@records $actual:expr, $expected:expr, $context:expr) => {
        crate::support::native_expect!(@bytes crate::support::record_bytes($actual), $expected, $context)
    };
    (@bytes $actual:expr, $expected:expr, $context:expr) => {
        crate::support::assert_bytes(&$actual[..], &crate::support::bytes($expected), &$context)
    };
    (@positions $actual:expr, $expected:expr, $context:expr) => {
        assert_eq!(&$actual[..], &crate::support::positions($expected)[..], "{}", $context)
    };
    (@words $actual:expr, $expected:expr, $context:expr) => {
        assert_eq!(&$actual[..], &crate::support::words($expected)[..], "{}", $context)
    };
}
pub(crate) use native_expect;

/// Native attack receipts may append caller metadata after the two body IDs.
pub fn attack_calls(value: &Value) -> Vec<(i16, i16)> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|call| {
            (
                call[0].as_i64().unwrap() as i16,
                call[1].as_i64().unwrap() as i16,
            )
        })
        .collect()
}

/// Apply the sparse byte changes recorded by original-instruction receipts.
pub fn patch_bytes(raw: &mut [u8], changes: &Value) {
    for change in changes.as_array().unwrap() {
        raw[change[0].as_u64().unwrap() as usize] = change[1].as_u64().unwrap() as u8;
    }
}

pub fn assert_bytes(actual: &[u8], expected: &[u8], context: &str) {
    assert_eq!(actual.len(), expected.len(), "{context}: length");
    let differing = actual.iter().zip(expected).position(|(a, b)| a != b);
    assert_eq!(
        differing.map(|i| (i, actual[i], expected[i])),
        None,
        "{context}: first differing byte"
    );
}
