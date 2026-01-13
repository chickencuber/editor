use std::{collections::HashMap, time::{Duration, Instant}};

use mlua::Function;
use sdl2::keyboard::{Keycode, Mod};

use crate::pane::Mode;

pub fn parse_keys(input: &str, leader:char) -> Vec<Key> {
    let mut out = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            let mut name = String::new();
            while let Some(&ch) = chars.peek() {
                chars.next();
                if ch == '>' {
                    break;
                }
                name.push(ch);
            }
            out.push(parse_section(&name.to_lowercase(), leader))
        } else {
            out.push(Key{
                key: Keys::Char(c.to_lowercase().nth(0).unwrap()),
                shift: c.is_uppercase(),
                ..Default::default()
            })
        }
    }

    return out
}

pub fn parse_section(name: &str, leader: char) -> Key {
    match name {
        "gt"=> Key{
            key: Keys::Char('>'),
            ..Default::default()
        },
        "lt"=> Key{
            key:Keys::Char('<'),
            ..Default::default()
        },
        "tab"=> Key{
            key:Keys::Tab,
            ..Default::default()
        },
        "cr"=> Key{
            key:Keys::CR,
            ..Default::default()
        },
        "esc"=> Key{
            key:Keys::Esc,
            ..Default::default()
        },
        "space"=> Key {
            key:Keys::Char(' '),
            ..Default::default()
        },
        "leader"=>Key {
            key:Keys::Char(leader),
            ..Default::default()
        },
        other => {
            if name.len() != 1 {
                let mut c = false;
                let mut s = false;
                let mut a = false;
                if name.starts_with("c-") {
                   c = true; 
                }
                if name.starts_with("s-") {
                   s = true; 
                }
                if name.starts_with("a-") || name.starts_with("m-") {
                    a=true 
                }
                if c||s||a {
                    let rest = name.chars().skip(2).collect::<String>();
                    let mut key = parse_section(&rest, leader);
                    key.alt = key.alt || a;
                    key.shift = key.shift || s;
                    key.ctrl = key.ctrl || c;
                    return key;
                }
                panic!("unknown key <{}>", other)
            }
            Key {
                key:Keys::Char(name.chars().nth(0).unwrap()),
                ..Default::default()
            }

        },
    }
}

#[derive(Hash, Eq, PartialEq, Default, Clone, Debug)]
pub struct Key{
    alt: bool,
    shift: bool,
    ctrl: bool,
    key: Keys,

}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Keys {
    Char(char),
    Esc,
    Tab,
    CR,
    Unknown,
}


impl From<Keycode> for Keys {
    fn from(value: Keycode) -> Self {
       match value {
           Keycode::Tab => Keys::Tab,
           Keycode::Escape => Keys::Esc,
           Keycode::RETURN => Keys::CR,
           Keycode::Space=>Keys::Char(' '),
           _ => {
               let c = format!("{}", value);
               if c.len() > 1 {
                   return Self::Unknown;
               }
               return Keys::Char(c.to_lowercase().chars().nth(0).unwrap());
           }
       } 
    }
}

impl Default for Keys {
    fn default() -> Self {
        Self::Esc
    }
}


#[derive(Debug)]
pub struct Keymaps {
    keymaps: HashMap<Mode, Keymap>,
    last: Option<Instant>,
    pos: Vec<Key>,
}

impl Keymaps {
    pub fn new() -> Self {
        Self {
            keymaps: HashMap::new(),
            last: None,
            pos: Vec::new(),
        }
    }
    pub fn set(&mut self, mode: String, keys: String, func: Function, leader: char) {
        let modes: Vec<Mode> = mode.chars().map(Mode::from).collect();
        for mode in modes {
            if !self.keymaps.contains_key(&mode) {
                self.keymaps.insert(mode.clone(), Keymap::new());
            } 
            self.keymaps.get_mut(&mode).unwrap().set(keys.clone(), func.clone(), leader);
        }
    }
    //TASK(20260112-210317-316-n6-047): make leader work
    pub fn handle(&mut self, mode: Mode, key: Keycode, keymod: Mod) {
        let ctrl  = keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD);
        let shift = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
        let alt   = keymod.intersects(Mod::LALTMOD | Mod::RALTMOD);

        let key = Key {
            key: Keys::from(key),
            ctrl,
            shift,
            alt,
        };

        if self.pos.is_empty() {
            self.last = Some(Instant::now());
        }

        self.pos.push(key);

        let Some(mut s) = self.keymaps.get(&mode) else {
            self.pos.clear();
            self.last = None;
            return;
        };

        let mut action_to_call = None;

        for p in self.pos.iter() {
            if !s.child.contains_key(p) {
                action_to_call = s.action.clone();
                break;
            }
            s = s.child.get(p).unwrap();
        }

        if s.child.is_empty() {
            action_to_call = s.action.clone();
            self.pos.clear();
            self.last = None;
        }

        if let Some(func) = action_to_call {
            self.pos.clear();
            self.last = None;
            func.call::<()>(()).unwrap();
        }
    }

    pub fn handle_timeout(&mut self, mode: Mode, timeout: u64) {
        let Some(start_time) = self.last else { return; };

        let Some(mut s) = self.keymaps.get(&mode) else {
            self.pos.clear();
            self.last = None;
            return;
        };

        if Instant::now() < start_time + Duration::from_millis(timeout) {
            return;
        }

        let mut action_to_call = None;

        for p in self.pos.iter() {
            if !s.child.contains_key(p) {
                action_to_call = s.action.clone();
                break;
            }
            s = s.child.get(p).unwrap();
        }

        if s.child.is_empty() {
            action_to_call = s.action.clone();
        }

        self.pos.clear();
        self.last = None;

        if let Some(func) = action_to_call {
            func.call::<()>(()).unwrap();
        }
    }
}


#[derive(Debug)]
pub struct Keymap {
    action: Option<Function>, 
    child: HashMap<Key, Self>,
}


impl Keymap {
    pub fn new() -> Self {
        Self {
            action: None,
            child: HashMap::new(),
        }
    }
    pub fn set(&mut self, keys: String, func: Function, leader: char) {
        let keys = parse_keys(&keys, leader);
        if keys.len() == 0 {
            return
        }
        let mut s = self;
        for key in keys {
            if !s.child.contains_key(&key) {
                s.child.insert(key.clone(), Keymap::new()); 
            }
            s = s.child.get_mut(&key).unwrap();
        }
        s.action = Some(func);
    }
}
