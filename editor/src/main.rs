// Crates ---------------------------------------------------------------------
extern crate rustc_serialize;
extern crate image;
extern crate toml;


// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::collections::HashSet;


// External Dependencies ------------------------------------------------------
use toml::Encoder;
use image::GenericImage;
use rustc_serialize::Encodable;


// PNG -> Map TOML Parser -----------------------------------------------------
fn main() {

    let img = image::open(&Path::new("map.png")).unwrap();

    let (bounds, paths) = find_paths(&img);
    let level = parse_paths(&bounds, paths);

    let mut e = Encoder::new();
    level.encode(&mut e).unwrap();

    let toml = toml::Value::Table(e.toml);
    if let Ok(mut file) = File::create("map.toml") {
        write!(file, "{}", toml).expect("Failed to write map toml.");
    }

}

fn parse_paths(bounds: &[i32; 4], paths: Vec<TracedPath>) -> Level {

    let mut level = Level::default();

    let (lw, lh) = ((bounds[2] - bounds[0]) / 2, (bounds[3] - bounds[1]) / 2);
    for p in paths {

        if p.typ == TracedPathType::Wall {

            // Split wall paths into line segments
            let mut lines = Vec::new();
            let (mut ox, mut oy, mut or, mut dr) = p.pixels[0];
            let (mut lx, mut ly, _, _) = p.pixels[0];
            for &(x, y, r, d) in &p.pixels {
                if r != or {
                    let (dx, dy) = (ox - lx, oy - ly);
                    if ((dx * dx + dy * dy) as f32).sqrt() > 0.0 {
                        let line = [ox as u32, oy as u32, lx as u32, ly as u32, r];
                        lines.push((line, d));
                    }
                    ox = lx;
                    oy = ly;
                    or = r;
                    dr = d;
                }
                lx = x;
                ly = y;
            }

            let (dx, dy) = (ox - lx, oy - ly);
            if ((dx * dx + dy * dy) as f32).sqrt() > 0.0 {
                let line = [ox as u32, oy as u32, lx as u32, ly as u32, or];
                lines.push((line, dr));
            }

            /*
            for (i, &(line, d)) in lines.iter().enumerate() {
                for (j, &(other, od)) in lines.iter().enumerate() {
                    if j != i && d == od {

                        // Overlap Start Start
                        if line[0] == other[0] && line[1] == other[1] {
                            println!("Overlapping start<>start");

                        // Overlap End End
                        } else if line[2] == other[2] && line[3] == other[3] {
                            println!("Overlapping end<>end");

                        // Overlap Start End
                        } else if line[0] == other[2] && line[1] == other[3] {
                            println!("Overlapping start<>end");

                        // Overlap End Start
                        } else if line[2] == other[0] && line[3] == other[1] {
                            println!("Overlapping end<>start");
                        }

                    }
                }
            }
            */

            for (l, _) in lines {

                let p = [
                    (l[0] as f32 - lw as f32),
                    (l[1] as f32 - lh as f32),
                    (l[2] as f32 - lw as f32),
                    (l[3] as f32 - lh as f32)
                ];

                let line = if p[0] == p[2] || p[1] == p[3] {
                    [
                        p[0].min(p[2]),
                        p[1].min(p[3]),
                        p[0].max(p[2]),
                        p[1].max(p[3])
                    ]

                } else {
                    p
                };

                level.walls.push(Wall {
                    line: line
                });

            }

        // Construct concave polygons from solids
        } else if p.typ == TracedPathType::Solid {

            let mut points = Vec::new();
            let (mut ox, mut oy, mut or, _) = p.pixels[0];
            let (mut lx, mut ly, _, _) = p.pixels[0];

            points.push(ox - lw);
            points.push(oy - lh);

            for &(x, y, r, _) in &p.pixels {
                if r != or {
                    let (dx, dy) = (ox - lx, oy - ly);
                    if ((dx * dx + dy * dy) as f32).sqrt() > 0.0 {
                        points.push(lx - lw);
                        points.push(ly - lh);
                    }
                    ox = lx;
                    oy = ly;
                    or = r;
                }
                lx = x;
                ly = y;
            }

            level.solids.push(points);

        // Calculate bounds and center of lights and spawns
        } else {

            let mut path_bounds = [
                bounds[2],
                bounds[3],
                bounds[0],
                bounds[1]
            ];
            for &(x, y, _, _) in &p.pixels {
                path_bounds[0] = cmp::min(path_bounds[0], x);
                path_bounds[1] = cmp::min(path_bounds[1], y);
                path_bounds[2] = cmp::max(path_bounds[2], x);
                path_bounds[3] = cmp::max(path_bounds[3], y);
            }

            let (w, h) = (path_bounds[2] - path_bounds[0], path_bounds[3] - path_bounds[1]);
            let (x, y) = (path_bounds[0] + w / 2, path_bounds[1] + h / 2);
            if p.typ == TracedPathType::Spawn {
                level.spawns.push(Spawn {
                    x: x as i32 - lw,
                    y: y as i32 - lh
                });

            } else if p.typ == TracedPathType::Light {
                level.lights.push(Light {
                    x: x as i32 - lw,
                    y: y as i32 - lh,
                    radius: (cmp::max(w, h) as f32 * (2.0f32).sqrt()).round() as i32
                });
            }

        }

    }

    level

}

