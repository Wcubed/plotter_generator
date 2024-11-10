use std::{fs, str::FromStr};

use camino::Utf8PathBuf;
use color_eyre::Result;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use svg::{
    node::element::{path::Data, Path},
    Document,
};

const OUTPUT_DIR: &str = "output";

fn main() -> Result<()> {
    color_eyre::install()?;
    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::default()
            .add_filter_allow("web_scraping".to_string())
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    let output_dir = Utf8PathBuf::from_str(OUTPUT_DIR)?;
    fs::create_dir(&output_dir)?;

    let data = Data::new().move_to((10, 10)).line_to((10, 50));

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", "3")
        .set("d", data);

    let document = Document::new().set("viewBox", (0, 0, 70, 70)).add(path);

    let output_file = output_dir.join("test.svg");
    svg::save(output_file, &document)?;

    Ok(())
}
