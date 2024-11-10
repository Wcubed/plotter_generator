use std::{fs, num::NonZeroUsize, ops, str::FromStr};

use camino::Utf8PathBuf;
use chrono::Local;
use clap::Parser;
use color_eyre::{eyre::Context, Result};
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use svg::{
    node::element::{path::Data, Path},
    Document,
};

const OUTPUT_DIR: &str = "output";

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Amount of iterations on the hilbert curve.
    #[arg(short, default_value_t = 5)]
    iterations: usize,
}

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

    let args = Args::parse();

    let output_dir = Utf8PathBuf::from_str(OUTPUT_DIR)?;
    if !output_dir.exists() {
        fs::create_dir(&output_dir)?;
    }

    let size = vec2(100.0, 100.0);

    let document = Document::new()
        .set("viewBox", (0.0, 0.0, size.x, size.y))
        .add(hilbert_curve_path(size, args.iterations));

    let local_time = Local::now();
    let timestamp = local_time.format("%Y-%m-%d_%H-%M-%S");

    let output_file = output_dir.join(format!("output_{}.svg", timestamp));
    svg::save(&output_file, &document)
        .wrap_err_with(|| format!("Could not save as `{output_file}`"))?;

    Ok(())
}

fn hilbert_curve_path(size: Vec2, iterations: usize) -> Path {
    let points = hilbert_curve(
        vec2(0.0, 0.0),
        vec2(size.x, 0.0),
        vec2(0.0, size.y),
        iterations,
    );

    let mut data = Data::new();

    for (index, point) in points.iter().enumerate() {
        if index == 0 {
            data = data.move_to((point.x, point.y));
        } else {
            data = data.line_to((point.x, point.y));
        }
    }

    Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", "1")
        .set("d", data)
}

/// Algorithm taken from https://www.fundza.com/algorithmic/space_filling/hilbert/basics/
fn hilbert_curve(p: Vec2, x_vec: Vec2, y_vec: Vec2, n: usize) -> Vec<Vec2> {
    let half_x = x_vec / 2.0;
    let half_y = y_vec / 2.0;

    if n == 0 {
        vec![p + half_x + half_y]
    } else {
        let mut output = vec![];
        output.append(&mut hilbert_curve(p, half_y, half_x, n - 1));
        output.append(&mut hilbert_curve(p + half_x, half_x, half_y, n - 1));
        output.append(&mut hilbert_curve(
            p + half_x + half_y,
            half_x,
            half_y,
            n - 1,
        ));
        output.append(&mut hilbert_curve(
            p + half_x + y_vec,
            -half_y,
            -half_x,
            n - 1,
        ));

        output
    }
}

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl ops::Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl ops::Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}
