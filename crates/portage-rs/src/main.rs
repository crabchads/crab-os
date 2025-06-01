use brush_core::{CreateOptions, Shell};
use clap::Parser as _;
use parser::PackageSpecifier;

pub mod parser;

#[derive(clap::Parser, Debug)]
#[command(multicall = true)]
pub enum Command {
	// emerge command
	Emerge {
		packages: Vec<PackageSpecifier>,

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
	let mut shell = Shell::new(&CreateOptions {
		..Default::default()
	})
	.await?;

	let args = Command::parse();
	dbg!(args);

	Ok(())
}
