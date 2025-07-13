use logos::{Logos, Span as LogosSpan};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum PackageVersionToken {
	#[regex(r"[0-9]+(\.[0-9]+)*")]
	Number,

	#[regex(r"[a-z]")]
	Letter,

	#[token("_alpha")]
	AlphaSuffix,

	#[token("_beta")]
	BetaSuffix,

	#[token("_pre")]
	PreSuffix,

	#[token("_rc")]
	RcSuffix,

	#[token("_p")]
	PatchSuffix,

	#[token("-r")]
	RevisionPrefix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
	pub start: usize,
	pub end: usize,
}

impl From<LogosSpan> for Span {
	fn from(s: LogosSpan) -> Self {
		Span {
			start: s.start,
			end: s.end,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSpecifier {
	pub version_selector: Option<(String, Span)>,
	pub category: (String, Span),
	pub name: (String, Span),
	pub version: Option<(String, Span)>,
}

impl std::fmt::Display for PackageSpecifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some((ref sel, _)) = self.version_selector {
			write!(f, "{sel}-")?;
		}
		write!(f, "{}/{}", self.category.0, self.name.0)?;
		if let Some((ref version, _)) = self.version {
			write!(f, "-{version}")?;
		}
		Ok(())
	}
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
#[logos(skip r"[ \t\n\f]+")]
pub enum SpecToken {
	// Version selectors
	#[token(">=")]
	Ge,
	#[token("<=")]
	Le,
	#[token(">")]
	Gt,
	#[token("<")]
	Lt,
	#[token("=")]
	Eq,
	#[token("==")]
	EqEq,
	#[token("~=")]
	TildeEq,

	// Category/Name
	#[regex(r"[A-Za-z0-9._+-]+", priority = 3)]
	Ident,

	#[token("/")]
	Slash,

	// Version
	#[regex(r"[0-9]+(\.[0-9]+)*")]
	Number,
	#[regex(r"[a-z]")]
	Letter,
	#[token("_alpha")]
	AlphaSuffix,
	#[token("_beta")]
	BetaSuffix,
	#[token("_pre")]
	PreSuffix,
	#[token("_rc")]
	RcSuffix,
	#[token("_p")]
	PatchSuffix,
	#[token("-r")]
	RevisionPrefix,

	#[token("-")]
	Dash,
}

pub fn parse_package_specifier(
	input: &str,
) -> Result<PackageSpecifier, Vec<(Span, String)>> {
	let mut lexer = SpecToken::lexer(input).spanned();
	let tokens: Vec<(SpecToken, std::ops::Range<usize>)> = lexer
		.by_ref()
		.filter_map(|(tok_res, span)| tok_res.ok().map(|tok| (tok, span)))
		.collect();
	let mut errors = Vec::new();
	let mut idx = 0;

	let peek = |i: usize, tokens: &Vec<(SpecToken, std::ops::Range<usize>)>| {
		tokens.get(i).map(|(tok, _)| *tok)
	};

	// Parse optional version selector
	let version_selector = match peek(idx, &tokens) {
		Some(
			SpecToken::Ge
			| SpecToken::Le
			| SpecToken::Gt
			| SpecToken::Lt
			| SpecToken::Eq
			| SpecToken::EqEq
			| SpecToken::TildeEq,
		) => {
			let (tok, span) = &tokens[idx];
			let s = match tok {
				SpecToken::Ge => ">=",
				SpecToken::Le => "<=",
				SpecToken::Gt => ">",
				SpecToken::Lt => "<",
				SpecToken::Eq => "=",
				SpecToken::EqEq => "==",
				SpecToken::TildeEq => "~=",
				_ => unreachable!(),
			};
			let span = Span {
				start: span.start,
				end: span.end,
			};
			idx += 1;
			Some((s.to_string(), span))
		}
		_ => None,
	};

	// Parse category
	let category = match peek(idx, &tokens) {
		Some(SpecToken::Ident) => {
			let (_tok, span) = &tokens[idx];
			let s = &input[span.clone()];
			let span = Span {
				start: span.start,
				end: span.end,
			};
			idx += 1;
			(s.to_string(), span)
		}
		Some(tok) => {
			let (_, span) = &tokens[idx];
			errors.push((
				Span {
					start: span.start,
					end: span.end,
				},
				format!("Expected category, found {tok:?}"),
			));
			return Err(errors);
		}
		None => {
			errors.push((
				Span { start: 0, end: 0 },
				"Unexpected end, expected category".to_string(),
			));
			return Err(errors);
		}
	};

	// Parse slash
	match peek(idx, &tokens) {
		Some(SpecToken::Slash) => {
			idx += 1;
		}
		Some(tok) => {
			let (_, span) = &tokens[idx];
			errors.push((
				Span {
					start: span.start,
					end: span.end,
				},
				format!("Expected '/', found {tok:?}"),
			));
			return Err(errors);
		}
		None => {
			errors.push((
				Span { start: 0, end: 0 },
				"Unexpected end, expected '/'".to_string(),
			));
			return Err(errors);
		}
	}

	// Parse name
	let name = match peek(idx, &tokens) {
		Some(SpecToken::Ident) => {
			let (_, span) = &tokens[idx];
			let s = &input[span.clone()];
			let span = Span {
				start: span.start,
				end: span.end,
			};
			idx += 1;
			(s.to_string(), span)
		}
		Some(tok) => {
			let (_, span) = &tokens[idx];
			errors.push((
				Span {
					start: span.start,
					end: span.end,
				},
				format!("Expected name, found {tok:?}"),
			));
			return Err(errors);
		}
		None => {
			errors.push((
				Span { start: 0, end: 0 },
				"Unexpected end, expected name".to_string(),
			));
			return Err(errors);
		}
	};

	// Parse optional version
	let mut version: Option<(String, Span)> = None;
	if let Some(SpecToken::Dash) = peek(idx, &tokens) {
		let version_start = tokens[idx].1.start;
		idx += 1; // consume '-'
		let mut version_str = String::new();
		let mut version_end = version_start;
		while let Some(tok) = peek(idx, &tokens) {
			match tok {
				SpecToken::Number
				| SpecToken::Letter
				| SpecToken::AlphaSuffix
				| SpecToken::BetaSuffix
				| SpecToken::PreSuffix
				| SpecToken::RcSuffix
				| SpecToken::PatchSuffix
				| SpecToken::RevisionPrefix
				| SpecToken::Dash => {
					let (_, span) = &tokens[idx];
					version_str.push_str(&input[span.clone()]);
					version_end = span.end;
					idx += 1;
				}
				_ => break,
			}
		}

		if !version_str.is_empty() {
			version = Some((
				version_str,
				Span {
					start: version_start,
					end: version_end,
				},
			));
		}
	}
	if errors.is_empty() {
		Ok(PackageSpecifier {
			version_selector,
			category,
			name,
			version,
		})
	} else {
		Err(errors)
	}
}
