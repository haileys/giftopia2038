use std::fs::File;
use std::io;
use std::path::Path;

use gif::{self, Decoder, SetParameter, ColorOutput};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{self, TextureCreator, Texture};
use sdl2::video::WindowContext;

pub struct Animation<'a> {
    pub rect: Rect,
    pub frames: Vec<(Texture<'a>, Texture<'a>)>,
}

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Gif(gif::DecodingError),
    Texture(render::TextureValueError),
    UpdateTexture(render::UpdateTextureError),
}

fn u32_swap_endian(buff: &mut [u8]) {
    let mut i = 0;

    while i < buff.len() {
        let a = buff[i + 0];
        let b = buff[i + 1];
        let c = buff[i + 2];
        let d = buff[i + 3];

        buff[i + 0] = d;
        buff[i + 1] = c;
        buff[i + 2] = b;
        buff[i + 3] = a;

        i += 4;
    }
}

fn u32_reverse(buff: &mut [u8]) {
    buff.reverse();
    // undo the reverse above within single u32s:
    u32_swap_endian(buff);
}

fn u32_reverse_rows(buff: &mut [u8], line_size: usize) {
    let mut i = 0;

    while i < buff.len() {
        u32_reverse(&mut buff[i..i + line_size]);
        i += line_size;
    }
}

impl<'a> Animation<'a> {
    pub fn load_gif(texture_creator: &'a TextureCreator<WindowContext>, path: &Path) -> Result<Animation<'a>, LoadError> {
        let file = File::open(path).map_err(LoadError::Io)?;

        let mut decoder = Decoder::new(file);
        decoder.set(ColorOutput::RGBA);

        let mut gif = decoder.read_info().map_err(LoadError::Gif)?;

        let width = gif.width() as usize;
        let height = gif.height() as usize;
        let buffer_size = gif.buffer_size();

        let mut frames = Vec::new();

        while let Some(frame) = gif.read_next_frame().map_err(LoadError::Gif)? {
            let rect = Rect::new(
                frame.left as i32,
                frame.top as i32,
                frame.width as u32,
                frame.height as u32);

            let mut buffer = frame.buffer.to_owned();
            let mut buffer = buffer.to_mut();
            u32_swap_endian(&mut buffer);

            let mut tex = texture_creator
                .create_texture_static(PixelFormatEnum::RGBA8888, width as u32, height as u32)
                .map_err(LoadError::Texture)?;

            tex.update(rect, &buffer, width * 4)
                .map_err(LoadError::UpdateTexture)?;

            let mut tex_flip = texture_creator
                .create_texture_static(PixelFormatEnum::RGBA8888, width as u32, height as u32)
                .map_err(LoadError::Texture)?;

            u32_reverse_rows(&mut buffer, width * 4);
            tex_flip.update(rect, &buffer, width * 4)
                .map_err(LoadError::UpdateTexture)?;

            frames.push((tex, tex_flip));
        }

        let rect = Rect::new(0, 0, width as u32, height as u32);

        Ok(Animation { rect, frames })
    }
}
