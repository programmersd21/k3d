use crate::renderer::Framebuffer;
use std::io::{self, Write};

pub fn supported() -> bool {
    std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("TERM")
            .map(|x| x.contains("kitty"))
            .unwrap_or(false)
}

pub struct Presenter {
    image: u32,
    placement: u32,
    encoded: String,
    buf: Vec<u8>,
}

impl Drop for Presenter {
    fn drop(&mut self) {
        let _ = self.clear();
    }
}

impl Presenter {
    pub fn new() -> Self {
        Self {
            image: 7,
            placement: 7,
            encoded: String::new(),
            buf: Vec::new(),
        }
    }

    pub fn present(&mut self, fb: &Framebuffer, columns: u16, rows: u16) -> io::Result<()> {
        let old_image = self.image;
        self.image = if self.image == 7 { 8 } else { 7 };
        self.placement = self.image;

        self.buf.clear();

        // Move cursor to top-left so the placement is anchored at (1,1).
        write!(self.buf, "\x1b[H")?;

        base64_into(&fb.pixels, &mut self.encoded);

        for (chunk, part) in self.encoded.as_bytes().chunks(4096).enumerate() {
            let more = usize::from((chunk + 1) * 4096 < self.encoded.len());
            if chunk == 0 {
                write!(
                    self.buf,
                    "\x1b_Ga=T,f=32,s={},v={},i={},p={},c={},r={},q=2,m={};",
                    fb.width, fb.height, self.image, self.placement, columns, rows, more
                )?;
            } else {
                write!(self.buf, "\x1b_Gm={};", more)?;
            }
            self.buf.extend_from_slice(part);
            write!(self.buf, "\x1b\\")?;
        }

        // Delete the previous image now that the new one is fully drawn and displayed.
        write!(self.buf, "\x1b_Ga=d,d=I,i={},q=2;\x1b\\", old_image)?;

        let mut out = io::stdout();
        out.write_all(&self.buf)?;
        out.flush()
    }

    pub fn clear(&self) -> io::Result<()> {
        let mut out = io::stdout();
        write!(out, "\x1b_Ga=d,d=I,i=7,q=2;\x1b\\")?;
        write!(out, "\x1b_Ga=d,d=I,i=8,q=2;\x1b\\")?;
        out.flush()
    }
}

fn base64_into(data: &[u8], output: &mut String) {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    output.clear();
    output.reserve((data.len().div_ceil(3) * 4).saturating_sub(output.capacity()));
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16
            | ((*c.get(1).unwrap_or(&0)) as u32) << 8
            | *c.get(2).unwrap_or(&0) as u32;
        output.push(T[(n >> 18 & 63) as usize] as char);
        output.push(T[(n >> 12 & 63) as usize] as char);
        output.push(if c.len() > 1 {
            T[(n >> 6 & 63) as usize] as char
        } else {
            '='
        });
        output.push(if c.len() > 2 {
            T[(n & 63) as usize] as char
        } else {
            '='
        });
    }
}
