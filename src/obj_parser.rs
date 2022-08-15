use crate::MtlData;
use cgmath::point3;
use cgmath::vec3;
use cgmath::InnerSpace;
use cgmath::Point3;
use cgmath::Vector3;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::rc::Rc;

// Holds current texture data when parsing obj files
// This is a 0: pointer to texture, 1: width, 2: height
#[derive(Clone)]
struct CurrentTextureData(Rc<Vec<u8>>, usize, usize);

// TODO: restructure everything to use flattened arrays
// Holds all the data needed to interpolate inside a texture,
#[derive(Clone)]
pub struct TextureData {
    pub texture: Rc<Vec<u8>>,
    pub width: usize,
    pub height: usize,
    pub points: [Point3<f32>; 3],
}

//TODO: figure out how like, specular textures and stuff work
impl TextureData {
    fn new(
        texture: Rc<Vec<u8>>,
        width: usize,
        height: usize,
        points: [Point3<f32>; 3],
    ) -> TextureData {
        TextureData {
            texture,
            width,
            height,
            points,
        }
    }
}

// Holds all data corresponding to a loaded obj
#[derive(Clone)]
pub struct ObjData {
    // Triplet of vertices, Triplet of normals, Texture coords
    pub tri_positions: Vec<[Point3<f32>; 3]>,
    pub tri_textures: Option<Vec<TextureData>>,
    pub tri_normals: Option<Vec<[Vector3<f32>; 3]>>,
    pub mtl: Option<MtlData>,
    textures: Vec<Rc<Vec<u8>>>,
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
        let mut tri_normals: Option<Vec<[Vector3<f32>; 3]>> = Some(Vec::new());

        // let mut tri_textures: Option<Vec<TextureInfo>> = Some(Vec::new());
        let mut tri_textures: Option<Vec<TextureData>> = Some(Vec::new());
        let mut current_texture_info: Option<CurrentTextureData> = None;
        let mut textures: Vec<Rc<Vec<u8>>> = Vec::new();
        let mut mtl: Option<MtlData> = None;

        let file = File::open(obj_path).unwrap();
        let reader = BufReader::new(file);
        let obj_dir = Path::new(obj_path).parent().unwrap().to_str().unwrap();

