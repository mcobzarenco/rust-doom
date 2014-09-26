use std::collections::HashMap;
use std::io::{BufReader, Reader, SeekSet};
use std::{str, mem};

use super::Archive;
use super::image::Image;
use super::types::*;
use super::util::{read_binary, name_toupper, flat_frame_names,
                  wall_frame_names};

use texture::Texture;

use gl;
use numvec::{Vec2, Vec2f};
use time;


pub type Palette = [[u8, ..3], ..256];
pub type Colormap = [u8, ..256];
pub type Flat = Vec<u8>;

pub struct Bounds {
    pub pos: Vec2f,
    pub size: Vec2f,
    pub num_frames: uint,
    pub frame_offset: uint,
}

pub struct TextureDirectory {
    textures: HashMap<Vec<u8>, Image>,
    patches: Vec<(WadName, Option<Image>)>,
    palettes: Vec<Palette>,
    colormaps: Vec<Colormap>,
    flats: HashMap<Vec<u8>, Flat>,
}

macro_rules! io_try(
    ($e:expr) => (try!($e.map_err(|e| String::from_str(e.desc))))
)

impl TextureDirectory {
    pub fn from_archive(wad: &mut Archive) -> Result<TextureDirectory, String> {
        info!("Reading texture directory...");
        // Read palettes & colormaps.
        let palettes = wad.read_lump_by_name(PLAYPAL_LUMP_NAME);
        let colormaps = wad.read_lump_by_name(COLORMAP_LUMP_NAME);
        info!("  {:4} palettes", palettes.len());
        info!("  {:4} colormaps", colormaps.len());

        // Read patches.
        let patches = try!(read_patches(wad));
        info!("  {:4} patches", patches.len());

        // Read textures.
        let t0 = time::precise_time_s();
        info!("Reading & assembling textures...");
        let mut textures = HashMap::new();
        for lump_name in TEXTURE_LUMP_NAMES.iter() {
            let lump_index = match wad.get_lump_index(lump_name) {
                Some(i) => i,
                None => {
                    info!("     0 textures in {}",
                          str::from_utf8(lump_name));
                    continue
                }
            };
            let num_textures = try!(
                read_textures(wad.read_lump(lump_index).as_slice(),
                              patches.as_slice(), &mut textures));
            info!("  {:4} textures in {}",
                  num_textures, str::from_utf8(lump_name));
        }
        let textures = textures;
        info!("Done in {:.4}s.", time::precise_time_s() - t0);

        // Read flats.
        let flats = try!(read_flats(wad));
        info!("  {:4} flats", flats.len());


        Ok(TextureDirectory {
            patches: patches,
            textures: textures,
            palettes: palettes,
            colormaps: colormaps,
            flats: flats,
        })
    }

