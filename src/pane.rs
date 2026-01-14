use mlua::UserData;
use sdl2::{keyboard::{Keycode, Mod}, pixels::{Color, PixelFormatEnum}, rect::Rect, render::{BlendMode, RenderTarget, Texture, TextureCreator}, surface::Surface, ttf::FontStyle, video::WindowContext
};

type Canvas = sdl2::render::Canvas<sdl2::video::Window>;

use crate::{font::{Fonts, Font}, Config};

#[derive(Debug)]
pub struct Line {
    pub cells: Vec<TextCell>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

impl Mode {
    pub fn to_char(&self) -> char {
        match self {
            Self::Insert => 'i',
            Self::Normal => 'n',
            Self::Visual => 'v',
        }
    }
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
    pub fn render(&self, canvas: &mut Canvas, y: &mut i32, fonts: &mut Fonts, cursor: &Cursor, l: usize, sx: i32, config:&Config) {
        let mut height = 0;
        let mut x = sx;
        let mut c = 0;
        if self.cells.len() == 0 {
            let f = fonts.find_font(&[&config.monospace]);
            let font = fonts.load_font(&(f, config.font_size));
            *y+=font.height() as i32; 
            return; 
        }
        for ch in self.cells.iter() {
            let ox = x;
            let mut inver = false;
            if cursor.y as usize == l && cursor.x as usize == c {
                let (w, h) = ch.size(fonts);
                cursor.cursor_type.render(ox, w as u32, *y, h, canvas, ch, &mut inver);
            }
            ch.render(canvas, *y, &mut x, &mut height, fonts, inver, config);
            c+=1;
        }
        *y += height as i32;
    }
}

#[derive(Debug)]
pub struct TextCell {
    pub char: char, 
    pub fg: Color,
    pub bg: Option<Color>,
    pub font: Font,
    pub font_style: FontStyle,
}

impl TextCell {
    pub fn size(&self, fonts: &mut Fonts) -> (u32, u32) {
        let font = fonts.load_font(&self.font);
        font.set_style(self.font_style);
        return font.size_of_char(self.char).unwrap();
    }
    pub fn render(&self, canvas: &mut Canvas, y: i32, x: &mut i32, height: &mut u32, fonts: &mut Fonts, inver: bool, config: &Config) {
        let font = fonts.load_font(&self.font);
        font.set_style(self.font_style);
        let (w, h)= if self.char == '\t' {
            let (w, h) = font.size_of_char(' ').unwrap();
            (w*config.tab_display as u32, h)
        } else {
            font.size_of_char(self.char).unwrap()
        };
        if h > *height  {
            *height = h;
        }

        let tc = canvas.texture_creator();
        let mut s = None;
        if self.char != '\t' {
            if inver {
                s = Some(font.render_char(self.char).solid(self.bg.unwrap_or(config.bg)).unwrap());
            } else {
                s = Some(font.render_char(self.char).solid(self.fg).unwrap()); 
            }
        }
        let rect = Rect::new(*x, y, w, h);
        let mut tex = None;
        if let Some(s) = s {
            tex = Some(tc.create_texture_from_surface(&s).unwrap());
        }

        if !inver {
            if let Some(bg) = self.bg {
                canvas.set_draw_color(bg);
                canvas.fill_rect(rect).unwrap();
            }
        } else {
            canvas.set_draw_color(self.fg);
            canvas.fill_rect(rect).unwrap();
        }

        if let Some(tex) = tex {
            canvas.copy(&tex, None, rect).unwrap();
        }

        *x+=w as i32;
    }
}

pub enum CursorType {
    Block,
    Line,
    Underline,
}

impl CursorType {
    fn render(&self, x:i32, w:u32, y:i32, h:u32, canvas: &mut Canvas, ch: &TextCell, inver: &mut bool) {
        canvas.set_draw_color(ch.fg);
        match self {
            Self::Block => {
                *inver = true;
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
    pub fn render(&self, canvas: &mut Canvas, fonts: &mut Fonts, config: &Config) {
        match &self.buf {
            BufType::Text{buf, cursor, ..} => {
                canvas.set_clip_rect(self.rect);
                canvas.set_draw_color(self.bg);
                canvas.fill_rect(self.rect).unwrap();

                let mut y = self.rect.x;
                let mut i = 0;
                for line in buf.iter() {
                    line.render(canvas, &mut y, fonts, cursor, i, self.rect.y, config);
                    i+=1;
                }
            }
            _ => {
                todo!()
            }
        }
    }
    pub fn set_cursor(&mut self, x: u32, y: u32) {
        match &mut self.buf {
            BufType::Text{cursor, ..} => {
                cursor.x = x;
                cursor.y = y;
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
    pub fn fix_cursor(&mut self, config: &Config, font: &mut Fonts) {
        match &mut self.buf {
            BufType::Text{buf, cursor, ..} => {
                while buf.len() <= cursor.y as usize {
                    buf.push(Line {
                        cells: Vec::new()
                    })
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
                cursor.cursor_type = match config.mode {
                    Mode::Normal=> CursorType::Block,
                    Mode::Insert=> CursorType::Line,
                    Mode::Visual=> CursorType::Block,
                }
            }
            _=>{}
        }
    }
    pub fn insert_char(&mut self, c: char, config: &Config, font: &mut Fonts) {
        match &mut self.buf {
            BufType::Text{buf, cursor, ..} => {
                if c == '\n' {
                    let c = buf[cursor.y as usize].cells.split_off(cursor.x as usize);
                    buf.insert(cursor.y as usize + 1, Line {
                        cells: c 
                    });
                    cursor.x = 0;
                    cursor.y +=1;
                    return;     
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
                    cursor.x+=1;
                }
            }
            _=>{}
        }
    }

    pub fn backspace(&mut self) {
        match &mut self.buf {
            BufType::Text{buf, cursor, ..} => {
                if cursor.x > 0 {
                    cursor.x-=1;
                    buf[cursor.y as usize].cells.remove(cursor.x as usize);
                } else if cursor.y > 0 {
                    let mut c = buf.remove(cursor.y as usize);
                    cursor.y -= 1;
                    cursor.x = buf[cursor.y as usize].cells.len() as u32;
                    buf[cursor.y as usize].cells.append(&mut c.cells);
                }
            }
            _=>{}
        }
    }
    pub fn delete_line(&mut self) {
        match &mut self.buf {
            BufType::Text{buf, cursor,..} => {
                buf.remove(cursor.y as usize);
                cursor.x = 0;
                if cursor.y >= buf.len() as u32 {
                    if cursor.y > 0 {
                        cursor.y-=1;
                    }
                }
            }
            _=>{}
        }
    }
    pub fn handle_events(&mut self, config: &mut Config, fonts: &mut Fonts, keycode: Keycode, keymod: Mod) {
        match &mut self.buf {
            BufType::Text{..} => {
                if let Mode::Insert = config.mode {
                    match keycode {
                        Keycode::TAB => {
                            self.insert_char('\t', config, fonts);
                            return;
                        }
                        Keycode::Space => {
                            self.insert_char(' ', config, fonts);
                            return;
                        }
                        Keycode::Return => {
                            self.insert_char('\n', config, fonts);
                            return
                        }
                        Keycode::BACKSPACE => {
                            self.backspace();
                            return;
                        }
                        _ => {
                            let mut str = format!("{}", keycode);
                            if str.len() == 1 {
                                let mut cap = keymod.intersects(Mod::RSHIFTMOD | Mod::LSHIFTMOD);
                                if keymod.intersects(Mod::CAPSMOD) {
                                    cap = !cap;
                                }
                                if cap {
                                    str = str.to_uppercase();
                                } else {
                                    str = str.to_lowercase();
                                }
                                self.insert_char(str.chars().nth(0).unwrap(), config, fonts);
                                return;
                            }
                        }
                    }
                }
                config.keymap.handle(config.mode.clone(), keycode, keymod);
            }
            _=>{}
        }
    }
}

impl UserData for Pane {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("set_cursor", |_, this, (x, y): (u32, u32)| {
            this.set_cursor(x, y);
            Ok(())
        });
        methods.add_method("get_cursor", |_, this, ()| {
            Ok(this.get_cursor())
        });

        methods.add_method_mut("delete_line", |_, this, ()| {
            Ok(this.delete_line())
        });
    }
}

//TASK(20260114-132528-029-n6-460): make visual mode work

