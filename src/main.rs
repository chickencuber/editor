mod pane;
mod font;
mod keymap;

fn first<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.len() == 0 {
        return None;
    }
    return Some(vec.remove(0))
}

use keymap::{Keymaps, parse_keys};

use std::{path::PathBuf, time::Duration};

use mlua::{Error, Function, Lua, Result, UserData, Value};
use sdl2::{
    event::Event, keyboard::{Keycode, Mod}, pixels::{
        Color,
        PixelFormat,
        PixelFormatEnum,
    }, rect::Rect,
};

use font::Fonts;

use crate::{keymap::Action, pane::{Mode, Pane}};

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

impl UserData for Panes {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("current_pane", |_, this| Ok(this.current_pane+1));
        fields.add_field_method_set("current_pane", |_, this, v: usize| {
            this.current_pane = v-1;
            Ok(())
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("get", |lua, this, (mut i, fun): (usize, Function)| {
            if i == 0 {
                i = this.current_pane+1;
            }
            lua.scope(|scope| {
                let arg = scope.create_userdata_ref_mut(this.panes.get_mut(i-1).unwrap());
                fun.call::<()>(arg)?;
                Ok(())
            })?;
            Ok(())
        });

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

            font_size: 20,

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
        fields.add_field_method_get("mode", |_, this| Ok(this.mode.to_char()));
        fields.add_field_method_set("mode", |_, this, value: char| {
            this.mode = Mode::from(value);
            Ok(())
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("key", |_, this, (mode, keys, val): (String, Value, Value)| {
            let func = match val {
                Value::Function(f) => {
                    Action::Function(f)  
                }
                Value::String(f) => {
                    Action::Macro(f.to_string_lossy())  
                }
                _ => return Err(Error::FromLuaConversionError {
                    from: val.type_name(),
                    to: "Function or String".to_string(),
                    message: Some("val must be a string or a function".to_string()),
                }),
            };
            match keys {
                Value::String(s) => {
                    this.keymap.set(mode, s.to_string_lossy(), func, this.leader);
                }
                Value::Table(s) => {
                    let len = s.len()?;
                    for i in 1..=len {
                        let k:String = s.get(i)?; 
                        this.keymap.set(mode.clone(), k, func.clone(), this.leader);
                    }
                }
                _ => return Err(Error::FromLuaConversionError {
                    from: keys.type_name(),
                    to: "String or Array of Strings".to_string(),
                    message: Some("keys must be a string or a table of strings".to_string()),
                }),
            }
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

    panes.panes.push(Pane::text(Rect::new(10, 10, 400, 400), 0, config.bg));

    lua.scope(|scope| {
        unsafe{
            let c = &mut config as *mut Config;
            let ud = scope.create_userdata_ref_mut(&mut *c)?;
            lua.globals().set("config", ud)?;

            let p = &mut panes as *mut Panes;
            let ud = scope.create_userdata_ref_mut(&mut *p)?;
            lua.globals().set("panes", ud)?;
        }
        lua.load(include_str!("./default.lua")).exec()?;
        let mut canvas = window.into_canvas().build().unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();

        'running: loop {
            canvas.set_draw_color(config.bg);
            canvas.clear();

            for pane in panes.panes.iter_mut() {
                pane.fix_cursor(&config, &mut fonts);
                pane.render(&mut canvas, &mut fonts, &config);
            }

            let pane = panes.panes.get_mut(panes.current_pane).unwrap();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => {
                        break 'running
                    },
                    Event::KeyDown {keycode: Some(keycode), keymod,..} => {
                        pane.handle_events(&mut config, &mut fonts, keycode, keymod);
                    },
                    _ => {}
                }
            }
            while let Some((keycode, keymod)) = first(&mut config.keymap.events) {
                pane.handle_events(&mut config, &mut fonts, keycode, keymod);
            }
            config.keymap.handle_timeout(config.mode.clone(), config.command_timeout);
            canvas.present();
        }
        Ok(())
    }).unwrap();
}
