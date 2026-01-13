mod pane;
mod font;
mod keymap;


use keymap::{Keymaps, parse_keys};

use std::{path::PathBuf, time::Duration};

use mlua::{Function, Lua, Result, UserData, Value};
use sdl2::{
    event::Event, keyboard::{Keycode, Mod}, pixels::{
        Color,
        PixelFormat,
        PixelFormatEnum,
    }, rect::Rect,
};

use font::Fonts;

use crate::pane::{Mode, Pane};

fn rgba(color: u32) -> Color {
    Color::from_u32(&PixelFormat::try_from(PixelFormatEnum::RGBA8888).unwrap(), color)
}

fn from_rgba(color: Color) -> u32{
    color.to_u32(&PixelFormat::try_from(PixelFormatEnum::RGBA8888).unwrap())
}

fn xdg_config_home() -> PathBuf {
    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(path);
    } else {
        let mut p = PathBuf::from(std::env::var("HOME").unwrap());
        p.push(".config");
        return p;
    }
}

struct Panes {
    panes: Vec<Pane>,
    current_pane: usize,
}

impl Panes {
    fn new() -> Self {
        Self{
            panes: Vec::new(),
            current_pane: 0,
        }
    }
}

struct Config {
    monospace: String,
    serif: String,
    sans_serif: String,

    font_size: u16,

    bg: Color,
    text: Color,

    tabs: Option<usize>,
    tab_display: usize,

    command_timeout: u64,

    leader: char,

    keymap: Keymaps,

    mode:Mode,
}

impl Config {
    fn new(fonts: &mut Fonts) -> Self {
        Self {
            monospace: fonts.find_font_exists(&[
                "DejaVu Sans Mono",
                "Liberation Mono",
                "Noto Sans Mono",
                "Monospace",
            ]),
            serif: fonts.find_font_exists(&[
                "Liberation Serif",
                "Noto Serif",
                "Times New Roman",
                "Serif",
            ]),
            sans_serif: fonts.find_font_exists(&[
                "Liberation Sans",
                "Noto Sans",
                "Arial",
                "Sans",
            ]),

            font_size: 18,

            bg: rgba(0x181818ff),
            text: rgba(0xffffffff),

            tabs: Some(4),
            tab_display: 4,

            command_timeout: 1000,
            leader: ' ',

            keymap: Keymaps::new(),

            mode: Mode::Normal,
        }
    }
}

impl UserData for Config {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("monospace", |_, this| Ok(this.monospace.clone()));
        fields.add_field_method_set("monospace", |_, this, value: String| {
            this.monospace = value;
            Ok(())
        });
        fields.add_field_method_get("serif", |_, this| Ok(this.serif.clone()));
        fields.add_field_method_set("serif", |_, this, value: String| {
            this.serif = value;
            Ok(())
        });
        fields.add_field_method_get("sans_serif", |_, this| Ok(this.sans_serif.clone()));
        fields.add_field_method_set("sans_serif", |_, this, value: String| {
            this.sans_serif = value;
            Ok(())
        });
        fields.add_field_method_get("font_size", |_, this| Ok(this.font_size));
        fields.add_field_method_set("font_size", |_, this, value: u16| {
            this.font_size = value;
            Ok(())
        });
        fields.add_field_method_get("bg", |_, this| Ok(from_rgba(this.bg)));
        fields.add_field_method_set("bg", |_, this, value: u32| {
            this.bg = rgba(value);
            Ok(())
        });
        fields.add_field_method_get("text", |_, this| Ok(from_rgba(this.text)));
        fields.add_field_method_set("text", |_, this, value: u32| {
            this.text = rgba(value);
            Ok(())
        });
        fields.add_field_method_get("tabs", |_, this| Ok(this.tabs));
        fields.add_field_method_set("tabs", |_, this, value: Option<usize>| {
            this.tabs = value;
            Ok(())
        });
        fields.add_field_method_get("tab_display", |_, this| Ok(this.tab_display));
        fields.add_field_method_set("tab_display", |_, this, value: usize| {
            this.tab_display = value;
            Ok(())
        });
        fields.add_field_method_get("command_timeout", |_, this| Ok(this.command_timeout));
        fields.add_field_method_set("command_timeout", |_, this, value: u64| {
            this.command_timeout = value;
            Ok(())
        });
        fields.add_field_method_get("leader", |_, this| Ok(this.leader));
        fields.add_field_method_set("leader", |_, this, value: char| {
            this.leader = value;
            Ok(())
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("key", |_, this, (mode, keys, func): (String, String, Function)| {
            this.keymap.set(mode, keys, func, this.leader);
            return Ok(());
        });
    }
}

pub fn main() {
    let lua = Lua::new();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut fonts = Fonts::new();
    let mut config = Config::new(&mut fonts);
    let mut panes = Panes::new();


    let window = video_subsystem.window("editor", 800, 600)
        .resizable()
        .build()
        .unwrap();

    panes.panes.push(Pane::text(Rect::new(0, 0, 1000, 1000), 0, config.bg));

    {
        let pane = panes.panes.first_mut().unwrap();
        pane.insert_char('a', &config, &mut fonts);
    }

    lua.scope(|scope| {
        let ud = scope.create_userdata_ref_mut(&mut config)?;
        lua.globals().set("config", ud)?;
        lua.load(include_str!("./default.lua")).exec()?;
        Ok(())
        }).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        canvas.set_draw_color(config.bg);
        canvas.clear();

        for pane in panes.panes.iter() {
            pane.render(&mut canvas, &mut fonts);
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                Event::KeyDown {keycode: Some(keycode), keymod,..} => {
                    config.keymap.handle(config.mode.clone(), keycode, keymod);
                },
                _ => {}
            }
        }
        config.keymap.handle_timeout(config.mode.clone(), config.command_timeout);
        canvas.present();
    }
}
