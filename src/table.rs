// originally from https://code.hinaria.com/p/arya
//
// > **Warning**
// > the order of variants in the `Token` and `CharacterType` enums **must** be kept in sync with the state transition table - we directly
// > cast `Token` and `CharacterType` variants into `usizes` to index into the transition table to find the next state transition.

use crate::Error;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterType {
	Space,        // space
	Whitespace,   // other whitespace
	BraceOpen,    // {
	BraceClose,   // }
	BracketOpen,  // [
	BracketClose, // ]
	Colon,        // :
	Comma,        // ,
	Quote,        // "
	Backslash,    // \
	Slash,        // /
	Plus,         // +
	Minus,        // -
	Dot,          // .
	Zero,         // 0
	Digit,        // 123456789
	LowA,         // a
	LowB,         // b
	LowC,         // c
	LowD,         // d
	LowE,         // e
	LowF,         // f
	LowL,         // l
	LowN,         // n
	LowR,         // r
	LowS,         // s
	LowT,         // t
	LowU,         // u
	Abcdf,        // ABCDF
	E,            // E
	Other,        // all other characters

	Error, // error-type. will never be returned / passed outside this module.
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
	Begin,      // <begin>
	Ok,         // <ok>
	Object,     // object
	Key,        // key
	Colon,      // colon
	Value,      // value
	Array,      // array
	String,     // string
	Escape,     // escape
	U1,         // u1
	U2,         // u2
	U3,         // u3
	U4,         // u4
	Minus,      // minus
	Zero,       // zero
	Integer,    // integer
	Fraction1,  // fraction 1
	Fraction2,  // fraction 2
	Exponent1,  // exponent 1
	Exponent2,  // exponent 2
	Exponent3,  // exponent 3
	TrueTr,     // tr
	TrueTru,    // tru
	TrueTrue,   // true
	FalseFa,    // fa
	FalseFal,   // fal
	FalseFals,  // fals
	FalseFalse, // false
	NullNu,     // nu
	NullNul,    // nul
	NullNull,   // null
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexToken {
	BraceEmptyClose, // } - empty brace
	BraceClose,      // }
	BracketClose,    // ]
	BraceOpen,       // {
	BracketOpen,     // [
	Quote,           // "
	Comma,           // ,
	Kolon,           // :
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
	Simple(Token),
	Complex(ComplexToken),

	Error, // error-type. will never be returned / passed outside this module.
}

#[rustfmt::skip]
const TRANSITIONS: [[Transition; 31]; 31] = {
    use self::{
        ComplexToken::{
            BraceClose, BraceEmptyClose, BraceOpen, BracketClose, BracketOpen, Comma, Kolon, Quote,
        },
        Token::{
            Array, Begin, Colon, Escape, Exponent1, Exponent2, Exponent3, FalseFa, FalseFal,
            FalseFals, FalseFalse, Fraction1, Fraction2, Integer, Key, Minus, NullNu, NullNul,
            NullNull, Object, Ok, String, TrueTr, TrueTru, TrueTrue, Value, Zero, U1, U2, U3, U4,
        },
        Transition::{Complex, Error, Simple},
    };

    [
        //                          <space>       <other-white-space>                          {                         }                         [                         ]                         :                         ,                         "                         \                         /                         +                         -                         .                         0               <123456789>                         a                         b                         c                         d                         e                         f                         l                         n                         r                         s                         t                         u                   <ABCDF>                         E                     <...>
        /* continue    */ [   Simple(Begin),            Simple(Begin),       Complex(BraceOpen),                    Error,     Complex(BracketOpen),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* ok          */ [      Simple(Ok),               Simple(Ok),                    Error,      Complex(BraceClose),                    Error,    Complex(BracketClose),                    Error,           Complex(Comma),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* object      */ [  Simple(Object),           Simple(Object),                    Error, Complex(BraceEmptyClose),                    Error,                    Error,                    Error,                    Error,           Simple(String),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* key         */ [     Simple(Key),              Simple(Key),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,           Simple(String),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* colon       */ [   Simple(Colon),            Simple(Colon),                    Error,                    Error,                    Error,                    Error,           Complex(Kolon),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* value       */ [   Simple(Value),            Simple(Value),       Complex(BraceOpen),                    Error,     Complex(BracketOpen),                    Error,                    Error,                    Error,           Simple(String),                    Error,                    Error,                    Error,            Simple(Minus),                    Error,             Simple(Zero),          Simple(Integer),                    Error,                    Error,                    Error,                    Error,                    Error,          Simple(FalseFa),                    Error,           Simple(NullNu),                    Error,                    Error,           Simple(TrueTr),                    Error,                    Error,                    Error,                    Error],
        /* array       */ [   Simple(Array),            Simple(Array),       Complex(BraceOpen),                    Error,     Complex(BracketOpen),    Complex(BracketClose),                    Error,                    Error,           Simple(String),                    Error,                    Error,                    Error,            Simple(Minus),                    Error,             Simple(Zero),          Simple(Integer),                    Error,                    Error,                    Error,                    Error,                    Error,          Simple(FalseFa),                    Error,           Simple(NullNu),                    Error,                    Error,           Simple(TrueTr),                    Error,                    Error,                    Error,                    Error],
        /* string      */ [  Simple(String),                    Error,           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Complex(Quote),           Simple(Escape),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String)],
        /* escape      */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,           Simple(String),           Simple(String),           Simple(String),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,           Simple(String),                    Error,                    Error,                    Error,           Simple(String),                    Error,           Simple(String),           Simple(String),                    Error,           Simple(String),               Simple(U1),                    Error,                    Error,                    Error],
        /* u1          */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,               Simple(U2),               Simple(U2),               Simple(U2),               Simple(U2),               Simple(U2),               Simple(U2),               Simple(U2),               Simple(U2),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,               Simple(U2),               Simple(U2),                    Error],
        /* u2          */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,               Simple(U3),               Simple(U3),               Simple(U3),               Simple(U3),               Simple(U3),               Simple(U3),               Simple(U3),               Simple(U3),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,               Simple(U3),               Simple(U3),                    Error],
        /* u3          */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,               Simple(U4),               Simple(U4),               Simple(U4),               Simple(U4),               Simple(U4),               Simple(U4),               Simple(U4),               Simple(U4),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,               Simple(U4),               Simple(U4),                    Error],
        /* u4          */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),           Simple(String),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,           Simple(String),           Simple(String),                    Error],
        /* minus       */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,             Simple(Zero),          Simple(Integer),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* zero        */ [      Simple(Ok),               Simple(Ok),                    Error,      Complex(BraceClose),                    Error,    Complex(BracketClose),                    Error,           Complex(Comma),                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Fraction1),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Exponent1),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Exponent1),                    Error],
        /* integer     */ [      Simple(Ok),               Simple(Ok),                    Error,      Complex(BraceClose),                    Error,    Complex(BracketClose),                    Error,           Complex(Comma),                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Fraction1),          Simple(Integer),          Simple(Integer),                    Error,                    Error,                    Error,                    Error,        Simple(Exponent1),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Exponent1),                    Error],
        /* fraction 1  */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Fraction2),        Simple(Fraction2),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* fraction 2  */ [      Simple(Ok),               Simple(Ok),                    Error,      Complex(BraceClose),                    Error,    Complex(BracketClose),                    Error,           Complex(Comma),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Fraction2),        Simple(Fraction2),                    Error,                    Error,                    Error,                    Error,        Simple(Exponent1),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Exponent1),                    Error],
        /* exponent 1  */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Exponent2),        Simple(Exponent2),                    Error,        Simple(Exponent3),        Simple(Exponent3),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* exponent 2  */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Exponent3),        Simple(Exponent3),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* exponent 3  */ [      Simple(Ok),               Simple(Ok),                    Error,      Complex(BraceClose),                    Error,    Complex(BracketClose),                    Error,           Complex(Comma),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(Exponent3),        Simple(Exponent3),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* true_tr     */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,          Simple(TrueTru),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* true_tru    */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,         Simple(TrueTrue),                    Error,                    Error,                    Error],
        /* true_true   */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,               Simple(Ok),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* false_fa    */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,         Simple(FalseFal),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* false_fal   */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,        Simple(FalseFals),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* false_fals  */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,       Simple(FalseFalse),                    Error,                    Error,                    Error,                    Error,                    Error],
        /* false_false */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,               Simple(Ok),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* null_nu     */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,          Simple(NullNul),                    Error,                    Error,                    Error],
        /* null_nul    */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,         Simple(NullNull),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
        /* null_null   */ [           Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,               Simple(Ok),                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error,                    Error],
    ]
};

