use core::mem::MaybeUninit;
use core::ops::Add;
use core::ops::Mul;
use core::ops::Sub;

use geometry::Vert3;

#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct Color(pub Vert3);
impl Add for Color {
    type Output = Color;
    fn add(self, rhs: Self) -> Self::Output {
        Color(Vert3(core::array::from_fn(|idx| {
            self.0.0[idx] + rhs.0.0[idx]
        })))
    }
}
impl Sub for Color {
    type Output = Color;
    fn sub(self, rhs: Self) -> Self::Output {
        Color(Vert3(core::array::from_fn(|idx| {
            self.0.0[idx] - rhs.0.0[idx]
        })))
    }
}
impl Mul for Color {
    type Output = Color;
    fn mul(self, rhs: Self) -> Self::Output {
        Color(Vert3(core::array::from_fn(|idx| {
            self.0.0[idx] * rhs.0.0[idx]
        })))
    }
}
impl Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Self::Output {
        Color(Vert3(core::array::from_fn(|idx| self.0.0[idx] * rhs)))
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;
    #[test]
    fn adding_colors() {
        let c1 = Color(Vert3::new(0.9, 0.6, 0.75));
        let c2 = Color(Vert3::new(0.7, 0.1, 0.25));
        assert_eq!(c1 + c2, Color(Vert3::new(1.6, 0.7, 1.0)))
    }
    #[test]
    fn subtracting_colors() {
        let c1 = Color(Vert3::new(0.9, 0.6, 0.75));
        let c2 = Color(Vert3::new(0.7, 0.1, 0.25));
        assert_eq!(c1 - c2, Color(Vert3::new(0.2, 0.5, 0.5)))
    }
    #[test]
    fn multiplying_a_color_by_a_scalar() {
        let c = Color(Vert3::new(0.2, 0.3, 0.4));
        assert_eq!(c * 2., Color(Vert3::new(0.4, 0.6, 0.8)))
    }
    #[test]
    fn multiplying_colors() {
        let c1 = Color(Vert3::new(1.0, 0.2, 0.4));
        let c2 = Color(Vert3::new(0.9, 1., 0.1));
        assert_eq!(c1 * c2, Color(Vert3::new(0.9, 0.2, 0.04)));
    }
}

