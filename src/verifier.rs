use crate::{
	table::{self, ComplexToken, Token, Transition},
	Builder, Error, Status,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueType {
	Key,
	Array,
	Object,
}

/// A fast JSON syntax validator for UTF-8 sequences.
///
/// # Remarks
///
/// This parser can continue even when an invalid character is attempted.
///
/// Invocations to `update()` only update the state and validity of this parser if and only if the input
/// sequence would result in a valid JSON object.
///
/// If the provided input would cause this JSON object to become invalid, `update()` will return an error but keep
/// its state, and the next invocation of `update()` will operate as if the bad character had never been applied.
///
/// In that sense, you can continue to "test" a character for JSON validity multiple times until one is found.
///
/// # Example
///
/// ```
/// # use repair_json::{Verifier, Status};
/// let mut verifier = Verifier::new();
///
/// for (i, char) in r#"{ "name": "annie", "value": 1 }"#.bytes().enumerate() {
///     verifier.update(char).unwrap();
///
///     if char == b'}' {
///         assert_eq!(verifier.status(), Status::Valid);
///     } else {
///         assert_eq!(verifier.status(), Status::Continue);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Verifier {
	maximum: usize,
	state: Token,
	nested_state: Vec<ValueType>,
	stack: Vec<(ValueType, usize)>,
	length: usize,
	last_ok: usize,
}

impl Verifier {
	/// Creates a new `Verifier` with the default maximum depth of [`std::usize::MAX`].
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates a new `Verifier` with the specified maximum depth.
	///
	/// # Panics
	///
	/// Panics if `maximum_depth` is `0`.
	#[must_use]
	pub fn with_maximum_depth(maximum_depth: usize) -> Self {
		assert!(maximum_depth > 0);

		Self {
			length: 0,
			last_ok: 0,
			stack: vec![],
			nested_state: vec![],
			state: Token::Begin,
			maximum: maximum_depth,
		}
	}

	#[must_use]
	/// Returns the current length of this JSON object.
	pub const fn len(&self) -> usize {
		self.length
	}

	#[must_use]
	/// Returns `true` if this JSON object is empty.
	pub const fn is_empty(&self) -> bool {
		self.length == 0
	}

	#[must_use]
	/// Returns the current status of this JSON object.
	pub fn status(&self) -> Status {
		if self.state == Token::Ok && self.nested_state.is_empty() {
			Status::Valid
		} else {
			Status::Continue
		}
	}

	/// Resets this JSON object to its initial state.
	pub fn reset(&mut self) {
		self.length = 0;
		self.last_ok = 0;
		self.state = Token::Begin;

		self.stack.clear();
		self.nested_state.clear();
	}

	/// Applies `character` to this JSON object.
	///
	/// # Remarks
	///
	/// If `character` would cause this JSON object to become invalid, this method returns an error, but keeps its state.
	/// The next invocation of `update()` will operate as if the bad character had never been applied.
	///
	/// # Errors
	///
	/// Returns an error if `character` is part of a valid UTF-8 sequence or if
	/// inserting `character` would cause this JSON object to become invalid.
	pub fn update(&mut self, character: u8) -> Result<(), Error> {
		// UTF-8 continuation.
		if character >= 128 {
			return self.state(self.state);
		}

		let character_type = table::character_type(character)?;
		let transition = table::transition(self.state, character_type)?;

		match transition {
			Transition::Error => {
				unreachable!("transition::error should never escape `table`.");
			},
			Transition::Simple(state) => self.state(state),
			Transition::Complex(ty) => match ty {
				ComplexToken::BraceEmptyClose => {
					self.pop(ValueType::Key)?;
					self.exit(ValueType::Object)?;
					self.state(Token::Ok)
				},
				ComplexToken::BraceClose => {
					self.pop(ValueType::Object)?;
					self.exit(ValueType::Object)?;
					self.state(Token::Ok)
				},
				ComplexToken::BracketClose => {
					self.pop(ValueType::Array)?;
					self.exit(ValueType::Array)?;
					self.state(Token::Ok)
				},
				ComplexToken::BraceOpen => {
					self.push(ValueType::Key)?;
					self.enter(ValueType::Object)?;
					self.state(Token::Object)
				},
				ComplexToken::BracketOpen => {
					self.push(ValueType::Array)?;
					self.enter(ValueType::Array)?;
					self.state(Token::Array)
				},
				ComplexToken::Quote => match self.nested_state.last() {
					Some(ValueType::Key) => self.state(Token::Colon),
					Some(ValueType::Object | ValueType::Array) => self.state(Token::Ok),
					_ => Err(Error::Invalid),
				},
				ComplexToken::Comma => match self.nested_state.last() {
					Some(ValueType::Object) => {
						self.last_ok = self.length;
						self.switch(ValueType::Object, ValueType::Key)?;
						self.state(Token::Key)
					},
					Some(ValueType::Array) => self.state(Token::Value),
					_ => Err(Error::Invalid),
				},
				ComplexToken::Kolon => {
					self.switch(ValueType::Key, ValueType::Object)?;
					self.state(Token::Value)
				},
			},
		}
	}

