#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod builder;
mod table;
mod verifier;

pub use builder::Builder;
pub use verifier::Verifier;

use builder::Source;

/// The status of the JSON object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
	/// More input is needed to complete the JSON object.
	Continue,

	/// The JSON object is complete and valid.
	Valid,
}

/// Errors that can occur while parsing JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
	/// The input stream is not valid JSON.
	#[error("The input stream is not valid JSON.")]
	Invalid,

	/// Conversion to a string failed because the input stream is not valid a UTF-8 sequence.
	#[error(
		"Conversion to a string failed because the input stream is not valid a utf8 sequence."
	)]
	Utf8,

	/// The input stream contained an object exceeding the maximum specified depth.
	#[error("The input stream contained an object exceeding the maximum specified depth.")]
	Exceeded,
}

#[allow(clippy::needless_pass_by_value)]
/// Repairs the provided JSON object.
///
/// # Errors
///
/// Returns an error if the JSON object cannot be repaired.
pub fn repair<I: Source>(input: I) -> Result<String, Error> {
	let mut builder = Builder::new();
	builder.update(&input)?;

	builder.completed_string()
}

#[cfg(test)]
mod tests {
	use crate::repair;

	#[test]
	fn can_complete_empty_object() {
		assert_eq!(repair("{").unwrap(), "{}");
		assert_eq!(repair("{ ").unwrap(), "{}");
		assert_eq!(repair("{ }").unwrap(), "{ }");
	}