pub struct Canvas {
    width: usize,
    height: usize,
    data: Box<[Color]>,
}
impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        let mut data: Box<[MaybeUninit<Color>]> = Box::new_uninit_slice(width * height);

        for point in data.as_mut() {
            point.write(Color(Vert3::ZERO));
        }

        let data = unsafe { data.assume_init() };

        Canvas {
            width,
            height,
            data,
        }
    }
    pub fn write_pixel(&mut self, (pixel_x, pixel_y): (usize, usize), pixel_color: Color) {
        debug_assert!(pixel_x <= self.width - 1);
        debug_assert!(pixel_y <= self.height - 1);
        self.data[pixel_y * self.width + pixel_x] = pixel_color;
    }
    pub fn pixel_at(&self, (pixel_x, pixel_y): (usize, usize)) -> &Color {
        debug_assert!(pixel_x <= self.width - 1);
        debug_assert!(pixel_y <= self.height - 1);
        &self.data[pixel_y * self.width + pixel_x]
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_a_canvas() {
        let c = Canvas::new(10, 20);
        assert!(c.data.iter().all(|elt| elt == &Color(Vert3::ZERO)))
    }
    #[test]
    fn writing_pixels_to_a_canvas() {
        let mut c = Canvas::new(10, 20);
        let red = Color(Vert3::X);
        c.write_pixel((2, 3), red);
        assert_eq!(c.pixel_at((2, 3)), &Color(Vert3::X))
    }
}
#[repr(C, packed)]
pub struct PPMHeader {
    pub magic: [u8; 3],
    pub width: usize,
    pub height: usize,
    pub color_scale: u8,
}
impl Default for PPMHeader {
    fn default() -> Self {
        PPMHeader {
            magic: [b'P', b'3', b'\n'],
            width: usize::default(),
            height: usize::default(),
            color_scale: 255,
        }
    }
}
pub enum PPMHeaderReaderState {
    Magic,
    Width,
    Height,
    ColorScale,
    Finished,
}
pub struct PPMHeaderReader<'c> {
    pub state: PPMHeaderReaderState,
    pub written: usize,
    pub header: &'c PPMHeader,
}
impl PPMHeaderReader<'_> {
    pub fn new(header: &PPMHeader) -> PPMHeaderReader<'_> {
        PPMHeaderReader {
            state: PPMHeaderReaderState::Magic,
            written: 0,
            header,
        }
    }
}
impl Read for PPMHeaderReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if matches!(self.state, PPMHeaderReaderState::Finished) {
            return Ok(0);
        }
        let remaining = buf.len() - self.written;
        if remaining == 0 {
            self.state = PPMHeaderReaderState::Finished;
            return Ok(0);
        }
        let dst = (&mut buf[self.written..]).as_mut_ptr();
        match self.state {
            PPMHeaderReaderState::Magic => {
                let to_write = usize::min(remaining, self.header.magic.len());
                let src = self.header.magic.as_slice().as_ptr();
                unsafe { core::ptr::copy_nonoverlapping(src, dst, to_write) };
                self.state = PPMHeaderReaderState::Width;
                self.written += to_write;
                self.read(buf)
            }
            PPMHeaderReaderState::Width => {
                let width = self.header.width;
                let src = format!("{} ", width);
                let src_buf = src.into_bytes();

                let to_write = usize::min(remaining, src_buf.len());

                let src = src_buf.as_ptr();

                unsafe { core::ptr::copy_nonoverlapping(src, dst, to_write) };

                self.state = PPMHeaderReaderState::Height;
                self.written += to_write;

                self.read(buf)
            }
            PPMHeaderReaderState::Height => {
                let height = self.header.height;
                let src = format!("{}\n", height);
                let src_buf = src.into_bytes();

                let to_write = usize::min(remaining, src_buf.len());

                let src = src_buf.as_ptr();

                unsafe { core::ptr::copy_nonoverlapping(src, dst, to_write) };

                self.state = PPMHeaderReaderState::ColorScale;
                self.written += to_write;

                self.read(buf)
            }
            PPMHeaderReaderState::ColorScale => {
                let color_scale = self.header.color_scale;
                let src = format!("{}\n", color_scale);
                let src_buf = src.into_bytes();

                let to_write = usize::min(remaining, src_buf.len());

                let src = src_buf.as_ptr();

                unsafe { core::ptr::copy_nonoverlapping(src, dst, to_write) };

                self.state = PPMHeaderReaderState::Finished;
                self.written += to_write;

                Ok(self.written)
            }
            PPMHeaderReaderState::Finished => Ok(0),
        }
    }
}
pub struct PPMWriter<'c> {
    pub buffer: Vec<u8>,
    pub canvas: &'c Canvas,
}
use std::io::{BufRead, Read};
impl Read for PPMReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.read >= self.inner_buf.len() {
            return Ok(0);
        }
        let cap = usize::min(self.inner_buf.len(), buf.len());
        for (curr, out) in self.inner_buf[self.read..].iter().zip(buf.iter_mut()) {
            *out = *curr;
            self.read += 1;
        }
        Ok(cap)
    }
}
pub struct PPMReader<'c> {
    pub canvas: &'c Canvas,
    pub written: usize,
    pub read: usize,
    pub inner_buf: Vec<u8>,
}
fn group_by_line_length(src: &[u8]) -> Vec<String> {
    let sp = src.split_inclusive(|spc| *spc == b' ');
    let mut out_buf = Vec::default();
    let mut line_buf = Vec::default();
    for seg in sp {
        if line_buf.len() + seg.len() < 70 {
            line_buf.extend(seg);
            // println!("Extended by {}, length now {}", seg.len(), line_buf.len());
        } else if line_buf.len() + seg.len() == 70 {
            println!("Edge case");
            line_buf.extend(seg);
            line_buf.push(b'\n');

            out_buf.push(String::from_utf8(line_buf.clone()).unwrap());

            line_buf.clear();
            println!("Length now {}", line_buf.len());
        } else {
            println!("Pushing newline");
            line_buf.push(b'\n');
            out_buf.push(String::from_utf8(line_buf.clone()).unwrap());
            line_buf.clear();
            println!("Length now {}", line_buf.len());

            line_buf.extend(seg);
        }
    }
    out_buf
}
impl PPMReader<'_> {
    pub fn new(canvas: &Canvas) -> std::io::Result<PPMReader<'_>> {
        let mut inner_buf = vec![];

        let header = PPMHeader {
            height: canvas.height,
            width: canvas.width,
            ..Default::default()
        };
        let header_written = PPMHeaderReader::new(&header).read_to_end(&mut inner_buf)?;

        // println!("Header written {header_written}");
        let mut written = header_written;
        // let mut line_length = 0;
        let canvas_chunks = canvas.data.as_ref().chunks(canvas.width);

        let mut chunk_buffer: Vec<Vec<u8>> = vec![Default::default(); canvas.width];
        for (chunk_idx, line_colors) in canvas_chunks.into_iter().enumerate() {
            let mut line_buffer = vec![];

            for Color(value) in line_colors {
                let r = format!(
                    "{} ",
                    (value.0[0].clamp(0., 1.) * header.color_scale as f32).round()
                );
                let r_src = r.into_bytes();

                line_buffer.extend_from_slice(&r_src);
                written += r_src.len();

                let g = format!(
                    "{} ",
                    (value.0[1].clamp(0., 1.) * header.color_scale as f32).round()
                );
                let g_src = g.into_bytes();

                line_buffer.extend_from_slice(&g_src);
                written += g_src.len();

                let b = format!(
                    "{} ",
                    (value.0[2].clamp(0., 1.) * header.color_scale as f32).round()
                );
                let b_src = b.into_bytes();

                line_buffer.extend_from_slice(&b_src);
                written += b_src.len();
                // println!("Line length {}, Line buffer written {}", written, len);
            }

            if line_buffer.len() > 70 {
                let mut nl_idx = 70;
                println!("Finished with buffering, grouping long line");
                // Replace last in line buffer
                loop {
                    if let Some(value) = line_buffer.get_mut(nl_idx) {
                        if *value == b' ' {
                            println!("Found suitable NL at {nl_idx}!");
                            *value = b'\n';
                            break;
                        }
                        nl_idx -= 1;
                    } else {
                        break;
                    }
                }
            }

            if let Some(last) = line_buffer.last_mut() {
                if *last == b' ' {
                    *last = b'\n';
                }
            }
            chunk_buffer[chunk_idx].extend_from_slice(line_buffer.as_slice());
            line_buffer.clear();
        }
        // let lines = group_by_line_length(&line_buffer);
        inner_buf.extend(chunk_buffer.into_iter().flat_map(core::convert::identity));

        Ok(PPMReader {
            canvas,
            written,
            read: 0,
            inner_buf,
        })
    }
}
#[cfg(test)]
mod ppm_tests {
    use std::ffi::{CStr, CString};

