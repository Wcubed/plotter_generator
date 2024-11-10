use std::{fs, str::FromStr};

use camino::Utf8PathBuf;
use chrono::Local;
use clap::{Parser, Subcommand};
use color_eyre::{eyre::Context, Result};
use itertools::Itertools;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use svg::{
    node::element::{path::Data, Path},
    Document,
};
use vec::{vec2, Vec2};

mod vec;

const OUTPUT_DIR: &str = "output";

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Width of the canvas.
    #[arg(short, long, default_value_t = 100.0)]
    width: f32,
    /// Height of the canvas.
    #[arg(short = 'H', long, default_value_t = 100.0)]
    height: f32,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Hilbert curve with 2 wonky offset lines.
    WonkyHilbert {
        /// Amount of iterations on the hilbert curve.
        #[arg(short, long, default_value_t = 5)]
        iterations: usize,

        /// Offset of the wonky lines.
        #[arg(short, long, default_value_t = 1.0)]
        offset: f32,
    },
    /// Hilbert curve.
    Hilbert {
        /// Amount of iterations on the hilbert curve.
        #[arg(short, long, default_value_t = 5)]
        iterations: usize,
    },
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

    let size = vec2(args.width, args.height);

    let mut document = Document::new().set("viewBox", (0.0, 0.0, size.x, size.y));

    match args.command {
        Commands::WonkyHilbert { iterations, offset } => {
            document = wonky_triple_hilbert_curve(document, size, iterations, offset)
        }
        Commands::Hilbert { iterations } => {
            document = hilbert_curve_path(document, size, iterations)
        }
    }

    let local_time = Local::now();
    let timestamp = local_time.format("%Y-%m-%d_%H-%M-%S");

    let output_file = output_dir.join(format!("output_{}.svg", timestamp));
    svg::save(&output_file, &document)
        .wrap_err_with(|| format!("Could not save as `{output_file}`"))?;

    Ok(())
}

fn wonky_triple_hilbert_curve(
    mut document: Document,
    size: Vec2,
    iterations: usize,
    offset: f32,
) -> Document {
    let points = hilbert_curve(
        vec2(0.0, 0.0),
        vec2(size.x, 0.0),
        vec2(0.0, size.y),
        iterations,
    );

    document = document.add(points_to_path(&points));

    let offset_points = wonky_offset_line(&points, offset);
    document = document.add(points_to_path(&offset_points));

    let offset_points = wonky_offset_line(&points, -offset);
    document = document.add(points_to_path(&offset_points));

    document
}

/// Creates a new line based on the original by calculating the points "inside"
/// the corners, and following that. Will cross over the original line if
/// the corners change direction.
fn wonky_offset_line(points: &[Vec2], amount: f32) -> Vec<Vec2> {
    let mut offset_points = vec![];

    for (&a, &b, &c) in points.iter().tuple_windows() {
        if let Some(direction) = direction_of_corner(a, b, c) {
            offset_points.push(b + direction * amount);
        }
    }

    offset_points
}

fn hilbert_curve_path(mut document: Document, size: Vec2, iterations: usize) -> Document {
    let points = hilbert_curve(
        vec2(0.0, 0.0),
        vec2(size.x, 0.0),
        vec2(0.0, size.y),
        iterations,
    );

    document = document.add(points_to_path(&points));

    let offset_points = offset_line(&points, 0.5);
    document = document.add(points_to_path(&offset_points));

    let offset_points = offset_line(&points, -0.5);
    document = document.add(points_to_path(&offset_points));

    document
}

/// Algorithm taken from https://stackoverflow.com/questions/68104969/offset-a-parallel-line-to-a-given-line-python
fn offset_line(points: &[Vec2], offset: f32) -> Vec<Vec2> {
    let mut offset_points = vec![];

    for (&a, &b, &c) in points.iter().tuple_windows() {
        let ab = (b - a).normalize();
        let bc = (c - b).normalize();

        let ab_90 = vec2(ab.y, -ab.x);
        let bc_90 = vec2(bc.y, -bc.x);

        let bisector = (ab_90 + bc_90).normalize();
        let length = offset / ((1.0 + ab_90.x * bc_90.x + ab_90.y * bc_90.y) / 2.0).sqrt();

        offset_points.push(b + bisector * length);
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
