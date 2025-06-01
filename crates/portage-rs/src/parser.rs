use winnow::{
	ModalResult, Parser,
	ascii::{alpha1, digit1},
	combinator::{alt, opt, repeat},
	error::StrContext,
	token::take_while,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSpecifier {
	pub version_selector: Option<String>,
	pub category: String,
	pub name: String,
	pub version: Option<String>,
}

impl From<&str> for PackageSpecifier {
	fn from(input: &str) -> Self {
		let mut input = input;
		package_specifier(&mut input).unwrap_or_else(|err| {
			panic!("Failed to parse package specifier: {err}: {input}");
		})
	}
}

impl std::fmt::Display for PackageSpecifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.version_selector.is_some() {
			write!(f, "{}-", self.version_selector.as_ref().unwrap())?;
		}

		write!(f, "{}/{}", self.category, self.name)?;
		if let Some(ref version) = self.version {
			write!(f, "-{version}")?;
		}
		Ok(())
	}
}

/// Parse a package version
pub fn package_version(input: &mut &str) -> ModalResult<String> {
	// A version starts with the number part, which is in the form [0-9]+(\.[0-9]+)* (an unsigned integer, followed by zero or more dot-prefixed unsigned integers).
	// This may optionally be followed by one of [a-z] (a lower-case letter).
	// This may be followed by zero or more of the suffixes _alpha, _beta, _pre, _rc or _p, each of which may optionally be followed by an unsigned integer.
	// Suffix and integer count as separate version components.
	// This may optionally be followed by the suffix -r followed immediately by an unsigned integer (the “revision number”). If this suffix is not present, it is assumed to be -r0.

	(
		digit1,
		repeat(0.., (".", digit1).map(|(_, num)| num)),
		opt(alpha1),
		repeat(
			0..,
			alt(("_alpha", "_beta", "_pre", "_rc", "_p"))
				.context(StrContext::Label("suffix")),
		)
		.context(StrContext::Label("suffixes")),
		repeat(0.., ("-r", digit1).map(|(_prefix, num)| num)),
	)
		.context(StrContext::Label("package version"))
		.map(
			|(major, minor, letter, suffixes, revision): (
				&str,
				Vec<&str>,
				Option<&str>,
				Vec<&str>,
				Vec<&str>,
			)| {
				let mut version = major.to_string();

				if !minor.is_empty() {
					version.push('.');
					version.push_str(&minor.join("."));
				}

				if let Some(letter) = letter {
					version.push_str(letter);
				}

				for suffix in suffixes {
					version.push_str(suffix);
				}

				for rev in revision {
					version.push_str(&format!("-r{rev}"));
				}

				version
			},
		)
		.parse_next(input)
}

/// Parse a package specifier
pub fn package_specifier(input: &mut &str) -> ModalResult<PackageSpecifier> {
	/*
	3.2 Version specifications
	The package manager must neither impose fixed limits upon the number of version components, nor upon the length of any component. Package managers should indicate or reject any version that is invalid according to the rules below.
	A version starts with the number part, which is in the form [0-9]+(\.[0-9]+)* (an unsigned integer, followed by zero or more dot-prefixed unsigned integers).
	This may optionally be followed by one of [a-z] (a lower-case letter).
	This may be followed by zero or more of the suffixes _alpha, _beta, _pre, _rc or _p, each of which may optionally be followed by an unsigned integer. Suffix and integer count as separate version components.
	This may optionally be followed by the suffix -r followed immediately by an unsigned integer (the “revision number”). If this suffix is not present, it is assumed to be -r0.
	*/

	// A package specifier is in the form <category>/<name>[-<version>]
	// where <category> is a sequence of alphanumeric characters and underscores, <name> is a sequence of alphanumeric characters and underscores, and <version> is an optional version string.
	(
		opt(alt((">", "<", ">=", "<=", "=", "==", "~=")))
			.context(StrContext::Label("version selector"))
			.map(|s: Option<&str>| s.map(|s| s.to_string())),
		(
			take_while(1.., |c: char| {
				c.is_ascii_alphanumeric()
					|| c == '.' || c == '_'
					|| c == '-' || c == '+'
			})
			.context(StrContext::Label("name")),
			opt(take_while(0.., |c: char| c == '-' || c == '+')
				.context(StrContext::Label("name suffix"))),
		)
			.map(|(name, suffix): (&str, Option<&str>)| {
				let mut name = name.to_string();
				if let Some(suffix) = suffix {
					name.push_str(suffix);
				}
				name
			})
			.context(StrContext::Label("category")),
		"/",
		// alpha-numeric characters, underscores, hyphens, and pluses, but only hyphens or plususes in the middle
		(
			take_while(1.., |c: char| {
				c.is_ascii_alphanumeric()
					|| c == '.' || c == '_'
					|| c == '-' || c == '+'
			})
			.context(StrContext::Label("name")),
			opt(take_while(0.., |c: char| c == '-' || c == '+')
				.context(StrContext::Label("name suffix"))),
		)
			.map(|(name, suffix): (&str, Option<&str>)| {
				let mut name = name.to_string();
				if let Some(suffix) = suffix {
					name.push_str(suffix);
				}
				name
			})
			.context(StrContext::Label("name")),
		opt(("-", package_version))
			.context(StrContext::Label("version"))
			.map(|v| v.map(|(_, ver)| ver)),
	)
		.map(|(version_selector, category, _, name, version)| {
			PackageSpecifier {
				version_selector,
				category: category.to_string(),
				name: name.to_string(),
				version,
			}
		})
		.parse_next(input)
}
