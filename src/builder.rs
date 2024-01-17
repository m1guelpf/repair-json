use crate::{verifier::Verifier, Error, Status};

/// Expanded options for constructing a `Builder` instance.
#[derive(Debug)]
pub struct Options {
	pub maximum_depth: usize,
	pub initial_capacity: usize,
}

impl Default for Options {
	fn default() -> Self {
		Self {
			initial_capacity: 512,
			maximum_depth: std::usize::MAX,
		}
	}
}

/// A string builder for JSON that can repair and complete incomplete/damaged JSON.
///
/// # Remarks
///
/// Unlike the [`Verifier`], adding a sequence of characters that would make the underlying JSON object
/// invalid will cause the `Builder` to remain invalid, even if more characters are added to it later.
///
/// # Example
/// ```
/// # use repair_json::Builder;
/// let mut builder = Builder::new();
///
/// builder.update(&r#"{
///     "name": "miguel",
///     "age": 21,
///     "parents": {
///         "mother": null,
///         "broken
/// "#.trim());
/// builder.update(&"value");
///
/// assert_eq!(builder.completed_string(), Ok(r#"{
///     "name": "miguel",
///     "age": 21,
///     "parents": {
///         "mother": null}}
/// "#.trim().to_string()));
/// ```
#[derive(Debug, Default)]
pub struct Builder {
	pub(crate) data: Vec<u8>,
	pub(crate) invalid: bool,
	pub(crate) verifier: Verifier,
}

impl Builder {
	/// Creates a new `Builder` with the default maximum depth of [`std::usize::MAX`].
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates a new [`Builder`] with the specified maximum depth.
	#[must_use]
	pub fn with_maximum_depth(maximum_depth: usize) -> Self {
		Self::with_options(&Options {
			maximum_depth,
			..Default::default()
		})
	}

	/// Creates a new `Builder` with the specified initial capacity.
	#[must_use]
	pub fn with_capacity(initial_capacity: usize) -> Self {
		Self::with_options(&Options {
			initial_capacity,
			..Default::default()
		})
	}

	/// Creates a new `Builder` with the specified options.
	#[must_use]
	pub fn with_options(options: &Options) -> Self {
		Self {
			invalid: false,
			data: Vec::with_capacity(options.initial_capacity),
			verifier: Verifier::with_maximum_depth(options.maximum_depth),
		}
	}

	/// Returns the current length of this JSON object.
	#[must_use]
	pub fn len(&self) -> usize {
		self.data.len()
	}

	/// Returns `true` if this JSON object is empty.
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}

	/// Returns the current status of this JSON object.
	#[must_use]
	pub fn status(&self) -> Status {
		self.verifier.status()
	}

	/// Resets this JSON object to its initial state.
	pub fn reset(&mut self) {
		self.invalid = false;

		self.data.clear();
		self.verifier.reset();
	}

	/// Appends the provided source to this JSON object.
	///
	/// # Remarks
	///
	/// Unlike the [`Verifier`], adding a sequence of characters that would make the underlying JSON object
	/// invalid will cause the `Builder` to remain invalid, even if more characters are added to it later.
	///
	/// # Errors
	///
	/// Returns an error if adding the provided source would cause this JSON object to become invalid, or if
	/// this JSON object is already invalid.
	pub fn update(&mut self, source: &impl Source) -> Result<(), Error> {
		if self.invalid {
			Err(Error::Invalid)
		} else {
			for character in source.stream() {
				match self.verifier.update(*character) {
					Ok(()) => {
						self.data.push(*character);
					},
					Err(e) => {
						self.invalid = true;
						return Err(e);
					},
				}
			}

			Ok(())
		}
	}

	/// Returns the underlying byte stream, or an error if the JSON object is invalid.
	///
	/// # Errors
	///
	/// Returns an error if the JSON object is invalid.
	pub fn bytes(self) -> Result<Vec<u8>, Error> {
		if self.invalid {
			return Err(Error::Invalid);
		}

		Ok(self.data)
	}

	/// Returns the JSON object as a string, or an error if the JSON object is invalid.
	///
	/// # Errors
	///
	/// Returns an error if the JSON object is invalid or contains invalid UTF-8.
	pub fn string(self) -> Result<String, Error> {
		let data = self.bytes()?;

		String::from_utf8(data).map_err(|_| Error::Utf8)
	}

	/// Returns the completed JSON object as a byte stream.
	///
	/// # Errors
	///
	/// Returns an error if the JSON object is invalid.
	pub fn completed_bytes(mut self) -> Result<Vec<u8>, Error> {
		if self.invalid {
			Err(Error::Invalid)
		} else {
			if self.verifier.status() == Status::Continue {
				let (until, tokens) = self.verifier.complete();

				if let Some(until) = until {
					self.data.truncate(if until == 0 { 1 } else { until });
				}
				self.data.extend(tokens);
			}

			Ok(self.data)
		}
	}

	/// Returns the completed JSON object as a string.
	///
	/// # Errors
	///
	/// Returns an error if the JSON object is invalid or contains invalid UTF-8.
	pub fn completed_string(self) -> Result<String, Error> {
		let data = self.completed_bytes()?;

		String::from_utf8(data).map_err(|_| Error::Utf8)
	}
}

/// A source of bytes.
pub trait Source {
	fn stream(&self) -> &[u8];
}

impl Source for u8 {
	fn stream(&self) -> &[u8] {
		// safety: the memory layout of a singular `T` is always the same as an array of one `T`.
		unsafe { std::slice::from_raw_parts(self, 1) }
	}
}

impl Source for &[u8] {
	fn stream(&self) -> &[u8] {
		self
	}
}

impl Source for Vec<u8> {
	fn stream(&self) -> &[u8] {
		self
	}
}

impl Source for &str {
	fn stream(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl Source for String {
	fn stream(&self) -> &[u8] {
		self.as_bytes()
	}
}
