// Crates ---------------------------------------------------------------------
extern crate rustc_serialize;
extern crate image;
extern crate toml;


// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::collections::{HashMap, HashSet};


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

fn find_paths(img: &image::DynamicImage) -> ([i32; 4], Vec<TracedPath>) {

    let (w, h) = img.dimensions();
    let mut pixel_usage: HashSet<(i32, i32)> = HashSet::new();

    // Find walls, spawns and lights
    let (bounds, mut paths) = extract_paths(
        img,
        &mut pixel_usage,
        false,
        &[0, 0, w as i32, h as i32]
    );

    // Find solids
    let (_, mut solid_paths) = extract_paths(
        img,
        &mut pixel_usage,
        true,
        &[
            bounds[0],
            bounds[1],
            bounds[2] + 1,
            bounds[3] + 1
        ]
    );

    paths.append(&mut solid_paths);

    (bounds, paths)

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

                let (dx, dy) = (line[2] - line[0], line[3] - line[1]);

                // Avoid adding diagonal walls with spanning just two pixels
                if (dx * dx + dy * dy).sqrt() > 1.5 {
                    level.walls.push(Wall {
                        line: line
                    });
                }

            }

        // Construct concave polygons from solids
        } else if p.typ == TracedPathType::Solid {

            //let lh = 0;
            //let lw = 0;
            let mut directions = Vec::new();
            let mut points = Vec::new();
            let (mut ox, mut oy, mut or, _) = p.pixels[0];
            let (mut lx, mut ly, _, _) = p.pixels[0];

            let ir = or;
            directions.push(ir);
            points.push(ox - lw);
            points.push(oy - lh);

            for &(x, y, r, _) in &p.pixels {
                if r != or {
                    let (dx, dy) = (lx - ox, ly - oy);
                    if ((dx * dx + dy * dy) as f32).sqrt() > 0.0 {
                        directions.push(direction_from_delta(dx, dy));
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


            let l = points.len() / 2;
            for i in 0..l {
                let (pr, nr) = (directions[i], directions[(i + 1) % l]);
                let e = extrusion(pr, nr);
                points[i * 2] += e.0;
                points[i * 2 + 1] += e.1;
            }

            if points.len() >= 8 {
                level.solids.push(points);
            }

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

    // Next up merge wall endpoints which meet up, this is done in order to avoid
    // visual glitches with visibility and light cone rendering (they would
    // otherwise shine through the cracks)
    let endpoint_list: Vec<(i32, i32)> = level.walls.iter().flat_map(|w| vec![(w.line[0] as i32, w.line[1] as i32), (w.line[2] as i32, w.line[3] as i32)]).collect();
    let mut count: HashMap<(i32, i32), usize> = HashMap::new();
    for e in &endpoint_list {
        count.insert(*e, 0);
    }

    for e in endpoint_list {
        *count.get_mut(&e).unwrap() += 1;
    }

    // Extract all endpoints where lines actually meet up
    let mut endpoints = HashSet::new();
    for (e, c) in count {
        if c >= 2 {
            endpoints.insert(e);
        }
    }
    for w in &mut level.walls {

        let a = (w.line[0] as i32, w.line[1] as i32);
        let b = (w.line[2] as i32, w.line[3] as i32);

        // Vertical
        if a.0 == b.0 {
            let d = (a.0, a.1 - 1);
            if endpoints.contains(&d) {
                w.line[1] -= 1.0;
            }

            let e = (b.0, b.1 + 1);
            if endpoints.contains(&e) {
                w.line[3] += 1.0;
            }

        // Horizontal
        } else {
            let d = (a.0 - 1, a.1);
            if endpoints.contains(&d) {
                w.line[0] -= 1.0;
            }

            let e = (b.0 + 1, b.1);
            if endpoints.contains(&e) {
                w.line[2] += 1.0;
            }
        }

    }

    level

}

fn extract_paths(
    img: &image::DynamicImage,
    pixel_usage: &mut HashSet<(i32, i32)>,
    solids: bool,
    outer_bounds: &[i32; 4]

) -> ([i32; 4], Vec<TracedPath>) {

    let mut paths = Vec::new();
    let mut bounds = [
        outer_bounds[2], outer_bounds[3],
        outer_bounds[0], outer_bounds[1]
    ];

    for y in outer_bounds[1]..outer_bounds[3] {
        for x in outer_bounds[0]..outer_bounds[2] {

            // Ignores pixels which are already part of a path
            if !pixel_usage.contains(&(x as i32, y as i32)) {

                // Start tracing a new path
                let path_type = get_path_type(x, y, img, solids, outer_bounds);
                if let Some(path_type) = path_type {

                    let mut px = x as i32;
                    let mut py = y as i32;

                    pixel_usage.insert((px, py));

                    let mut pixels = Vec::new();
                    let mut first_pixel = true;
                    let mut lr = 0;

                    // TODO prevent consuming pixels paths with len < 4
                    loop {

                        // Check top right bottom left first
                        let m = if is_valid_path_pixel(px, py - 1, &img, &pixel_usage, path_type, solids, outer_bounds) {
                            Some((px, py - 1, 0, Direction::Vertical))

                        } else if is_valid_path_pixel(px + 1, py, &img, &pixel_usage, path_type, solids, outer_bounds) {
                            Some((px + 1, py, 90, Direction::Horizontal))

                        } else if is_valid_path_pixel(px, py + 1, &img, &pixel_usage, path_type, solids, outer_bounds) {
                            Some((px, py + 1, 180, Direction::Vertical))

                        } else if is_valid_path_pixel(px - 1, py, &img, &pixel_usage, path_type, solids, outer_bounds) {
                            Some((px - 1, py, 270, Direction::Horizontal))

                        // Check topright bottomright bottomleft topleft second
                        } else if is_valid_path_pixel(px + 1, py - 1, &img, &pixel_usage, path_type, solids, outer_bounds) {
                            Some((px + 1, py - 1, 45, Direction::DiagonalOne))

                        } else if is_valid_path_pixel(px + 1, py + 1, &img, &pixel_usage, path_type, solids, outer_bounds) {
                            Some((px + 1, py + 1, 135, Direction::DiagonalTwo))

                        } else if is_valid_path_pixel(px - 1, py + 1, &img, &pixel_usage, path_type, solids, outer_bounds) {
                            Some((px - 1, py + 1, 225, Direction::DiagonalOne))

                        } else if is_valid_path_pixel(px - 1, py - 1, &img, &pixel_usage, path_type, solids, outer_bounds) {
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
                            let m = if lr == 0 && is_potential_path_pixel(px, py - 1, &img, path_type, solids, outer_bounds) {
                                Some((px, py - 1, 0, Direction::Vertical))

                            } else if lr == 90 && is_potential_path_pixel(px + 1, py, &img, path_type, solids, outer_bounds) {
                                Some((px + 1, py, 90, Direction::Horizontal))

                            } else if lr == 180 && is_potential_path_pixel(px, py + 1, &img, path_type, solids, outer_bounds) {
                                Some((px, py + 1, 180, Direction::Vertical))

                            } else if lr == 270 && is_potential_path_pixel(px - 1, py, &img, path_type, solids, outer_bounds) {
                                Some((px - 1, py, 270, Direction::Horizontal))

                            // Check topright bottomright bottomleft topleft second
                            } else if lr == 45 && is_potential_path_pixel(px + 1, py - 1, &img, path_type, solids, outer_bounds) {
                                Some((px + 1, py - 1, 45, Direction::DiagonalOne))

                            } else if lr == 135 && is_potential_path_pixel(px + 1, py + 1, &img, path_type, solids, outer_bounds) {
                                Some((px + 1, py + 1, 135, Direction::DiagonalTwo))

                            } else if lr == 225 && is_potential_path_pixel(px - 1, py + 1, &img, path_type, solids, outer_bounds) {
                                Some((px - 1, py + 1, 225, Direction::DiagonalOne))

                            } else if lr == 315 && is_potential_path_pixel(px - 1, py - 1, &img, path_type, solids, outer_bounds) {
                                Some((px - 1, py - 1, 315, Direction::DiagonalTwo))

                            } else {
                                None
                            };

                            if let Some((nx, ny, r, d)) = m {
                                pixels.push((nx, ny, r, d));
                            }

                            // Merge with adjacent paths at the end
                            let (px, py, lr, _) = pixels[0];
                            let m = if lr == 180 && is_potential_path_pixel(px, py - 1, &img, path_type, solids, outer_bounds) {
                                Some((px, py - 1, 0, Direction::Vertical))

                            } else if lr == 270 && is_potential_path_pixel(px + 1, py, &img, path_type, solids, outer_bounds) {
                                Some((px + 1, py, 90, Direction::Horizontal))

                            } else if lr == 00 && is_potential_path_pixel(px, py + 1, &img, path_type, solids, outer_bounds) {
                                Some((px, py + 1, 180, Direction::Vertical))

                            } else if lr == 90 && is_potential_path_pixel(px - 1, py, &img, path_type, solids, outer_bounds) {
                                Some((px - 1, py, 270, Direction::Horizontal))

                            // Check topright bottomright bottomleft topleft second
                            } else if lr == 225 && is_potential_path_pixel(px + 1, py - 1, &img, path_type, solids, outer_bounds) {
                                Some((px + 1, py - 1, 45, Direction::DiagonalOne))

                            } else if lr == 315 && is_potential_path_pixel(px + 1, py + 1, &img, path_type, solids, outer_bounds) {
                                Some((px + 1, py + 1, 135, Direction::DiagonalTwo))

                            } else if lr == 45 && is_potential_path_pixel(px - 1, py + 1, &img, path_type, solids, outer_bounds) {
                                Some((px - 1, py + 1, 225, Direction::DiagonalOne))

                            } else if lr == 135 && is_potential_path_pixel(px - 1, py - 1, &img, path_type, solids, outer_bounds) {
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
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum TracedPathType {
    Wall,
    Light,
    Spawn,
    Solid
}

#[derive(Debug)]
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
fn direction_from_delta(dx: i32, dy: i32) -> u32 {

    // Vertical
    if dx == 0 {
        if dy > 0 {
            270

        } else {
            90
        }

    // Horizontal
    } else if dy == 0 {
        if dx > 0 {
            180

        } else {
            0
        }

    // Diagonal
    } else if dx > 0 {

        if dy > 0 {
            225

        } else {
            135
        }

    } else {
        if dy < 0 {
            45

        } else {
            315
        }
    }

}

fn extrusion(pr: u32, nr: u32) -> (i32, i32) {

    match (pr, nr) {

        // 90° Edges
        (  0,  90) => (-1,  1),
        ( 90, 180) => (-1, -1),

        (180, 270) => ( 1, -1),
        (270,   0) => ( 1,  1),

        // 45° In
        (  0,  45) => ( 0,  1),
        ( 45,  90) => (-1,  0),

        ( 90, 135) => (-1,  0),
        (135, 180) => ( 0, -1),

        (180, 225) => ( 0, -1),
        (225, 270) => ( 1,  0),

        (270, 315) => ( 1,  0),
        (315,   0) => ( 0,  1),

        // 45° Out

        ( 90,  45) => (-1,  0),
        ( 45,   0) => ( 0,  1),

        (180, 135) => ( 0, -1),
        (135,  90) => (-1,  0),

        (  0, 315) => ( 0,  1),
        (315, 270) => ( 1,  0),

        (270, 225) => ( 1,  0),
        (225, 180) => ( 0, -1),

        _ => unreachable!()

    }
}


fn is_valid_path_pixel(
    x: i32, y: i32,
    img: &image::DynamicImage,
    usage: &HashSet<(i32, i32)>,
    path_type: TracedPathType,
    check_solids: bool,
    bounds: &[i32; 4]

) -> bool {

    if x >= bounds[0] && x < bounds[2] && y >= bounds[1] && y < bounds[3] {
        if !usage.contains(&(x, y)) {
            if let Some(p) = get_path_type(x, y, img, check_solids, bounds) {
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
    path_type: TracedPathType,
    check_solids: bool,
    bounds: &[i32; 4]

) -> bool {

    if x >= bounds[0] && x < bounds[2] && y >= bounds[1] && y < bounds[3] {
        if let Some(p) = get_path_type(x, y, img, check_solids, bounds) {
            p == path_type

        } else {
            false
        }

    } else {
        false
    }

}

fn get_path_type(x: i32, y: i32, img: &image::DynamicImage, solids: bool, bounds: &[i32; 4]) -> Option<TracedPathType> {

    let pixel = img.get_pixel(x as u32, y as u32);
    let (r, g, b) = (pixel.data[0], pixel.data[1], pixel.data[2]);
    if solids && r == 255 && g == 0 && b == 255 {
        if any_wall_or_edge(x, y, img, bounds) {
            Some(TracedPathType::Solid)

        } else {
            None
        }

    } else if r == 255 && g == 255 && b == 255 {
        Some(TracedPathType::Wall)

    } else if r == 255 && g == 255 && b == 0 {
        Some(TracedPathType::Light)

    } else if r == 0 && g == 255 && b == 255 {
        Some(TracedPathType::Spawn)

    } else {
        None
    }
}

fn any_wall_or_edge(x: i32, y: i32, img: &image::DynamicImage, bounds: &[i32; 4]) -> bool {
    is_wall_or_edge(x, y - 1, img, bounds) ||
    is_wall_or_edge(x - 1, y, img, bounds) ||
    is_wall_or_edge(x + 1, y, img, bounds) ||
    is_wall_or_edge(x, y + 1, img, bounds)
}

fn is_wall_or_edge(x: i32, y: i32, img: &image::DynamicImage, bounds: &[i32; 4]) -> bool {
    if x < bounds[0] || x >= bounds[2] || y < bounds[1] || y >= bounds[3] {
        true

    } else {
        let pixel = img.get_pixel(x as u32, y as u32);
        pixel.data[0] == 255 && pixel.data[1] == 255 && pixel.data[2] == 255
    }

}