fn find_paths(img: &image::DynamicImage) -> ([i32; 4], Vec<TracedPath>) {

    let (w, h) = img.dimensions();
    let mut paths = Vec::new();
    let mut bounds = [w as i32, h as i32, 0, 0];
    let mut pixel_usage: HashSet<(i32, i32)> = HashSet::new();

    for y in 0..h {
        for x in 0..w {

            // Ignores pixels which are already part of a path
            if !pixel_usage.contains(&(x as i32, y as i32)) {

                // Start tracing a new path
                let path_type = get_path_type(&img.get_pixel(x, y));
                if let Some(path_type) = path_type {


                    let mut px = x as i32;
                    let mut py = y as i32;

                    pixel_usage.insert((px, py));


                    let (w, h) = (w as i32, h as i32);
                    let mut pixels = Vec::new();
                    let mut first_pixel = true;
                    let mut lr = 0;
                    loop {

                        // Check top right bottom left first
                        let m = if is_valid_path_pixel(px, py - 1, &img, &pixel_usage, w, h, path_type) {
                            Some((px, py - 1, 0, Direction::Vertical))

                        } else if is_valid_path_pixel(px + 1, py, &img, &pixel_usage, w, h, path_type) {
                            Some((px + 1, py, 90, Direction::Horizontal))

                        } else if is_valid_path_pixel(px, py + 1, &img, &pixel_usage, w, h, path_type) {
                            Some((px, py + 1, 180, Direction::Vertical))

                        } else if is_valid_path_pixel(px - 1, py, &img, &pixel_usage, w, h, path_type) {
                            Some((px - 1, py, 270, Direction::Horizontal))

                        // Check topright bottomright bottomleft topleft second
                        } else if is_valid_path_pixel(px + 1, py - 1, &img, &pixel_usage, w, h, path_type) {
                            Some((px + 1, py - 1, 45, Direction::DiagonalOne))

                        } else if is_valid_path_pixel(px + 1, py + 1, &img, &pixel_usage, w, h, path_type) {
                            Some((px + 1, py + 1, 135, Direction::DiagonalTwo))

                        } else if is_valid_path_pixel(px - 1, py + 1, &img, &pixel_usage, w, h, path_type) {
                            Some((px - 1, py + 1, 225, Direction::DiagonalOne))

                        } else if is_valid_path_pixel(px - 1, py - 1, &img, &pixel_usage, w, h, path_type) {
                            Some((px - 1, py - 1, 315, Direction::DiagonalTwo))

                        } else {
                            None
                        };

                        if let Some((nx, ny, r, d)) = m {

                            bounds[0] = cmp::min(bounds[0], px);
                            bounds[1] = cmp::min(bounds[1], py);
                            bounds[2] = cmp::max(bounds[2], px);
                            bounds[3] = cmp::max(bounds[3], py);

                            if first_pixel {
                                first_pixel = false;
                                pixels.push((px, py, r, d));
                            }

                            pixel_usage.insert((nx, ny));
                            pixels.push((nx, ny, r, d));
                            lr = r;
                            px = nx;
                            py = ny;

                        } else {

                            // Merge with adjacent paths at the end
                            let m = if lr == 0 && is_potential_path_pixel(px, py - 1, &img, w, h, path_type) {
                                Some((px, py - 1, 0, Direction::Vertical))

                            } else if lr == 90 && is_potential_path_pixel(px + 1, py, &img, w, h, path_type) {
                                Some((px + 1, py, 90, Direction::Horizontal))

                            } else if lr == 180 && is_potential_path_pixel(px, py + 1, &img, w, h, path_type) {
                                Some((px, py + 1, 180, Direction::Vertical))

                            } else if lr == 270 && is_potential_path_pixel(px - 1, py, &img, w, h, path_type) {
                                Some((px - 1, py, 270, Direction::Horizontal))

                            // Check topright bottomright bottomleft topleft second
                            } else if lr == 45 && is_potential_path_pixel(px + 1, py - 1, &img, w, h, path_type) {
                                Some((px + 1, py - 1, 45, Direction::DiagonalOne))

                            } else if lr == 135 && is_potential_path_pixel(px + 1, py + 1, &img, w, h, path_type) {
                                Some((px + 1, py + 1, 135, Direction::DiagonalTwo))

                            } else if lr == 225 && is_potential_path_pixel(px - 1, py + 1, &img, w, h, path_type) {
                                Some((px - 1, py + 1, 225, Direction::DiagonalOne))

                            } else if lr == 315 && is_potential_path_pixel(px - 1, py - 1, &img, w, h, path_type) {
                                Some((px - 1, py - 1, 315, Direction::DiagonalTwo))

                            } else {
                                None
                            };

                            if let Some((nx, ny, r, d)) = m {
                                pixels.push((nx, ny, r, d));
                            }

                            // Merge with adjacent paths at the end
                            let (px, py, lr, _) = pixels[0];
                            let m = if lr == 180 && is_potential_path_pixel(px, py - 1, &img, w, h, path_type) {
                                Some((px, py - 1, 0, Direction::Vertical))

                            } else if lr == 270 && is_potential_path_pixel(px + 1, py, &img, w, h, path_type) {
                                Some((px + 1, py, 90, Direction::Horizontal))

                            } else if lr == 00 && is_potential_path_pixel(px, py + 1, &img, w, h, path_type) {
                                Some((px, py + 1, 180, Direction::Vertical))

                            } else if lr == 90 && is_potential_path_pixel(px - 1, py, &img, w, h, path_type) {
                                Some((px - 1, py, 270, Direction::Horizontal))

                            // Check topright bottomright bottomleft topleft second
                            } else if lr == 225 && is_potential_path_pixel(px + 1, py - 1, &img, w, h, path_type) {
                                Some((px + 1, py - 1, 45, Direction::DiagonalOne))

                            } else if lr == 315 && is_potential_path_pixel(px + 1, py + 1, &img, w, h, path_type) {
                                Some((px + 1, py + 1, 135, Direction::DiagonalTwo))

                            } else if lr == 45 && is_potential_path_pixel(px - 1, py + 1, &img, w, h, path_type) {
                                Some((px - 1, py + 1, 225, Direction::DiagonalOne))

                            } else if lr == 135 && is_potential_path_pixel(px - 1, py - 1, &img, w, h, path_type) {
                                Some((px - 1, py - 1, 315, Direction::DiagonalTwo))

                            } else {
                                None
                            };

                            if let Some((nx, ny, r, d)) = m {
                                pixels.insert(0, (nx, ny, r, d));
                            }


                            break;

                        }

                    }

                    paths.push(TracedPath {
                        typ: path_type,
                        pixels: pixels
                    });

                }

            }

        }
    }

    (bounds, paths)

}


