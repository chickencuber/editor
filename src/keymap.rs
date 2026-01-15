use std::{collections::HashMap, time::{Duration, Instant}};

use mlua::Function;
use sdl2::keyboard::{Keycode, Mod};

use crate::pane::Mode;

#[derive(Debug, Clone)]
pub enum Action {
    Function(Function),
    Macro(String),
}

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
        "bs"=> Key{
            key: Keys::Backspace,
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
        "left"=>Key {
            key:Keys::Left,
            ..Default::default()
        },
        "right"=>Key {
            key:Keys::Right,
            ..Default::default()
        },
        "up"=>Key {
            key:Keys::Up,
            ..Default::default()
        },
        "down"=>Key {
            key:Keys::Down,
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

type Event = (Keycode, Mod, Option<String>, bool);

impl Key {
    fn finish() -> Self {
        Self {
            alt: false,
            shift: false,
            ctrl: false,
            key: Keys::Finish,
        }
    }
    fn to_event(&self) -> Event {
        let mut keymod = Mod::NOMOD;
        if self.alt {
            keymod = keymod|Mod::LALTMOD;    
        } 
        if self.shift {
            keymod = keymod|Mod::LSHIFTMOD;    
        } 
        if self.ctrl{
            keymod = keymod|Mod::LCTRLMOD;    
        } 
        let mut text = None;
        let mut finish = false;
        let keycode = match self.key {
            Keys::Esc => Keycode::ESCAPE,
            Keys::Tab => Keycode::TAB,
            Keys::CR => Keycode::Return,
            Keys::Up => Keycode::Up,
            Keys::Left => Keycode::Left,
            Keys::Right => Keycode::Right,
            Keys::Down => Keycode::Down,
            Keys::Backspace => Keycode::Backspace,
            Keys::Unknown => panic!("nope"),
            Keys::Finish => {
                finish = true;
                Keycode::KP_0
            },
            Keys::Char(' ') => Keycode::Space,
            Keys::Char(c) => {
                text = Some(c.to_string());
                if let Some(s) = Keycode::from_name(&c.to_string()) {
                    s
                } else {
                    Keycode::KP_0
                }
            }
        };
        return (keycode, keymod, text, finish);
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Keys {
    Char(char),
    Esc,
    Tab,
    CR,
    Unknown,
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Finish,
}


impl Keys {
    fn from(value: Keycode, text: Option<String>) -> Self {
       match value {
           Keycode::Tab => Keys::Tab,
           Keycode::Escape => Keys::Esc,
           Keycode::RETURN => Keys::CR,
           Keycode::Space=>Keys::Char(' '),
           Keycode::Left=>Keys::Left,
           Keycode::Right=>Keys::Right,
           Keycode::Up=>Keys::Up,
           Keycode::BACKSPACE => Keys::Backspace,
           Keycode::Down=>Keys::Down,
           _ => {
               if let Some(c) = text {
                   return Keys::Char(c.to_lowercase().chars().nth(0).unwrap());
               }
               return Self::Unknown;
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
    pub count: String,
    pub events: Vec<Event>,
}

impl Keymaps {
    pub fn new() -> Self {
        Self {
            keymaps: HashMap::new(),
            last: None,
            pos: Vec::new(),
            events: Vec::new(),
            count:String::new(),
        }
    }
    pub fn set(&mut self, mode: String, keys: String, func: Action, leader: char) {
        let modes: Vec<Mode> = mode.chars().map(Mode::from).collect();
        for mode in modes {
            if !self.keymaps.contains_key(&mode) {
                self.keymaps.insert(mode.clone(), Keymap::new());
            } 
            self.keymaps.get_mut(&mode).unwrap().set(keys.clone(), func.clone(), leader);
        }
    }
    //TASK(20260112-210317-316-n6-047): make leader work
    pub fn handle(&mut self, mode: Mode, key: Keycode, keymod: Mod, text: Option<String>, finish: bool) {
        let ctrl  = keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD);
        let mut shift = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
        let alt   = keymod.intersects(Mod::LALTMOD | Mod::RALTMOD);

        if keymod.intersects(Mod::CAPSMOD) {
            shift = !shift;
        }

        let key = Key {
            key: Keys::from(key, text),
            ctrl,
            shift,
            alt,
        };
        if let Keys::Char(v) = key.key {
            if v.is_numeric() {
                if !(v == '0' && self.count.len() == 0) {
                    self.count.push(v); 
                    self.last = Some(Instant::now());
                    return;
                }
            }
        }

        if self.pos.is_empty() {
            self.last = Some(Instant::now());
        }

        self.pos.push(key);

        let Some(mut s) = self.keymaps.get(&mode) else {
            self.pos.clear();
            self.count = "".to_string();
            self.last = None;
            return;
        };

        let mut action_to_call = None;
        let mut exit = false;

        for p in self.pos.iter_mut() {
            if (!s.child.contains_key(p)) || finish {
                p.alt = false;
                p.shift = false;
                p.ctrl = false;
                if (!s.child.contains_key(p)) || finish {
                    exit = true;
                    action_to_call = s.action.clone();
                    break;
                }
            }
            s = s.child.get(p).unwrap();
        }

        if s.child.is_empty() {
            action_to_call = s.action.clone();
            self.pos.clear();
            self.last = None;
            exit = false;
        }

        if let Some(func) = action_to_call {
            self.pos.clear();
            self.last = None;
            exit = false;
            match func {
                Action::Function(f) => {
                    f.call::<()>(()).unwrap();
                }
                Action::Macro(m) => {
                    self.call_macro(m);
                }
            }
            self.count = "".to_string();
        }
        if exit {
            self.pos.clear();
            self.count = "".to_string();
            self.last = None;
        }
    }
    pub fn call_macro(&mut self, m: String) {
        let keys = parse_keys(&m, ' ');
        let mut event: Vec<Event> = keys.iter().map(|v| v.to_event()).collect();
        event.push(Key::finish().to_event());
        self.events.splice(0..0, event);
    }

    pub fn handle_timeout(&mut self, mode: Mode, timeout: u64) {
        let Some(start_time) = self.last else { return; };

        let Some(mut s) = self.keymaps.get(&mode) else {
            self.pos.clear();
            self.count = "".to_string();
            self.last = None;
            return;
        };

        if Instant::now() < start_time + Duration::from_millis(timeout) {
            return;
        }

        let mut action_to_call = None;

        for p in self.pos.iter_mut() {
            if !s.child.contains_key(p) {
                p.alt = false;
                p.shift = false;
                p.ctrl = false;
                if !s.child.contains_key(p) {
                    action_to_call = s.action.clone();
                    break;
                }
            }
            s = s.child.get(p).unwrap();
        }

        if s.child.is_empty() {
            action_to_call = s.action.clone();
        }

        self.pos.clear();
        self.last = None;

        if let Some(func) = action_to_call {
            match func {
                Action::Function(f) => {
                    f.call::<()>(()).unwrap();
                }
                Action::Macro(m) => {
                    self.call_macro(m);
                }
            }
        }
        self.count = "".to_string();
    }
}


#[derive(Debug)]
pub struct Keymap {
    action: Option<Action>, 
    child: HashMap<Key, Self>,
}


impl Keymap {
    pub fn new() -> Self {
        Self {
            action: None,
            child: HashMap::new(),
        }
    }
    pub fn set(&mut self, keys: String, func: Action, leader: char) {
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