	pub(crate) fn complete(&self) -> (Option<usize>, Vec<u8>) {
		let mut tokens = Vec::new();
		let mut last_ok = None;

		match self.state {
			Token::Integer => {},
			Token::NullNu => tokens.extend("ull".bytes()),
			Token::NullNul => tokens.extend("ll".bytes()),
			Token::NullNull => tokens.extend("l".bytes()),
			Token::TrueTr => tokens.extend("rue".bytes()),
			Token::TrueTru => tokens.extend("ue".bytes()),
			Token::FalseFa => tokens.extend("alse".bytes()),
			Token::FalseFal => tokens.extend("lse".bytes()),
			Token::FalseFals => tokens.extend("se".bytes()),
			Token::FalseFalse | Token::TrueTrue => tokens.push(b'e'),
			Token::String => {
				if self.nested_state.last() == Some(&ValueType::Key) {
					last_ok = Some(self.last_ok);
				} else {
					tokens.push(b'"');
				}
			},
			_ => last_ok = Some(self.last_ok),
		};

		tokens.extend(
			self.stack
				.iter()
				.filter(|(_, depth)| {
					let Some(last_ok) = last_ok else {
						return true;
					};

					last_ok == 0 || *depth < last_ok
				})
				.rev()
				.filter_map(|(ty, _)| match ty {
					ValueType::Key => None,
					ValueType::Array => Some(b']'),
					ValueType::Object => Some(b'}'),
				}),
		);

		(last_ok, tokens)
	}

	fn push(&mut self, ty: ValueType) -> Result<(), Error> {
		if self.nested_state.len() < self.maximum {
			self.nested_state.push(ty);
			Ok(())
		} else {
			Err(Error::Exceeded)
		}
	}

	fn enter(&mut self, ty: ValueType) -> Result<(), Error> {
		if self.stack.len() < self.maximum && ValueType::Key != ty {
			self.stack.push((ty, self.last_ok));
			return Ok(());
		}

		Err(Error::Invalid)
	}

	fn exit(&mut self, ty: ValueType) -> Result<(), Error> {
		let (pop_ty, _) = self.stack.pop().ok_or(Error::Invalid)?;

		if ty != pop_ty {
			return Err(Error::Invalid);
		}

		Ok(())
	}

	fn pop(&mut self, ty: ValueType) -> Result<(), Error> {
		if self.nested_state.pop() == Some(ty) {
			Ok(())
		} else {
			Err(Error::Invalid)
		}
	}

	fn switch(&mut self, from: ValueType, to: ValueType) -> Result<(), Error> {
		self.pop(from)?;
		self.push(to)?;
		Ok(())
	}

	#[allow(clippy::unnecessary_wraps)]
	fn state(&mut self, state: Token) -> Result<(), Error> {
		self.length += 1;

		if state == Token::Ok {
			self.last_ok = self.length;
		}

		if (state == Token::Object && self.state == Token::Value)
			|| (state == Token::Array && self.nested_state.last() == Some(&ValueType::Array))
		{
			self.last_ok = self.length;
		}

		self.state = state;

		Ok(())
	}
}

impl Default for Verifier {
	fn default() -> Self {
		Self::with_maximum_depth(std::usize::MAX)
	}
}

impl From<Verifier> for Builder {
	fn from(verifier: Verifier) -> Self {
		Self {
			verifier,
			..Default::default()
		}
	}
}
