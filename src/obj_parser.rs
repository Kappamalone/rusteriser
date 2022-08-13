use cgmath::point3;
use cgmath::vec3;
use cgmath::InnerSpace;
use cgmath::Point3;
use cgmath::Vector3;
use std::fs::File;
use std::io::{BufRead, BufReader};

struct TextureAndCoordinates {
    texture_data: Vec<u32>,
    texture_coordinates: Vec<[Point3<f32>; 3]>,
}

// Holds all data corresponding to a loaded obj
#[derive(Clone)]
pub struct ObjData {
    // Triplet of vertices, Triplet of normals, Texture coords
    pub tri_positions: Vec<[Point3<f32>; 3]>,
    pub tri_textures: Vec<[Point3<f32>; 3]>,
    // pub tri_textures: Vec<TextureAndCoordinates>,
    pub tri_normals: Option<Vec<[Vector3<f32>; 3]>>,
}

impl ObjData {
    pub fn len(&self) -> usize {
        return self.tri_positions.len();
    }

    pub fn new(obj_path: &str) -> ObjData {
        // Temp buffers to be indexed into
        let mut temp_vertex_buffer: Vec<f32> = Vec::new();
        let mut temp_vertex_texture_buffer: Vec<f32> = Vec::new();
        let mut temp_vertex_normal_buffer: Vec<f32> = Vec::new();

        // The actual data
        let mut tri_positions: Vec<[Point3<f32>; 3]> = Vec::new();
        let mut tri_textures: Vec<[Point3<f32>; 3]> = Vec::new();
        let mut tri_normals: Option<Vec<[Vector3<f32>; 3]>> = Some(Vec::new());

        let file = File::open(obj_path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            // println!("{} {}", index, line.unwrap());
            let line = line.unwrap();
            // NOTE: `if let some` is useful for ignoring None
            //  The `if let` construct reads: "if `let` destructures `number` into
            // `Some(i)`, evaluate the block (`{}`).
            if let Some(id) = line.chars().nth(0) {
                match id {
                    'v' => match line.chars().nth(1).unwrap() {
                        ' ' => {
                            let vertexes: Vec<&str> = line.split_whitespace().collect();
                            temp_vertex_buffer.push(vertexes[1].parse::<f32>().unwrap());
                            temp_vertex_buffer.push(vertexes[2].parse::<f32>().unwrap());
                            temp_vertex_buffer.push(vertexes[3].parse::<f32>().unwrap());
                        }
                        't' => {
                            //(u, [v, w]) coordinates, these will vary between 0 and 1.
                            // v, w are optional and default to 0.
                            let vertexes: Vec<&str> = line.split_whitespace().collect();
                            match vertexes.len() {
                                2 => {
                                    temp_vertex_texture_buffer
                                        .push(vertexes[1].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer.push(0.);
                                    temp_vertex_texture_buffer.push(0.);
                                }
                                3 => {
                                    temp_vertex_texture_buffer
                                        .push(vertexes[1].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer
                                        .push(vertexes[2].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer.push(0.);
                                }
                                4 => {
                                    temp_vertex_texture_buffer
                                        .push(vertexes[1].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer
                                        .push(vertexes[2].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer
                                        .push(vertexes[3].parse::<f32>().unwrap());
                                }
                                _ => panic!("Error parsing vertex textures: {}", line),
                            }
                        }
                        'n' => {
                            let vertexes: Vec<&str> = line.split_whitespace().collect();
                            temp_vertex_normal_buffer.push(vertexes[1].parse::<f32>().unwrap());
                            temp_vertex_normal_buffer.push(vertexes[2].parse::<f32>().unwrap());
                            temp_vertex_normal_buffer.push(vertexes[3].parse::<f32>().unwrap());
                        }
                        _ => println!("Unhandled obj expression: {}", line),
                    },
                    'f' => {
                        let faces: Vec<&str> = line.split_whitespace().collect();
                        let slash_frequency: usize = faces
                            .iter()
                            .map(|x| x.chars().filter(|y| *y == '/').count())
                            .sum();
                        let v0: usize;
                        let v1: usize;
                        let v2: usize;
                        let mut vt0: Option<usize> = None;
                        let mut vt1: Option<usize> = None;
                        let mut vt2: Option<usize> = None;
                        let mut vn0: Option<usize> = None;
                        let mut vn1: Option<usize> = None;
                        let mut vn2: Option<usize> = None;
                        match slash_frequency {
                            0 => {
                                v0 = faces[1].parse::<usize>().unwrap() - 1;
                                v1 = faces[2].parse::<usize>().unwrap() - 1;
                                v2 = faces[3].parse::<usize>().unwrap() - 1;
                            }
                            3 => {
                                let group0 = faces[1].split('/').collect::<Vec<&str>>();
                                let group1 = faces[2].split('/').collect::<Vec<&str>>();
                                let group2 = faces[3].split('/').collect::<Vec<&str>>();
                                v0 = group0[0].parse::<usize>().unwrap() - 1;
                                v1 = group1[0].parse::<usize>().unwrap() - 1;
                                v2 = group2[0].parse::<usize>().unwrap() - 1;
                                vt0 = Some(group0[1].parse::<usize>().unwrap() - 1);
                                vt1 = Some(group1[1].parse::<usize>().unwrap() - 1);
                                vt2 = Some(group2[1].parse::<usize>().unwrap() - 1);
                            }
                            6 => {
                                // FIXME: probably breaks with v//vn format
                                let group0 = faces[1].split('/').collect::<Vec<&str>>();
                                let group1 = faces[2].split('/').collect::<Vec<&str>>();
                                let group2 = faces[3].split('/').collect::<Vec<&str>>();
                                v0 = group0[0].parse::<usize>().unwrap() - 1;
                                v1 = group1[0].parse::<usize>().unwrap() - 1;
                                v2 = group2[0].parse::<usize>().unwrap() - 1;
                                vt0 = Some(group0[1].parse::<usize>().unwrap() - 1);
                                vt1 = Some(group1[1].parse::<usize>().unwrap() - 1);
                                vt2 = Some(group2[1].parse::<usize>().unwrap() - 1);
                                vn0 = Some(group0[2].parse::<usize>().unwrap() - 1);
                                vn1 = Some(group1[2].parse::<usize>().unwrap() - 1);
                                vn2 = Some(group2[2].parse::<usize>().unwrap() - 1);
                            }
                            _ => panic!("Unhandled format of faces: {}", line),
                        }
                        let tri_position: [Point3<f32>; 3] = [
                            point3(
                                temp_vertex_buffer[v0 * 3],
                                temp_vertex_buffer[v0 * 3 + 1],
                                temp_vertex_buffer[v0 * 3 + 2],
                            ),
                            point3(
                                temp_vertex_buffer[v1 * 3],
                                temp_vertex_buffer[v1 * 3 + 1],
                                temp_vertex_buffer[v1 * 3 + 2],
                            ),
                            point3(
                                temp_vertex_buffer[v2 * 3],
                                temp_vertex_buffer[v2 * 3 + 1],
                                temp_vertex_buffer[v2 * 3 + 2],
                            ),
                        ];
                        let tri_texture: [Point3<f32>; 3];
                        if let (Some(vt0), Some(vt1), Some(vt2)) = (vt0, vt1, vt2) {
                            tri_texture = [
                                point3(
                                    temp_vertex_texture_buffer[vt0 * 3],
                                    temp_vertex_texture_buffer[vt0 * 3 + 1],
                                    temp_vertex_texture_buffer[vt0 * 3 + 2],
                                ),
                                point3(
                                    temp_vertex_texture_buffer[vt1 * 3],
                                    temp_vertex_texture_buffer[vt1 * 3 + 1],
                                    temp_vertex_texture_buffer[vt1 * 3 + 2],
                                ),
                                point3(
                                    temp_vertex_texture_buffer[vt2 * 3],
                                    temp_vertex_texture_buffer[vt2 * 3 + 1],
                                    temp_vertex_texture_buffer[vt2 * 3 + 2],
                                ),
                            ];
                        } else {
                            tri_texture =
                                [point3(1., 1., 1.), point3(1., 1., 1.), point3(1., 1., 1.)];
                        }
                        let tri_normal: [Vector3<f32>; 3];
                        if let (Some(vn0), Some(vn1), Some(vn2)) = (vn0, vn1, vn2) {
                            tri_normal = [
                                vec3(
                                    temp_vertex_normal_buffer[vn0 * 3],
                                    temp_vertex_normal_buffer[vn0 * 3 + 1],
                                    temp_vertex_normal_buffer[vn0 * 3 + 2],
                                )
                                .normalize(),
                                vec3(
                                    temp_vertex_normal_buffer[vn1 * 3],
                                    temp_vertex_normal_buffer[vn1 * 3 + 1],
                                    temp_vertex_normal_buffer[vn1 * 3 + 2],
                                )
                                .normalize(),
                                vec3(
                                    temp_vertex_normal_buffer[vn2 * 3],
                                    temp_vertex_normal_buffer[vn2 * 3 + 1],
                                    temp_vertex_normal_buffer[vn2 * 3 + 2],
                                )
                                .normalize(),
                            ];
                            tri_normals.as_mut().unwrap().push(tri_normal);
                        } else {
                            tri_normals = None;
                        }

                        tri_positions.push(tri_position);
                        tri_textures.push(tri_texture);
                    }
                    '#' => println!(".obj file comment: {}", line),
                    _ => println!("Unhandled obj expression: {}", line), // should panic!() instead
                }
            }
        }

        assert!(
            tri_positions.len() == tri_textures.len(),
            "Incorrect parsing of position/normals/textures: {} {}",
            tri_positions.len(),
            tri_textures.len(),
        );

        ObjData {
            tri_positions,
            tri_textures,
            tri_normals,
        }
    }
}