// Types -----------------------------------------------------------------------
#[derive(Eq, PartialEq, Copy, Clone)]
enum TracedPathType {
    Wall,
    Light,
    Spawn,
    Solid
}

struct TracedPath {
    typ: TracedPathType,
    pixels: Vec<(i32, i32, u32, Direction)>
}


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Direction {
    Vertical,
    Horizontal,
    DiagonalOne,
    DiagonalTwo
}

#[derive(Debug, Default, RustcEncodable)]
struct Level {
    walls: Vec<Wall>,
    spawns: Vec<Spawn>,
    lights: Vec<Light>,
    solids: Vec<Vec<i32>>
}

#[derive(Debug, RustcEncodable)]
struct Wall {
    line: [f32; 4]
}

#[derive(Debug, RustcEncodable)]
struct Spawn {
    x: i32,
    y: i32
}

#[derive(Debug, RustcEncodable)]
struct Light {
    x: i32,
    y: i32,
    radius: i32
}


// Helpers --------------------------------------------------------------------
fn is_valid_path_pixel(
    x: i32, y: i32,
    img: &image::DynamicImage,
    usage: &HashSet<(i32, i32)>,
    w: i32, h: i32,
    path_type: TracedPathType

) -> bool {

    if x >= 0 && x < w && y >= 0 && y < h {
        if !usage.contains(&(x, y)) {
            if let Some(p) = get_path_type(&img.get_pixel(x as u32, y as u32)) {
                p == path_type

            } else {
                false
            }

        } else {
            false
        }

    } else {
        false
    }

}

fn is_potential_path_pixel(
    x: i32, y: i32,
    img: &image::DynamicImage,
    w: i32, h: i32,
    path_type: TracedPathType

) -> bool {

    if x >= 0 && x < w && y >= 0 && y < h {
        if let Some(p) = get_path_type(&img.get_pixel(x as u32, y as u32)) {
            p == path_type

        } else {
            false
        }

    } else {
        false
    }

}

fn get_path_type(pixel: &image::Rgba<u8>) -> Option<TracedPathType> {
    let (r, g, b) = (pixel.data[0], pixel.data[1], pixel.data[2]);
    if r == 255 && g == 255 && b == 255 {
        Some(TracedPathType::Wall)

    } else if r == 255 && g == 255 && b == 0 {
        Some(TracedPathType::Light)

    // TODO Automatically parse:
    // > pixel IS pink AND (neighbor is out of bounds OR neighbor is a wall)
    } else if r == 0 && g == 255 && b == 0 {
        Some(TracedPathType::Solid)

    } else if r == 0 && g == 255 && b == 255 {
        Some(TracedPathType::Spawn)

    } else {
        None
    }
}

