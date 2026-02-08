use handlebars::Handlebars;
use resvg::usvg::Options;
use resvg::{render, tiny_skia::Pixmap, usvg::Tree};
use std::collections::HashMap;

static SVG_FILE: &str = include_str!("./source.svg.template");

enum RenderDataTypes {
    Int(i32),
    FloatList(Vec<f32>),
}

impl serde::Serialize for RenderDataTypes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::FloatList(flt) => flt.serialize(serializer),
            Self::Int(int) => int.serialize(serializer),
        }
    }
}

pub fn render_progress_clock(segments: u8) -> Result<(), String> {
    let mut handlebars = Handlebars::new();

    handlebars
        .register_template_string("progress_clock", SVG_FILE)
        .map_err(|e| e.to_string())?;

    let angle_segment: f32 = 360f32 / f32::from(segments);
    let mut angles: Vec<f32> = vec![angle_segment];
    for i in 0..segments {
        angles.push(angle_segment * f32::from(i));
    }

    let mut render_data = HashMap::new();
    render_data.insert("angle", RenderDataTypes::FloatList(angles));
    render_data.insert("height", RenderDataTypes::Int(100));
    render_data.insert("width", RenderDataTypes::Int(100));

    let svg_source = handlebars
        .render("progress_clock", &render_data)
        .map_err(|e| e.to_string())?;

    println!("{}", svg_source);

    let mut pixmap = Pixmap::new(200, 200).expect("Could not get mutable pixmap.");
    render(
        &Tree::from_data(&svg_source.into_bytes(), &Options::default())
            .expect("Could not build tree from SVG source"),
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );

    pixmap.save_png("./out.png").map_err(|e| e.to_string())
}
