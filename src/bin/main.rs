use clap::Parser;
use deez::{standard::StandardNotation, Notation, Roll};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output only the result total
    #[arg(short, long, default_value_t = false)]
    simple: bool,

    /// Dice rolls of the format [A]dB[RET][MOD]
    rolls: Vec<String>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    for input in args.rolls {
        let rolls = StandardNotation::parse_from_str(&input)?;

        for mut r in rolls {
            if args.simple {
                println!("{}", r.roll().total);
            } else {
                println!("{}", r.roll());
            }
        }
    }

    Ok(())
}