    pub fn get_texture<'a>(&'a self, name: &[u8]) -> Option<&'a Image> {
        self.textures.find(&name_toupper(name))
    }
    pub fn get_flat<'a>(&'a self, name: &[u8]) -> Option<&'a Flat> {
        self.flats.find(&name_toupper(name))
    }

    pub fn num_patches(&self) -> uint { self.patches.len() }
    pub fn get_patch<'a>(&'a self, index: uint) -> Option<&'a Image> {
        self.patches[index].1.as_ref()
    }

    pub fn num_palettes(&self) -> uint { self.palettes.len() }
    pub fn get_palette<'a>(&'a self, index: uint) -> &'a Palette {
        &self.palettes[index]
    }

    pub fn num_colormaps(&self) -> uint { self.colormaps.len() }
    pub fn get_colormap<'a>(&'a self, index: uint) -> &'a Colormap {
        &self.colormaps[index]
    }

    pub fn build_palette_texture(&self,
                                 palette: uint,
                                 colormap_start: uint,
                                 colormap_end: uint) -> Texture {
        let num_colormaps = colormap_end - colormap_start;
        let mut data = Vec::from_elem(256 * num_colormaps * 3, 0u8);
        let palette = &self.palettes[palette];
        for i_colormap in range(colormap_start, colormap_end) {
            for i_color in range(0, 256) {
                let rgb = &palette[self.colormaps[i_colormap][i_color] as uint];
                *data.get_mut(0 + i_color * 3 + i_colormap * 768) = rgb[0];
                *data.get_mut(1 + i_color * 3 + i_colormap * 768) = rgb[1];
                *data.get_mut(2 + i_color * 3 + i_colormap * 768) = rgb[2];
            }
        }

        let mut palette_tex = Texture::new(gl::TEXTURE_2D);
        palette_tex.bind(gl::TEXTURE0);
        palette_tex
            .set_filters_nearest()
            .data_rgb_u8(0, 256, num_colormaps, data.as_slice())
            .unbind(gl::TEXTURE0);
        palette_tex
    }

    pub fn build_colormap_texture(&self) -> Texture {
        let mut colormap_tex = Texture::new(gl::TEXTURE_2D);
        colormap_tex.bind(gl::TEXTURE0);
        colormap_tex
            .set_filters_nearest()
            .data_red_u8(0, 256, self.colormaps.len(),
                         self.colormaps.as_slice())
            .unbind(gl::TEXTURE0);
        colormap_tex
    }

    pub fn build_picture_atlas<'a, T: Iterator<&'a [u8]>>(
            &self, mut names_iter: T) -> (Texture, HashMap<Vec<u8>, Bounds>) {
        let mut images = Vec::new();
        for name in names_iter {
            let name = name_toupper(name);
            match wall_frame_names(name.as_slice()) {
                None => images.push((self.get_texture(name.as_slice()).unwrap(),
                                     name, 0, 1)),
                Some(frames) => {
                    for (offset, name) in frames.iter().enumerate() {
                        images.push((self.get_texture(name.as_slice()).unwrap(),
                                     name.to_vec(), offset, frames.len()));
                    }
                }
            }
        }
        let images = images;
        assert!(images.len() > 0, "No images in wall atlas.");

        fn img_bound((x_offset, y_offset): (int, int), img: &Image,
                     frame_offset: uint, num_frames: uint) -> Bounds {
            Bounds { pos: Vec2::new(x_offset as f32, y_offset as f32),
                     size: Vec2::new(img.width() as f32, img.height() as f32),
<<<<<<< HEAD
                     num_frames: num_frames, frame_offset: frame_offset }
=======
                     num_frames: 1,
                     frame_offset: 0 }
>>>>>>> ac48e418d63845623908696733b64029e1b6584c
        }

        let num_pixels = images
            .iter().map(|t| t.0.num_pixels()).fold(0, |x, y| x + y);
        let min_atlas_width = images
            .iter().map(|t| t.0.width()).max().unwrap();
        let min_atlas_height = 128;
        let max_size = 4096;

        let next_size = |w: &mut uint, h: &mut uint| {
            loop {
                if *w == *h {
                    if *w == max_size { fail!("Could not fit wall atlas."); }
                    *w *= 2; *h = min_atlas_height;
                } else { *h *= 2; }

                if *w * *h >= num_pixels { break; }
            }
        };

        let (mut atlas_width, mut atlas_height) = (min_atlas_width,
                                                   min_atlas_height);
        next_size(&mut atlas_width, &mut atlas_height);

        let mut transposed = false;
        let mut offsets = Vec::with_capacity(images.len());
        loop {
            let mut x_offset = 0;
            let mut y_offset = 0;
            let mut failed = false;
            let mut max_height = 0;
            for &(image, _, _, _) in images.iter() {
                let (width, height) = (image.width(), image.height());
                if height > max_height { max_height = height; }
                if x_offset + width > atlas_width {
                    x_offset = 0;
                    y_offset += max_height;
                    max_height = 0;
                }
                if y_offset + height > atlas_height {
                    failed = true;
                    break;
                }
                offsets.push((x_offset as int, y_offset as int));
                x_offset += width;
            }

            if failed {
                offsets.clear();

                // Try transposing width<->height.
                let aux = atlas_width;
                atlas_width = atlas_height;
                atlas_height = aux;
                transposed = !transposed;
                if transposed && atlas_width != atlas_height {
                    continue;
                }

                // If all else fails try a larger size for the atlas.
                transposed = false;
                next_size(&mut atlas_width, &mut atlas_height);
            } else {
                break;
            }
        }
        let (atlas_width, atlas_height) = (atlas_width, atlas_height);

        assert!(offsets.len() == images.len());
        let mut atlas = Image::new(atlas_width, atlas_height);
        let mut bound_map = HashMap::with_capacity(images.len());
        for (i, (image, name, frame_offset, num_frames)) in
                images.into_iter().enumerate() {
            atlas.blit(image, offsets[i].0, offsets[i].1 as int, true);
            bound_map.insert(name, img_bound(offsets[i - frame_offset],
                                             image, frame_offset, num_frames));
        }
        drop(offsets);

        let mut tex = Texture::new(gl::TEXTURE_2D);
        tex.bind(gl::TEXTURE0);
        tex.set_filters_nearest()
           .data_rg_u8(0, atlas_width, atlas_height, atlas.get_pixels())
           .unbind(gl::TEXTURE0);

        info!("Wall texture atlas size: {}x{}", atlas_width, atlas_height);
        (tex, bound_map)
    }

    pub fn build_flat_atlas<'a, T: Iterator<&'a [u8]>>(&self, mut names_iter: T)
            -> (Texture, HashMap<Vec<u8>, Bounds>) {
        let mut names = Vec::new();
        for name in names_iter {
            let name = name_toupper(name);
            match flat_frame_names(name.as_slice()) {
                None => names.push((0, 1, name)),
<<<<<<< HEAD
                Some(frames) => {
=======
                Some((_, frames)) => {
>>>>>>> ac48e418d63845623908696733b64029e1b6584c
                    for (offset, frame) in frames.iter().enumerate() {
                        names.push((offset, frames.len(), frame.to_vec()));
                    }
                }
            }
        }
<<<<<<< HEAD
        let names = names;
=======
>>>>>>> ac48e418d63845623908696733b64029e1b6584c
        let num_names = names.len();

        let width = next_pow2((num_names as f64).sqrt().ceil() as uint * 64);
        let flats_per_row = width / 64;

        let num_rows = (num_names as f64 / flats_per_row as f64).ceil() as uint;
        let height = next_pow2(num_rows * 64);

        let mut offsets = HashMap::with_capacity(num_names);
        let mut data = Vec::from_elem(width * height, 255u8);
        let (mut row, mut column) = (0, 0);
        info!("Flat atlas size: {}x{} ({}, {})", width, height, flats_per_row,
                                                 num_rows);
        let mut anim_start_pos = Vec2::zero();
        for (frame_offset, num_frames, name) in names.into_iter() {
            let flat = self.get_flat(name.as_slice()).expect("Unknown flat.");
            let x_offset = column * 64;
            let y_offset = row * 64;
<<<<<<< HEAD
=======
            println!("{} {} {}",
                     frame_offset, num_frames,
                     str::from_utf8(name.as_slice()));
>>>>>>> ac48e418d63845623908696733b64029e1b6584c

            if frame_offset == 0 {
               anim_start_pos = Vec2::new(x_offset as f32, y_offset as f32);
            }
            offsets.insert(name, Bounds {
                pos: anim_start_pos,
                size: Vec2::new(64.0, 64.0),
                num_frames: num_frames,
                frame_offset: frame_offset
            });

            for y in range(0, 64) {
                for x in range(0, 64) {
                    *data.get_mut(x_offset + x + (y + y_offset) * width)
                        = flat[x + y * 64];
                }
            }

            column += 1;
            if column == flats_per_row {
                column = 0;
                row += 1;
            }
        }

        let mut tex = Texture::new(gl::TEXTURE_2D);
        tex.bind(gl::TEXTURE0);
        tex.set_filters_nearest()
           .data_red_u8(0, width, height, data.as_slice())
           .unbind(gl::TEXTURE0);

        (tex, offsets)
    }

}