    use super::*;

    #[test]
    fn constructing_the_ppm_header() {
        let c = Canvas::new(5, 3);
        let mut ppm_reader = PPMReader::new(&c).unwrap();
        let mut buf = vec![];
        ppm_reader.read_to_end(&mut buf);
        let st = String::from_utf8(buf).unwrap();
        let expected = r#"P3
5 3
255
"#;
        assert_eq!(&st[..expected.len()], expected);
        println!("{st}")
    }
    #[test]
    fn constructing_the_ppm_pixel_data() {
        let mut c = Canvas::new(5, 3);

        let c1 = Color(Vert3([1.5, 0., 0.]));
        c.write_pixel((0, 0), c1);

        let c2 = Color(Vert3([0., 0.5, 0.]));
        c.write_pixel((2, 1), c2);

        let c3 = Color(Vert3([-0.5, 0., 1.0]));
        c.write_pixel((4, 2), c3);

        let mut ppm_reader = PPMReader::new(&c).unwrap();
        let mut buf = vec![];

        ppm_reader.read_to_end(&mut buf);

        let st = String::from_utf8(buf).unwrap();
        let expected = r#"P3
5 3
255
255 0 0 0 0 0 0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 128 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0 0 0 0 0 0 255
"#;
        assert_eq!(&st[..expected.len()], expected);
        println!("{st}")
    }
    #[test]
    fn splitting_long_lines_in_ppm_files() {
        let mut c = Canvas::new(10, 2);
        for x in 0..10 {
            for y in 0..2 {
                c.write_pixel((x, y), Color(Vert3([1., 0.8, 0.6])));
            }
        }

        let mut ppm_reader = PPMReader::new(&c).unwrap();
        let mut buf = vec![];

        ppm_reader.read_to_end(&mut buf);

        let st = String::from_utf8(buf).unwrap();
        let expected = r#"P3
10 2
255
255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204
153 255 204 153 255 204 153 255 204 153 255 204 153
255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204
153 255 204 153 255 204 153 255 204 153 255 204 153
"#;
        assert_eq!(&st[..expected.len()], expected);
        // println!("{st}")
    }
    #[test]
    fn ppm_files_are_terminated_by_a_newline_character() {
        let c = Canvas::new(5, 3);
        let mut ppm_reader = PPMReader::new(&c).unwrap();
        let mut buf = vec![];
        ppm_reader.read_to_end(&mut buf);
        let st = String::from_utf8(buf).unwrap();
        println!("{st:?}");
        assert_eq!(
            st.into_bytes()
                .as_slice()
                .iter()
                .filter(|byte| *byte != &0_u8)
                .last()
                .unwrap(),
            &b'\n'
        );
    }
}
