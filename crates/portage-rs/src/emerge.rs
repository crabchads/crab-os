use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet};
use brush_core::{CreateOptions, Shell};
use clap::Parser as _;

use crate::parser::parse_package_specifier;

pub mod parser;

#[derive(clap::Parser, Debug)]
#[command(multicall = true)]
pub enum Command {
	// emerge command
	Emerge {
		packages: Vec<String>,

		#[clap(long, short = 'a')]
		ask: bool,
		#[clap(long, short = 'v')]
		verbose: bool,
		#[clap(long, short = 't')]
		tree: bool,
		#[clap(long, short = 'u')]
		update: bool,
		#[clap(long, short = 'D')]
		deep: bool,
		#[clap(long, short = 'N')]
		newuse: bool,
	},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let shell = Shell::new(&CreateOptions {
		..Default::default()
	})
	.await?;

	let command = Command::parse();

	match command {
		Command::Emerge { packages, .. } => {
			for package in &packages {
				let specifier =
					parse_package_specifier(package).map_err(|err| {
						let mut message = Group::with_title(
							Level::ERROR
								.title("Failed to parse package specifier"),
						);
						for (span, msg) in err {
							let snippet = Snippet::source(package).annotation(
								AnnotationKind::Primary
									.span(span.start..span.end)
									.label(msg),
							);

							message = message.element(snippet);
						}

						let renderer = Renderer::styled();
						anstream::eprintln!("{}", renderer.render(&[message]));
						std::process::exit(1);
					})?;
				println!("Parsed package specifier: {specifier}");
			}
		}
	}

	Ok(())
}
