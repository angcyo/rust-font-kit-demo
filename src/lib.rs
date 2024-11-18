pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::Arc;
    use font_kit::loader::Loader;
    use super::*;

    macro_rules! ok(($result:expr) => ($result.unwrap()));

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_font() {
        let path = "tests/JetBrainsMono-Bold.ttf".to_string();
        //let bytes= read_file_bytes(&path);
        //font_is_single_otf(&bytes);
        let mut fonts = font::File::open(&path).unwrap();
        let len = fonts.len();
        let characters = fonts[0].characters().unwrap();
        let names = fonts[0].names();
        let glyph = ok!(ok!(fonts[0].glyph(' ')));
        let mut path_data = String::new();
        for contour in glyph.iter() {
            for segment in contour.iter() {
                match segment {
                    &font::glyph::Segment::Linear(offset) => {
                        path_data.push_str(&format!("M {} {}", offset.0, offset.1));
                    }
                    &font::glyph::Segment::Quadratic(o1, o2) => {
                        path_data.push_str(&format!("Q {} {} {} {}", o1.0, o1.1, o2.0, o2.1));
                    }
                    &font::glyph::Segment::Cubic(o1, o2, o3) => {
                        path_data.push_str(&format!("C {} {} {} {} {} {}", o1.0, o1.1, o2.0, o2.1, o3.0, o3.1));
                    }
                }
            }
        }
        println!("path_data:{}", path_data);
    }

    #[test]
    fn test_ttf_parser() {
        //let path = "tests/JetBrainsMono-Bold.ttf".to_string(); //不支持中文
        //let path = "tests/微软雅黑-Bold.ttc".to_string();
        let path = "tests/SourceHanSansCN-Normal.otf".to_string();
        let bytes = read_file_bytes(&path);
        let face = owned_ttf_parser::Face::parse(&bytes, 0).unwrap();
        println!("face glyphs number:{:?}", face.number_of_glyphs());
        //let glyph_id = face.glyph_index('&').unwrap();
        /*let glyph_id = face.glyph_index('我').unwrap();
        // Extract glyph outline.
        let mut outline = SvgOutline::new();
        face.outline_glyph(glyph_id, &mut outline).unwrap();
        let path = outline.to_svg_path();
        println!("path:{}", path);*/

        for text in "中国人?.".chars() {
            let path = ttf_glyph_to_svg(&face, &text.to_string());
            println!("{}->{}", text, path);

            std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(format!("tests/output/ttf_parser_{}.txt", text))
                .unwrap()
                .write_all(path.as_bytes()).unwrap();
        }
    }

    fn ttf_glyph_to_svg(face: &owned_ttf_parser::Face, char: &String) -> String {
        let glyph_id = face.glyph_index(char.chars().next().unwrap()).unwrap();
        // Extract glyph outline.
        let mut outline = SvgOutline::new();
        face.outline_glyph(glyph_id, &mut outline).unwrap();
        let path = outline.to_svg_path();
        path
    }

    #[test]
    fn test_fontkit() {
        let path = "tests/JetBrainsMono-Bold.ttf".to_string(); //不支持中文
        //let path = "tests/微软雅黑-Bold.ttc".to_string();
        let bytes = read_file_bytes(&path);
        println!("is_otf:{:?}", fontkit::is_otf(&bytes));
        println!("is_ttf:{:?}", fontkit::is_ttf(&bytes));
        println!("is_woff:{:?}", fontkit::is_woff(&bytes));
        println!("is_woff2:{:?}", fontkit::is_woff2(&bytes));

        let font_kit = fontkit::FontKit::new();
        let font_keys = font_kit.add_font_from_buffer(bytes).unwrap();
        let font = font_kit.query(&font_keys[0]).unwrap();
        let glyph = font.outline('A').unwrap();
        println!("glyph:{:?} {:?}", glyph.0.path, glyph.1); //输出的垂直镜像的数据
    }

    /// 需要[pathfinder_geometry]库支持
    #[test]
    fn test_font_kit() {
        //let path = "tests/JetBrainsMono-Bold.ttf".to_string(); //不支持中文
        let path = "tests/微软雅黑-Bold.ttc".to_string();
        let bytes = read_file_bytes(&path);
        //font_kit::source::SystemSource::new(&bytes, 0).unwrap();
        let font = font_kit::font::Font::from_bytes(Arc::new(bytes), 0).unwrap();
        let count = font.glyph_count();
        println!("glyph count:{:?}", count);

        for text in "中国人?".chars() {
            let glyph_id = font.glyph_for_char(text).unwrap();

            let mut outline = SvgOutline::new();
            let glyph = font.outline(glyph_id, font_kit::hinting::HintingOptions::None, &mut outline).unwrap();
            let path = outline.to_svg_path();
            println!("path:{}", path);

            std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(format!("tests/output/font_kit_{}.txt", text))
                .unwrap()
                .write_all(path.as_bytes()).unwrap();
        }
    }

    /// 读取文件字节数据
    fn read_file_bytes(path: &String) -> Vec<u8> {
        //println!("{:?}", std::env::current_dir());
        let mut f = std::fs::File::open(path).unwrap();
        let mut buffer = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut buffer).unwrap();
        println!("文件字节大小:{:?}", buffer.len());
        buffer
    }

    /// A simple struct to collect paths from a glyph.
    struct SvgOutline {
        paths: Vec<String>,
    }

    impl SvgOutline {
        fn new() -> Self {
            SvgOutline { paths: Vec::new() }
        }

        fn to_svg_path(&self) -> String {
            self.paths.join(" ")
        }
    }

    /// Implement `OutlineBuilder` for SvgOutline to collect TTF paths.
    /// y轴方向是向下的，需要转换为向上
    impl owned_ttf_parser::OutlineBuilder for SvgOutline {
        fn move_to(&mut self, x: f32, y: f32) {
            self.paths.push(format!("M{} {}", x, -y));
            // self.paths.push(format!("M{} {}", x, y));
        }

        fn line_to(&mut self, x: f32, y: f32) {
            self.paths.push(format!("L{} {}", x, -y));
            // self.paths.push(format!("L{} {}", x, y));
        }

        fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
            self.paths.push(format!("Q{} {} {} {}", x1, -y1, x, -y));
            // self.paths.push(format!("Q{} {} {} {}", x1, y1, x, y));
        }

        fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
            self.paths.push(format!("C{} {} {} {} {} {}", x1, -y1, x2, -y2, x, -y));
            // self.paths.push(format!("C{} {} {} {} {} {}", x1, y1, x2, -y2, x, y));
        }

        fn close(&mut self) {
            self.paths.push("Z".to_string());
        }
    }

    impl font_kit::outline::OutlineSink for SvgOutline {
        fn move_to(&mut self, to: pathfinder_geometry::vector::Vector2F) {
            self.paths.push(format!("M{} {}", to.x(), -to.y()));
        }

        fn line_to(&mut self, to: pathfinder_geometry::vector::Vector2F) {
            self.paths.push(format!("L{} {}", to.x(), -to.y()));
        }

        fn quadratic_curve_to(&mut self, ctrl: pathfinder_geometry::vector::Vector2F, to: pathfinder_geometry::vector::Vector2F) {
            self.paths.push(format!("Q{} {} {} {}", ctrl.x(), -ctrl.y(), to.x(), -to.y()));
        }

        fn cubic_curve_to(&mut self, ctrl: pathfinder_geometry::line_segment::LineSegment2F, to: pathfinder_geometry::vector::Vector2F) {
            self.paths.push(format!("C{} {} {} {} {} {}", ctrl.from().x(), -ctrl.from().y(), ctrl.to().x(), -ctrl.to().y(), to.x(), -to.y()));
        }

        fn close(&mut self) {
            self.paths.push("Z".to_string());
        }
    }
}