        for line in reader.lines() {
            let line = line.unwrap();
            if line == "" {
                continue;
            }
            let elements = line.split_whitespace().collect::<Vec<&str>>();
            let id = elements[0];
            {
                match id {
                    "mtllib" => {
                        //TODO
                        let mtl_name = elements[1];
                        let mtl_path = Path::new(obj_dir).join(mtl_name);
                        println!("Loaded .mtl file: {:?}", mtl_path);
                        mtl = Some(MtlData::new(mtl_path.to_str().unwrap()));
                    }
                    "usemtl" => {
                        //TODO: change this back
                        let texture_name = &mtl.as_ref().unwrap().texture_path_map[elements[1]];
                        let texture_path = Path::new(obj_dir).join(texture_name);
                        let file_type = Path::new(&texture_path)
                            .extension()
                            .unwrap()
                            .to_str()
                            .unwrap();

                        println!("Loaded texture: {:?} {}", texture_path, file_type);

                        // load png data into vector
                        match file_type {
                            "png" => {
                                let decoder = png::Decoder::new(File::open(texture_path).unwrap());
                                let mut reader = decoder.read_info().unwrap();
                                // Allocate the output buffer.
                                let mut buf = vec![0; reader.output_buffer_size()];
                                // Read the next frame. An APNG might contain multiple frames.
                                let png_info = reader.next_frame(&mut buf).unwrap();

                                assert!(
                                    png_info.bit_depth == png::BitDepth::Eight,
                                    "PNG bit depth not 8!"
                                );
                                assert!(
                                    png_info.color_type == png::ColorType::Rgba,
                                    "PNG color type not rgba!"
                                );

                                let mut bytes = vec![0; png_info.buffer_size()];
                                // this makes origin bottom left instead of top left
                                for y in 0..png_info.height as usize {
                                    for x in 0..png_info.width as usize {
                                        let idx_transform = x * 4 + y * png_info.width as usize * 4;
                                        let idx_original = x * 4
                                            + (png_info.height as usize - y - 1)
                                                * png_info.width as usize
                                                * 4;
                                        for i in 0..3 {
                                            bytes[idx_transform + i] = buf[idx_original + i];
                                        }
                                    }
                                }

                                let texture = Rc::new(bytes);
                                current_texture_info = Some(CurrentTextureData(
                                    Rc::clone(&texture),
                                    png_info.width as usize,
                                    png_info.height as usize,
                                ));

                                println!("png metadata: {:?}", png_info);
                                textures.push(texture);
                            }
                            _ => panic!("unhandled texture file type!"),
                        }
                    }
                    "v" | "vt" | "vn" => match line.chars().nth(1).unwrap() {
                        ' ' => {
                            temp_vertex_buffer.push(elements[1].parse::<f32>().unwrap());
                            temp_vertex_buffer.push(elements[2].parse::<f32>().unwrap());
                            temp_vertex_buffer.push(elements[3].parse::<f32>().unwrap());
                        }
                        't' => {
                            //(u, [v, w]) coordinates, these will vary between 0 and 1.
                            // v, w are optional and default to 0.
                            match elements.len() {
                                2 => {
                                    temp_vertex_texture_buffer
                                        .push(elements[1].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer.push(0.);
                                    temp_vertex_texture_buffer.push(0.);
                                }
                                3 => {
                                    temp_vertex_texture_buffer
                                        .push(elements[1].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer
                                        .push(elements[2].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer.push(0.);
                                }
                                4 => {
                                    temp_vertex_texture_buffer
                                        .push(elements[1].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer
                                        .push(elements[2].parse::<f32>().unwrap());
                                    temp_vertex_texture_buffer
                                        .push(elements[3].parse::<f32>().unwrap());
                                }
                                _ => panic!("Error parsing vertex textures: {}", line),
                            }
                        }
                        'n' => {
                            temp_vertex_normal_buffer.push(elements[1].parse::<f32>().unwrap());
                            temp_vertex_normal_buffer.push(elements[2].parse::<f32>().unwrap());
                            temp_vertex_normal_buffer.push(elements[3].parse::<f32>().unwrap());
                        }
                        _ => println!("Unhandled obj expression: {}", line),
                    },
                    "f" => {
                        let slash_frequency: usize = elements
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
                                v0 = elements[1].parse::<usize>().unwrap() - 1;
                                v1 = elements[2].parse::<usize>().unwrap() - 1;
                                v2 = elements[3].parse::<usize>().unwrap() - 1;
                            }
                            3 => {
                                let group0 = elements[1].split('/').collect::<Vec<&str>>();
                                let group1 = elements[2].split('/').collect::<Vec<&str>>();
                                let group2 = elements[3].split('/').collect::<Vec<&str>>();
                                v0 = group0[0].parse::<usize>().unwrap() - 1;
                                v1 = group1[0].parse::<usize>().unwrap() - 1;
                                v2 = group2[0].parse::<usize>().unwrap() - 1;
                                vt0 = Some(group0[1].parse::<usize>().unwrap() - 1);
                                vt1 = Some(group1[1].parse::<usize>().unwrap() - 1);
                                vt2 = Some(group2[1].parse::<usize>().unwrap() - 1);
                            }
                            6 => {
                                // FIXME: breaks with v//vn format
                                let group0 = elements[1].split('/').collect::<Vec<&str>>();
                                let group1 = elements[2].split('/').collect::<Vec<&str>>();
                                let group2 = elements[3].split('/').collect::<Vec<&str>>();
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
                        tri_positions.push(tri_position);

                        if let (Some(vt0), Some(vt1), Some(vt2)) = (vt0, vt1, vt2) {
                            let points = [
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
                            let tri_texture_data = TextureData::new(
                                Rc::clone(&current_texture_info.as_ref().unwrap().0),
                                current_texture_info.as_ref().unwrap().1,
                                current_texture_info.as_ref().unwrap().2,
                                points,
                            );
                            tri_textures.as_mut().unwrap().push(tri_texture_data);
                        } else {
                            tri_textures = None;
                        }

                        if let (Some(vn0), Some(vn1), Some(vn2)) = (vn0, vn1, vn2) {
                            let tri_normal = [
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
                    }
                    "#" => println!(".obj file comment: {}", line),
                    _ => println!("Unhandled .obj expression: {}", line), // should panic!() instead
                }
            }
        }

        /*
        assert!(
            tri_positions.len() == tri_textures.as_ref().unwrap().len(),
            "REMOVE ME"
        );
        */
        ObjData {
            tri_positions,
            tri_textures,
            tri_normals,
            textures,
            mtl,
        }
    }
}
