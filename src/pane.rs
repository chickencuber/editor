use mlua::UserData;
use sdl2::{pixels::{Color, PixelFormatEnum}, rect::Rect, render::{BlendMode, RenderTarget, Texture, TextureCreator}, surface::Surface, ttf::FontStyle, video::WindowContext
};

type Canvas = sdl2::render::Canvas<sdl2::video::Window>;
type TC = TextureCreator<WindowContext>;

use crate::{font::{Fonts, Font}, Config};

pub struct Line {
    pub cells: Vec<TextCell>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

impl From<char> for Mode {
    fn from(c: char) -> Self {
        match c.to_lowercase().collect::<Vec<char>>()[0] {
            'i' => Self::Insert,
            'n' => Self::Normal,
            'v' => Self::Visual,
            _ => panic!("mode doesn't exist"),
        }
    }
}

impl Line {
    pub fn render(&self, canvas: &mut Canvas, y: &mut i32, fonts: &mut Fonts, cursor: &Cursor, l: usize) {
        let mut height = 0;
        let mut x = 0;
        let mut c = 0;
        for ch in self.cells.iter() {
            let ox = x;
            let mut oh = 0;
            ch.render(canvas, *y, &mut x, &mut height, fonts, &mut oh);
            if cursor.y as usize == l && cursor.x as usize == c {
                cursor.cursor_type.render(ox, (x-ox) as u32, *y, oh, canvas);
            }
            c+=1;
        }
        *y += height as i32;
    }
}

pub struct TextCell {
    pub char: char, 
    pub fg: Color,
    pub bg: Option<Color>,
    pub font: Font,
    pub font_style: FontStyle,
}

impl TextCell {
    pub fn render(&self, canvas: &mut Canvas, y: i32, x: &mut i32, height: &mut u32, fonts: &mut Fonts, ch: &mut u32) {
        let font = fonts.load_font(&self.font);
        font.set_style(self.font_style);
        let (w, h) = font.size_of_char(self.char).unwrap();
        *ch = h;
        if h > *height  {
            *height = h;
        }

        let tc = canvas.texture_creator();
        let s = font.render_char(self.char).solid(self.fg).unwrap(); 
        let tex = Texture::from_surface(&s, &tc).unwrap();
        let rect = Rect::new(*x, y, w, h);

        if let Some(bg) = self.bg {
            canvas.set_draw_color(bg);
            canvas.fill_rect(rect).unwrap();
        }

        canvas.copy(&tex, Rect::new(0, 0, w, h), rect).unwrap();

        *x+=w as i32;
    }
}

pub enum CursorType {
    Block,
    Line,
    Underline,
}

impl CursorType {
    fn render(&self, x:i32, w:u32, y:i32, h:u32, canvas: &mut Canvas) {
        canvas.set_draw_color(Color::WHITE);
        match self {
            Self::Block => {
                let rect = Rect::new(x, y, w, h);
                let _ = canvas.fill_rect(rect);

            }
            Self::Line => {
                let rect = Rect::new(x, y, 2, h);
                let _ = canvas.fill_rect(rect);

            }
            Self::Underline => {
                let rect = Rect::new(x, y + h as i32 - 2, w, 2);
                let _ = canvas.fill_rect(rect);
            }
        }
    }
}

pub struct Cursor {
    pub x: u32,
    pub y: u32,
    pub cursor_type: CursorType,
}

pub struct TextBufOptions {

}
impl TextBufOptions {
    fn new() -> Self {
        Self {

        }
    }
}

pub enum BufType {
    Text{buf: Vec<Line>, cursor: Cursor, opts: TextBufOptions}
}

//TASK(20260111-161006-254-n6-036): actually implement floating panes
pub struct Pane {
    pub rect: Rect, 
    pub z_index: u32, //if z-index is 0, then its a tiled widnow, otherwise it floats
    pub buf: BufType,
    pub bg: Color,
}

//TASK(20260111-161148-304-n6-294): make tiled panes automatically change width height and position

impl Pane {
    pub fn text(rect: Rect, z_index: u32, bg: Color) -> Self {
        Self {
            rect,
            z_index,
            bg,
            buf: BufType::Text{
                buf: Vec::new(),
                cursor: Cursor {
                    x: 0,
                    y: 0,
                    cursor_type: CursorType::Block
                },
                opts: TextBufOptions::new()
            },
        }
    }
    pub fn render(&self, canvas: &mut Canvas, fonts: &mut Fonts) {
        match &self.buf {
            BufType::Text{buf, cursor, ..} => {
                canvas.set_clip_rect(self.rect);
                canvas.set_draw_color(self.bg);
                canvas.fill_rect(self.rect).unwrap();

                let mut y = 0;
                let mut i = 0;
                for line in buf.iter() {
                    line.render(canvas, &mut y, fonts, cursor, i);
                    i+=1;
                }
            }
            _ => {
                todo!()
            }
        }
    }
    pub fn move_cursor(&mut self, x: u32, y: u32) {
        match &mut self.buf {
            BufType::Text{cursor, ..} => {
                cursor.x = x;
                cursor.y = y;
            }
            _ => panic!("pane not text buffer")
        }
    }
    pub fn set_cursor_style(&mut self, ty: CursorType) {

        match &mut self.buf {
            BufType::Text{cursor, ..} => {
                cursor.cursor_type = ty;
            }
            _ => panic!("pane not text buffer")
        }
    }
    pub fn get_cursor(&self) -> (u32, u32) {
        match &self.buf {
            BufType::Text{cursor, ..} => {
                (
                    cursor.x,
                    cursor.y
                )
            }
            _ => panic!("pane not text buffer")
        }
    }
    pub fn insert_char(&mut self, c: char, config: &Config, font: &mut Fonts) {
        match &mut self.buf {
            BufType::Text{buf, cursor, ..} => {
                while buf.len() <= cursor.y as usize {
                    buf.push(Line {
                        cells: Vec::new()
                    })
                }
                if c == '\n' {
                    cursor.x = 0;
                    buf.insert(cursor.y as usize, Line {
                       cells: Vec::new() 
                    });
                    cursor.y+=1;
                    return;     
                } 

                while buf[cursor.y as usize].cells.len() <= cursor.x as usize {
                    buf[cursor.y as usize].cells.push(
                        TextCell { 
                            char: ' ', 
                            fg: config.text,
                            bg: None,
                            font: (font.find_font(&[&config.monospace.clone()]), config.font_size),
                            font_style: FontStyle::NORMAL,
                        }
                    )
                }
                let mut insert = String::new();

                if c == '\t' {
                    if let Some(n) = config.tabs {
                        insert.push_str(&(" ".repeat(n)));
                    } else {
                        insert.push('\t');
                    }
                } else {
                    insert.push(c);
                }
                for ch in insert.chars() {
                    buf[cursor.y as usize].cells.insert(cursor.x as usize, TextCell { 
                        char: ch, 
                        fg: config.text,
                        bg: None,
                        font: (font.find_font(&[&config.monospace.clone()]), config.font_size),
                        font_style: FontStyle::NORMAL,
                    });
                }
            }
            _ => panic!("pane not text buffer")
        }
    }
    pub fn handle_events(&mut self, mode: &Mode, config: &Config, fonts: &Fonts) {
        match &mut self.buf {
            BufType::Text{..} => {
            }
            _ => panic!("pane not text buffer")
        }
    }
}

impl UserData for Pane {

}

