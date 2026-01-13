//font manager
pub type Font = (PathBuf, u16);
use fontconfig::Fontconfig;
use std::collections::HashMap;
use std::path::PathBuf;



pub struct Fonts {
    ttf: &'static sdl2::ttf::Sdl2TtfContext,
    fonts: HashMap<Font, sdl2::ttf::Font<'static, 'static>>,
    paths: HashMap<String, PathBuf>,
}


impl Fonts {

    pub fn new() -> Self {
        let ttf = Box::leak(Box::new(sdl2::ttf::init().unwrap()));

        Self {
            ttf,
            fonts: HashMap::new(),
            paths: HashMap::new(),
        }
    }
    pub fn find_font(&mut self, candidates: &[&str]) -> PathBuf {
        let fc = Fontconfig::new().unwrap();
        for &name in candidates {
            if self.paths.contains_key(name) {
                return self.paths.get(name).unwrap().clone();
            }
            if let Some(path) = fc.find(name, None) {
                println!("found font {name}");
                self.paths.insert(name.to_string(), path.path.clone());
                return path.path;
            }
        }
        panic!("no suitable font found from {:?}", candidates);
    }

    pub fn find_font_exists(&mut self, candidates: &[&str]) -> String {
        let fc = Fontconfig::new().unwrap();
        for &name in candidates {
            if self.paths.contains_key(name) {
                return name.to_string();
            }
            if let Some(path) = fc.find(name, None) {
                println!("found font {name}");
                self.paths.insert(name.to_string(), path.path);
                return name.to_string();
            }
        }
        panic!("no suitable font found from {:?}", candidates);
    }
    pub fn load_font(&mut self, font: &Font) -> &mut sdl2::ttf::Font<'static, 'static> {
        let (path, size) = font;

        if !self.fonts.contains_key(&font) {
            let fontf = self.ttf.load_font(path, *size).unwrap();
            self.fonts.insert(font.clone(), fontf);
        }

        self.fonts.get_mut(&font).unwrap()
    }
}