fn next_pow2(x: uint) -> uint {
    let mut pow2 = 1;
    while pow2 < x { pow2 *= 2; }
    pow2
}


static PNAMES_LUMP_NAME: &'static [u8, ..8] =
    &[b'P', b'N', b'A', b'M', b'E', b'S', b'\0', b'\0'];

static PLAYPAL_LUMP_NAME: &'static [u8, ..8] =
    &[b'P', b'L', b'A', b'Y', b'P', b'A', b'L', b'\0'];

static COLORMAP_LUMP_NAME: &'static [u8, ..8] =
    &[b'C', b'O', b'L', b'O', b'R', b'M', b'A', b'P'];

static F_START_LUMP_NAME: &'static [u8, ..8] =
    &[b'F', b'_', b'S', b'T', b'A', b'R', b'T', b'\0'];

static F_END_LUMP_NAME: &'static [u8, ..8] =
    &[b'F', b'_', b'E', b'N', b'D', b'\0', b'\0', b'\0'];

static TEXTURE_LUMP_NAMES: &'static [[u8, ..8]] =
    &[[b'T', b'E', b'X', b'T', b'U', b'R', b'E', b'1'],
      [b'T', b'E', b'X', b'T', b'U', b'R', b'E', b'2']];


fn read_patches(wad: &mut Archive)
        -> Result<Vec<(WadName, Option<Image>)>, String> {
    let pnames_buffer = wad.read_lump_by_name(PNAMES_LUMP_NAME);
    let mut lump = BufReader::new(pnames_buffer.as_slice());

    let num_patches = io_try!(lump.read_le_u32()) as uint;
    let mut patches = Vec::with_capacity(num_patches);

    patches.reserve_additional(num_patches);
    let mut missing_patches = 0u;
    info!("Reading {} patches....", num_patches);
    let t0 = time::precise_time_s();
    for _ in range(0, num_patches) {
        let name = read_binary::<WadName, _>(&mut lump);
        let patch = wad.get_lump_index(&name).map(|index| {
            let patch_buffer = wad.read_lump(index);
            Image::from_buffer(patch_buffer.as_slice())
        });
        if patch.is_none() { missing_patches += 1; }
        patches.push((name, patch));
    }
    let time = time::precise_time_s() - t0;
    warn!("Done in {:.4f}s; {} missing patches.", time, missing_patches);
    Ok(patches)
}


