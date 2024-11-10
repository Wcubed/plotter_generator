use std::{fs, str::FromStr};

use camino::Utf8PathBuf;
use chrono::Local;
use clap::Parser;
use color_eyre::{eyre::Context, Result};
use itertools::Itertools;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use svg::{
    node::element::{path::Data, Path},
    Document,
};
use vec::{vec2, Vec2};

mod vec;

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
            .add_filter_allow("plotter_generator".to_string())
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

    let mut document = Document::new().set("viewBox", (0.0, 0.0, size.x, size.y));

    document = hilbert_curve_path(document, size, args.iterations);

    let local_time = Local::now();
    let timestamp = local_time.format("%Y-%m-%d_%H-%M-%S");

    let output_file = output_dir.join(format!("output_{}.svg", timestamp));
    svg::save(&output_file, &document)
        .wrap_err_with(|| format!("Could not save as `{output_file}`"))?;

    Ok(())
}

fn hilbert_curve_path(mut document: Document, size: Vec2, iterations: usize) -> Document {
    let points = hilbert_curve(
        vec2(0.0, 0.0),
        vec2(size.x, 0.0),
        vec2(0.0, size.y),
        iterations,
    );

    document = document.add(points_to_path(&points));

    let offset = 1.0;

    let offset_points = offset_line(&points, offset);
    document = document.add(points_to_path(&offset_points));

    let offset_points = offset_line(&points, -offset);
    document = document.add(points_to_path(&offset_points));

    document
}

fn offset_line(points: &[Vec2], amount: f32) -> Vec<Vec2> {
    let mut offset_points = vec![];

    for (&a, &b, &c) in points.iter().tuple_windows() {
        if let Some(direction) = direction_of_corner(a, b, c) {
            offset_points.push(b + direction * amount);
        }
    }

    offset_points
}

fn points_to_path(points: &[Vec2]) -> Path {
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
        .set("stroke-width", "0.1")
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

/// Returns a unit vector pointing inwards from the corner (the shortest angle).
/// `b` is the "pointy bit" of the corner.
///
/// Returns `None` in the case of a straight line.
pub fn direction_of_corner(a: Vec2, b: Vec2, c: Vec2) -> Option<Vec2> {
    let ba = (a - b).normalize();
    let bc = (c - b).normalize();

    let dir = ba + bc;

    if dir == Vec2::ZERO {
        None
    } else {
        Some(dir.normalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn direction_of_corner_90_degrees() {
        let direction = direction_of_corner(vec2(0.0, 0.0), vec2(10.0, 0.0), vec2(10.0, 5.0));

        assert_eq!(direction, Some(vec2(-0.70710677, 0.70710677)));
    }

    #[test]
    fn direction_of_corner_180_degrees() {
        let direction = direction_of_corner(vec2(0.0, 0.0), vec2(10.0, 0.0), vec2(20.0, 0.0));

        assert_eq!(direction, None);
    }
}