#[rustfmt::skip]
const CATEGORIES: [CharacterType; 128] = {
    use self::CharacterType::{
        Abcdf, Backslash, BraceClose, BraceOpen, BracketClose, BracketOpen, Colon, Comma, Digit,
        Dot, Error, LowA, LowB, LowC, LowD, LowE, LowF, LowL, LowN, LowR, LowS, LowT, LowU, Minus,
        Other, Plus, Quote, Slash, Space, Whitespace, Zero, E,
    };

    [
        Error,        Error,        Error,        Error,        Error,        Error,        Error,        Error,
        Error,   Whitespace,   Whitespace,        Error,        Error,   Whitespace,        Error,        Error,
        Error,        Error,        Error,        Error,        Error,        Error,        Error,        Error,
        Error,        Error,        Error,        Error,        Error,        Error,        Error,        Error,

        Space,        Other,        Quote,        Other,        Other,        Other,        Other,        Other,
        Other,        Other,        Other,         Plus,        Comma,        Minus,          Dot,        Slash,
         Zero,        Digit,        Digit,        Digit,        Digit,        Digit,        Digit,        Digit,
        Digit,        Digit,        Colon,        Other,        Other,        Other,        Other,        Other,

        Other,        Abcdf,        Abcdf,        Abcdf,        Abcdf,            E,        Abcdf,        Other,
        Other,        Other,        Other,        Other,        Other,        Other,        Other,        Other,
        Other,        Other,        Other,        Other,        Other,        Other,        Other,        Other,
        Other,        Other,        Other,  BracketOpen,    Backslash, BracketClose,        Other,        Other,

        Other,         LowA,         LowB,         LowC,         LowD,         LowE,         LowF,        Other,
        Other,        Other,        Other,        Other,         LowL,        Other,         LowN,        Other,
        Other,        Other,         LowR,         LowS,         LowT,         LowU,        Other,        Other,
        Other,        Other,        Other,    BraceOpen,        Other,   BraceClose,        Other,        Other,
    ]
};

pub fn character_type(character: u8) -> Result<CharacterType, Error> {
	debug_assert!(character < 128);

	match CATEGORIES[character as usize] {
		CharacterType::Error => Err(Error::Invalid),
		character_type => Ok(character_type),
	}
}

pub fn transition(from: Token, ty: CharacterType) -> Result<Transition, Error> {
	debug_assert!(ty != CharacterType::Error);

	match TRANSITIONS[from as usize][ty as usize] {
		Transition::Error => Err(Error::Invalid),
		transition => Ok(transition),
	}
}
