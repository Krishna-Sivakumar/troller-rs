use handlebars::Handlebars;
use resvg::usvg::Options;
use resvg::{render, tiny_skia::Pixmap, usvg::Tree};
use std::collections::HashMap;
use std::f32::consts::PI;

static SVG_FILE: &str = include_str!("./source.svg");

#[derive(Debug)]
enum RenderDataTypes {
    Int(i32),
    Float(f32),
    FloatList(Vec<f32>),
}

impl serde::Serialize for RenderDataTypes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::FloatList(floats) => floats.serialize(serializer),
            Self::Int(int) => int.serialize(serializer),
            Self::Float(float) => float.serialize(serializer),
        }
    }
}

pub fn render_progress_clock(
    segments: u8,
    segments_filled: u8,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    dbg!("{segments} {segments_filled}");
    if segments_filled > segments {
        return Err(String::from("segments filled must be lesser than existing segments.").into());
    }

    let mut handlebars = Handlebars::new();

    handlebars
        .register_template_string("progress_clock", SVG_FILE)
        .map_err(|e| e.to_string())?;

    let angle_segment: f32 = 360f32 / f32::from(segments);
    let mut spoke_angles: Vec<f32> = vec![angle_segment];
    let mut shade_angles: Vec<f32> = vec![];
    for i in 0..segments {
        spoke_angles.push(angle_segment * f32::from(i));
        if i < segments_filled {
            // default angle of a shade object is 90deg. Need to use that as an offset.
            shade_angles.push(angle_segment * f32::from(i) - 90f32);
        }
    }

    let mut render_data = HashMap::new();
    let radius = 90f32;
    let width = 200;
    let height = 200;
    render_data.insert("spoke_angle", RenderDataTypes::FloatList(spoke_angles));
    render_data.insert("shade_angle", RenderDataTypes::FloatList(shade_angles));
    render_data.insert("height", RenderDataTypes::Int(height));
    render_data.insert("width", RenderDataTypes::Int(width));
    render_data.insert("cx", RenderDataTypes::Int(width / 2));
    render_data.insert("cy", RenderDataTypes::Int(height / 2));
    render_data.insert("radius", RenderDataTypes::Float(radius));
    render_data.insert(
        "rcostheta",
        RenderDataTypes::Float(radius * (angle_segment * PI / 180f32).cos()),
    );
    render_data.insert(
        "rsintheta",
        RenderDataTypes::Float(radius * (angle_segment * PI / 180f32).sin()),
    );

    let svg_source = handlebars
        .render("progress_clock", &render_data)
        .map_err(|e| e.to_string())?;

    dbg!("{}", &svg_source);

    let mut pixmap = Pixmap::new(200, 200).expect("Could not get mutable pixmap.");
    render(
        &Tree::from_data(&svg_source.into_bytes(), &Options::default())
            .expect("Could not build tree from SVG source"),
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );

    pixmap.encode_png().map_err(|err| err.into())
}
