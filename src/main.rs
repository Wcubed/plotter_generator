use std::{fs, str::FromStr};

use camino::Utf8PathBuf;
use chrono::{DateTime, Local};
use color_eyre::{eyre::Context, Result};
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
    if !output_dir.exists() {
        fs::create_dir(&output_dir)?;
    }

    let height = 70;
    let width = 70;
    let path_amount = 10;

    let mut document = Document::new().set("viewBox", (0, 0, width, height));

    let x_spacing = width / path_amount;

    for index in 0..path_amount {
        document = document.add(generate_path(index * x_spacing, height));
    }

    let local_time = Local::now();
    let timestamp = local_time.format("%Y-%m-%d_%H-%M-%S");

    let output_file = output_dir.join(format!("test_{}.svg", timestamp));
    svg::save(&output_file, &document)
        .wrap_err_with(|| format!("Could not save as `{output_file}`"))?;

    Ok(())
}

fn generate_path(x_start: i32, y_end: i32) -> Path {
    let data = Data::new()
        .move_to((x_start, 0))
        .line_to((x_start, y_end / 2 - 10))
        .line_to((x_start + 10, y_end / 2 + 10))
        .line_to((x_start + 10, y_end));

    Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", "1")
        .set("d", data)
}