fn read_textures(lump_buffer: &[u8], patches: &[(WadName, Option<Image>)],
                 textures: &mut HashMap<Vec<u8>, Image>)
        -> Result<uint, String> {
    let mut lump = BufReader::new(lump_buffer.as_slice());
    let num_textures = io_try!(lump.read_le_u32()) as uint;
    let current_num_textures = textures.len();
    textures.reserve(current_num_textures + num_textures);

    let mut offsets = BufReader::new({
        let begin = io_try!(lump.tell()) as uint;
        let size = num_textures * mem::size_of::<u32>();
        lump_buffer.slice(begin, begin + size)
    });

    for _ in range(0, num_textures) {
        io_try!(lump.seek(io_try!(offsets.read_le_u32()) as i64, SeekSet));
        let header = read_binary::<WadTextureHeader, _>(&mut lump);
        let mut image = Image::new_from_header(&header);

        for i_patch in range(0, header.num_patches) {
            let pref = read_binary::<WadTexturePatchRef, _>(&mut lump);
            let (off_x, off_y) = (pref.origin_x as int, pref.origin_y as int);
            match patches[pref.patch as uint] {
                (_, Some(ref patch)) => {
                    image.blit(patch,
                               off_x, if off_y < 0 { 0 } else { off_y },
                               i_patch == 0);
                },
                (ref patch_name, None) => {
                    return Err(format!("Texture {} uses missing patch {}.",
                               str::from_utf8(header.name),
                               str::from_utf8(patch_name)));
                }
            }
        }

        textures.insert(name_toupper(header.name), image);
    }
    Ok(num_textures)
}

fn read_flats(wad: &mut Archive) -> Result<HashMap<Vec<u8>, Flat>, String> {
    let start = match wad.get_lump_index(F_START_LUMP_NAME) {
        Some(index) => index + 1,
        None => return Err(String::from_str("Missing F_START."))
    };

    let end = match wad.get_lump_index(F_END_LUMP_NAME) {
        Some(index) => index,
        None => return Err(String::from_str("Missing F_END."))
    };

    let mut flats = HashMap::with_capacity(end - start);
    for i_lump in range(start, end) {
        if wad.is_virtual_lump(i_lump) { continue; }
        let lump = wad.read_lump(i_lump);
        flats.insert(name_toupper(wad.get_lump_name(i_lump)), lump);
    }

    Ok(flats)
}

