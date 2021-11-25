use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use jcss::Predictor;

#[derive(Parser)]
struct Args {
    /// Custom inference model.
    #[clap(short, long, default_value = "model.onnx")]
    model: PathBuf,
    /// Must be of 40x100 size.
    image: PathBuf,
}

fn main() -> Result<()> {
    let args: Args = Args::parse();
    let predictor =
        Predictor::new(File::open(args.model).context("Failed to open inference model")?)
            .context("Failed to read inference model")?;
    let image = image::io::Reader::open(args.image)
        .context("Failed to read image")?
        .decode()
        .context("Failed to decode image.")?;

    println!("{}", predictor.predict(image)?);
    Ok(())
}