	#[test]
	fn ignores_incomplete_key() {
		assert_eq!(repair(r#"{ ""#).unwrap(), "{}");
		assert_eq!(repair(r#"{ "test"#).unwrap(), "{}");
		assert_eq!(repair(r#"{ "test":"#).unwrap(), "{}");
		assert_eq!(repair(r#"{ "test": ""#).unwrap(), r#"{ "test": ""}"#);

		assert_eq!(
			repair(r#"{ "hello": "world", "#).unwrap(),
			r#"{ "hello": "world"}"#
		);

		assert_eq!(
			repair(r#"{ "hello": "world", ""#).unwrap(),
			r#"{ "hello": "world"}"#
		);

		assert_eq!(
			repair(r#"{ "hello": "world", "test"#).unwrap(),
			r#"{ "hello": "world"}"#
		);

		assert_eq!(
			repair(r#"{ "hello": "world", "test":"#).unwrap(),
			r#"{ "hello": "world"}"#
		);

		assert_eq!(
			repair(r#"{ "hello": "world", "test": ""#).unwrap(),
			r#"{ "hello": "world", "test": ""}"#
		);
	}

	#[test]
	fn completes_incomplete_string_value() {
		assert_eq!(
			repair(r#"{ "hello": "world"#).unwrap(),
			r#"{ "hello": "world"}"#
		);

		assert_eq!(
			repair(r#"{ "hello": "world", "test": "te"#).unwrap(),
			r#"{ "hello": "world", "test": "te"}"#
		);
	}

	#[test]
	fn completes_incomplete_null() {
		assert_eq!(repair(r#"{ "test": n"#).unwrap(), r#"{ "test": null}"#);
		assert_eq!(repair(r#"{ "test": nu"#).unwrap(), r#"{ "test": null}"#);
		assert_eq!(repair(r#"{ "test": nul"#).unwrap(), r#"{ "test": null}"#);
		assert_eq!(repair(r#"{ "test": null"#).unwrap(), r#"{ "test": null}"#);
	}

	#[test]
	fn completes_incomplete_booleans() {
		assert_eq!(repair(r#"{ "test": t"#).unwrap(), r#"{ "test": true}"#);
		assert_eq!(repair(r#"{ "test": tr"#).unwrap(), r#"{ "test": true}"#);
		assert_eq!(repair(r#"{ "test": tru"#).unwrap(), r#"{ "test": true}"#);
		assert_eq!(repair(r#"{ "test": true"#).unwrap(), r#"{ "test": true}"#);

		assert_eq!(repair(r#"{ "test": f"#).unwrap(), r#"{ "test": false}"#);
		assert_eq!(repair(r#"{ "test": fa"#).unwrap(), r#"{ "test": false}"#);
		assert_eq!(repair(r#"{ "test": fal"#).unwrap(), r#"{ "test": false}"#);
		assert_eq!(repair(r#"{ "test": fals"#).unwrap(), r#"{ "test": false}"#);
		assert_eq!(repair(r#"{ "test": false"#).unwrap(), r#"{ "test": false}"#);
	}

	#[test]
	fn handles_escape_sequences() {
		assert_eq!(
			repair(r#"{ "hello": "world", "test": "he\"#).unwrap(),
			r#"{ "hello": "world"}"#
		);

		assert_eq!(
			repair(r#"{ "hello": "world", "test": "he\""#).unwrap(),
			r#"{ "hello": "world", "test": "he\""}"#
		);
	}

	#[test]
	fn properly_handles_arrays() {
		assert_eq!(repair(r#"{ "toys": ["#).unwrap(), r#"{ "toys": []}"#);
		assert_eq!(repair(r#"{ "toys": [""#).unwrap(), r#"{ "toys": [""]}"#);
		assert_eq!(
			repair(r#"{ "toys": ["test"#).unwrap(),
			r#"{ "toys": ["test"]}"#
		);
		assert_eq!(
			repair(r#"{ "toys": ["test", ""#).unwrap(),
			r#"{ "toys": ["test", ""]}"#
		);
	}

	#[test]
	fn properly_handles_objects() {
		assert_eq!(repair(r#"{ "user": {"#).unwrap(), r#"{ "user": {}}"#);
		assert_eq!(repair(r#"{ "user": {""#).unwrap(), r#"{ "user": {}}"#);
		assert_eq!(repair(r#"{ "user": {}"#).unwrap(), r#"{ "user": {}}"#);

		assert_eq!(repair(r#"{ "user": {"test"#).unwrap(), r#"{ "user": {}}"#);
		assert_eq!(
			repair(r#"{ "user": {"test": ""#).unwrap(),
			r#"{ "user": {"test": ""}}"#
		);
		assert_eq!(
			repair(r#"{ "user": {"name": "miguel"#).unwrap(),
			r#"{ "user": {"name": "miguel"}}"#
		);

		assert_eq!(
			repair(r#"{ "user": {"name": "miguel", "age": 21"#).unwrap(),
			r#"{ "user": {"name": "miguel", "age": 21}}"#
		);

		assert_eq!(
			repair(r#"{ "user": {"name": "miguel", "account": {"#).unwrap(),
			r#"{ "user": {"name": "miguel", "account": {}}}"#
		);
	}

	#[test]
	fn can_complete_mixed_example() {
		let full_json = r#"{ "users": [{ "id": 1, "name": "Miguel", "verified_at": null }, { "id": 2, "name": "Anne", "verified_at": 1234 }] }"#;

		assert_eq!(repair(full_json).unwrap(), full_json);
		assert_eq!(repair(r#"{ "users": [{"#).unwrap(), r#"{ "users": []}"#);
		assert_eq!(
			repair(r#"{ "users": [{ "id": 1"#).unwrap(),
			r#"{ "users": [{ "id": 1}]}"#
		);
		assert_eq!(
			repair(r#"{ "users": [{ "id": 1,"#).unwrap(),
			r#"{ "users": [{ "id": 1}]}"#
		);
		assert_eq!(
			repair(r#"{ "users": [{ "id": 1, "name": "Miguel"#).unwrap(),
			r#"{ "users": [{ "id": 1, "name": "Miguel"}]}"#
		);
		assert_eq!(
			repair(r#"{ "users": [{ "id": 1, "name": "Miguel", "verified_at":"#).unwrap(),
			r#"{ "users": [{ "id": 1, "name": "Miguel"}]}"#
		);
		assert_eq!(
			repair(r#"{ "users": [{ "id": 1, "name": "Miguel", "verified_at": n"#).unwrap(),
			r#"{ "users": [{ "id": 1, "name": "Miguel", "verified_at": null}]}"#
		);
		assert_eq!(
			repair(r#"{ "users": [{ "id": 1, "name": "Miguel", "verified_at": null }, "#).unwrap(),
			r#"{ "users": [{ "id": 1, "name": "Miguel", "verified_at": null }]}"#
		);
		assert_eq!(
			repair(r#"{ "users": [{ "id": 1, "name": "Miguel", "verified_at": null }, {"#).unwrap(),
			r#"{ "users": [{ "id": 1, "name": "Miguel", "verified_at": null }, {}]}"#
		);
	}
}
