use anyhow::{anyhow, Result};
use geo_types::{Geometry, LineString, Point, Polygon};
use mvt_reader::feature::Feature;
use mvt_reader::Reader;
use rustiles_style::{LayerType, Style};
use serde_json::Value;
use tiny_skia::*;

const TILE_EXTENT: u32 = 4096;

#[derive(Debug, Clone)]
struct RenderRule {
    source_layer: Option<String>,
    layer_type: LayerType,
    filter: Option<Value>,
    paint: Value,
}

pub fn render_tile_with_style(reader: Reader, style: &Style) -> Result<Pixmap> {
    let Some(mut pixmap) = Pixmap::new(TILE_EXTENT, TILE_EXTENT) else {
        return Err(anyhow!("Could not create pixmap"));
    };

    apply_background(&mut pixmap, style)?;

    let layer_names = reader
        .get_layer_names()
        .map_err(|_| anyhow!("Could not get layer names"))?;

    let mut tileset_layers = Vec::new();
    for (index, name) in layer_names.iter().enumerate() {
        let features = reader
            .get_features(index)
            .map_err(|_| anyhow!("Could not get features for layer {name}"))?;
        tileset_layers.push((name.clone(), features));
    }

    let rules = compile_rules(style);
    for rule in rules {
        for (source_layer, features) in &tileset_layers {
            if let Some(expected_layer) = &rule.source_layer {
                if expected_layer != source_layer {
                    continue;
                }
            }

            for feature in features {
                if matches_filter(feature, rule.filter.as_ref()) {
                    render_feature(&mut pixmap, feature, &rule)?;
                }
            }
        }
    }

    Ok(pixmap)
}

pub fn render_tile(reader: Reader) -> Result<Pixmap> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 127, 0, 200);

    let mut stroke = Stroke::default();
    stroke.width = 4.0;

    let Some(mut pixmap) = Pixmap::new(TILE_EXTENT, TILE_EXTENT) else {
        return Err(anyhow!("Could not create pixmap"));
    };

    let layer_names = reader
        .get_layer_names()
        .map_err(|_| anyhow!("Could not get layer names"))?;

    for (index, _name) in layer_names.iter().enumerate() {
        let features = reader
            .get_features(index)
            .map_err(|_| anyhow!("Could not get features"))?;

        for feature in features {
            render_geometry_stroked(&mut pixmap, &paint, &stroke, feature.get_geometry())?;
        }
    }

    Ok(pixmap)
}

fn compile_rules(style: &Style) -> Vec<RenderRule> {
    style
        .layers
        .iter()
        .filter_map(|layer| {
            let layer_type = layer.layer_type.clone()?;
            Some(RenderRule {
                source_layer: layer.source_layer.clone(),
                layer_type,
                filter: layer.filter.clone(),
                paint: layer.paint.clone().unwrap_or(Value::Null),
            })
        })
        .collect()
}

fn apply_background(pixmap: &mut Pixmap, style: &Style) -> Result<()> {
    for layer in &style.layers {
        if !matches!(layer.layer_type, Some(LayerType::Background)) {
            continue;
        }

        if let Some(paint) = &layer.paint {
            if let Some(color) = paint.get("background-color").and_then(Value::as_str) {
                let parsed = parse_color(color).unwrap_or(Color::WHITE);
                pixmap.fill(parsed);
                return Ok(());
            }
        }
    }

    pixmap.fill(Color::from_rgba8(240, 240, 240, 255));
    Ok(())
}

fn render_feature(pixmap: &mut Pixmap, feature: &Feature, rule: &RenderRule) -> Result<()> {
    match rule.layer_type {
        LayerType::Line => {
            let mut paint = Paint::default();
            if let Some(color) = rule.paint.get("line-color").and_then(read_style_string) {
                if let Some(parsed) = parse_color(color) {
                    paint.set_color(parsed);
                }
            } else {
                paint.set_color(Color::from_rgba8(80, 80, 80, 255));
            }

            let mut stroke = Stroke::default();
            stroke.width = rule
                .paint
                .get("line-width")
                .and_then(read_style_number)
                .unwrap_or(1.0);

            render_geometry_stroked(pixmap, &paint, &stroke, feature.get_geometry())
        }
        LayerType::Fill => {
            let mut paint = Paint::default();
            if let Some(color) = rule.paint.get("fill-color").and_then(read_style_string) {
                if let Some(parsed) = parse_color(color) {
                    paint.set_color(parsed);
                }
            } else {
                paint.set_color(Color::from_rgba8(170, 170, 170, 255));
            }

            render_geometry_filled(pixmap, &paint, feature.get_geometry())
        }
        LayerType::Circle => {
            let mut paint = Paint::default();
            if let Some(color) = rule.paint.get("circle-color").and_then(read_style_string) {
                if let Some(parsed) = parse_color(color) {
                    paint.set_color(parsed);
                }
            } else {
                paint.set_color(Color::from_rgba8(40, 110, 210, 255));
            }

            let radius = rule
                .paint
                .get("circle-radius")
                .and_then(read_style_number)
                .unwrap_or(3.0);

            render_points_with_radius(pixmap, &paint, feature.get_geometry(), radius)
        }
        _ => Ok(()),
    }
}

fn render_points_with_radius(
    pixmap: &mut Pixmap,
    paint: &Paint,
    geometry: &Geometry<f32>,
    radius: f32,
) -> Result<()> {
    match geometry {
        Geometry::Point(point) => render_point_circle(pixmap, paint, point, radius),
        Geometry::MultiPoint(multi_point) => {
            for point in multi_point {
                render_point_circle(pixmap, paint, point, radius)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn render_point_circle(
    pixmap: &mut Pixmap,
    paint: &Paint,
    point: &Point<f32>,
    radius: f32,
) -> Result<()> {
    let mut pb = PathBuilder::new();
    pb.push_circle(point.x(), point.y(), radius);
    let Some(path) = pb.finish() else {
        return Err(anyhow!("Could not create point path"));
    };

    pixmap.fill_path(&path, paint, FillRule::Winding, Transform::identity(), None);
    Ok(())
}

fn render_geometry_stroked(
    pixmap: &mut Pixmap,
    paint: &Paint,
    stroke: &Stroke,
    geometry: &Geometry<f32>,
) -> Result<()> {
    if let Some(path) = build_path_from_geometry(geometry, false) {
        pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
    }

    Ok(())
}

fn render_geometry_filled(
    pixmap: &mut Pixmap,
    paint: &Paint,
    geometry: &Geometry<f32>,
) -> Result<()> {
    if let Some(path) = build_path_from_geometry(geometry, true) {
        pixmap.fill_path(&path, paint, FillRule::Winding, Transform::identity(), None);
    }

    Ok(())
}

fn build_path_from_geometry(geometry: &Geometry<f32>, closed: bool) -> Option<Path> {
    let mut pb = PathBuilder::new();

    match geometry {
        Geometry::Point(point) => {
            pb.move_to(point.x(), point.y());
            pb.line_to(point.x() + 1.0, point.y());
        }
        Geometry::Line(line) => {
            pb.move_to(line.start.x, line.start.y);
            pb.line_to(line.end.x, line.end.y);
        }
        Geometry::LineString(line_string) => push_line_string(&mut pb, line_string, closed),
        Geometry::MultiLineString(lines) => {
            for line in lines {
                push_line_string(&mut pb, line, closed);
            }
        }
        Geometry::Polygon(polygon) => push_polygon(&mut pb, polygon),
        Geometry::MultiPolygon(polygons) => {
            for polygon in polygons {
                push_polygon(&mut pb, polygon);
            }
        }
        _ => return None,
    }

    pb.finish()
}

fn push_line_string(pb: &mut PathBuilder, line_string: &LineString<f32>, close: bool) {
    let mut points = line_string.points();
    let Some(first) = points.next() else {
        return;
    };

    pb.move_to(first.x(), first.y());
    for point in points {
        pb.line_to(point.x(), point.y());
    }

    if close {
        pb.close();
    }
}

fn push_polygon(pb: &mut PathBuilder, polygon: &Polygon<f32>) {
    push_line_string(pb, polygon.exterior(), true);
    for hole in polygon.interiors() {
        push_line_string(pb, hole, true);
    }
}

fn parse_color(input: &str) -> Option<Color> {
    let value = input.trim();
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color::from_rgba8(r, g, b, a))
        }
        _ => None,
    }
}

fn read_style_string(value: &Value) -> Option<&str> {
    if let Some(s) = value.as_str() {
        return Some(s);
    }

    value
        .get("stops")
        .and_then(Value::as_array)
        .and_then(|stops| stops.last())
        .and_then(Value::as_array)
        .and_then(|stop| stop.get(1))
        .and_then(Value::as_str)
}

fn read_style_number(value: &Value) -> Option<f32> {
    if let Some(n) = value.as_f64() {
        return Some(n as f32);
    }

    value
        .get("stops")
        .and_then(Value::as_array)
        .and_then(|stops| stops.last())
        .and_then(Value::as_array)
        .and_then(|stop| stop.get(1))
        .and_then(Value::as_f64)
        .map(|v| v as f32)
}

fn matches_filter(feature: &Feature, filter: Option<&Value>) -> bool {
    let Some(filter) = filter else {
        return true;
    };

    evaluate_filter(feature, filter).unwrap_or(true)
}

fn evaluate_filter(feature: &Feature, filter: &Value) -> Option<bool> {
    let items = filter.as_array()?;
    let op = items.first()?.as_str()?;

    match op {
        "==" => {
            let key = parse_get_key(items.get(1)?)?;
            let value = as_filter_value(items.get(2)?)?;
            Some(get_property(feature, key).is_some_and(|actual| actual == value))
        }
        "!=" => {
            let key = parse_get_key(items.get(1)?)?;
            let value = as_filter_value(items.get(2)?)?;
            Some(get_property(feature, key).is_some_and(|actual| actual != value))
        }
        "in" => {
            let key = parse_get_key(items.get(1)?)?;
            let Some(actual) = get_property(feature, key) else {
                return Some(false);
            };
            let any_match = items
                .iter()
                .skip(2)
                .filter_map(as_filter_value)
                .any(|candidate| candidate == actual);
            Some(any_match)
        }
        "!in" => {
            let key = parse_get_key(items.get(1)?)?;
            let Some(actual) = get_property(feature, key) else {
                return Some(true);
            };
            let any_match = items
                .iter()
                .skip(2)
                .filter_map(as_filter_value)
                .any(|candidate| candidate == actual);
            Some(!any_match)
        }
        "all" => Some(
            items
                .iter()
                .skip(1)
                .all(|child| evaluate_filter(feature, child).unwrap_or(false)),
        ),
        "any" => Some(
            items
                .iter()
                .skip(1)
                .any(|child| evaluate_filter(feature, child).unwrap_or(false)),
        ),
        "none" => Some(
            !items
                .iter()
                .skip(1)
                .any(|child| evaluate_filter(feature, child).unwrap_or(false)),
        ),
        _ => None,
    }
}

fn parse_get_key(value: &Value) -> Option<&str> {
    if let Some(key) = value.as_str() {
        return Some(key);
    }

    let expression = value.as_array()?;
    if expression.first()?.as_str()? != "get" {
        return None;
    }

    expression.get(1)?.as_str()
}

fn as_filter_value(value: &Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        return Some(s.to_string());
    }

    if let Some(n) = value.as_i64() {
        return Some(n.to_string());
    }

    if let Some(n) = value.as_f64() {
        return Some(n.to_string());
    }

    value.as_bool().map(|b| b.to_string())
}

fn get_property<'a>(feature: &'a Feature, key: &str) -> Option<&'a str> {
    feature.properties.as_ref()?.get(key).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::{evaluate_filter, parse_color};
    use geo_types::Point;
    use mvt_reader::feature::Feature;
    use serde_json::json;
    use std::collections::HashMap;

    fn feature_with_class(class: &str) -> Feature {
        let mut properties = HashMap::<String, String>::new();
        properties.insert("class".to_string(), class.to_string());

        Feature {
            geometry: geo_types::Geometry::Point(Point::<f32>::new(0.0, 0.0)),
            properties: Some(properties),
        }
    }

    #[test]
    fn parses_hex_color() {
        assert!(parse_color("#00ff99").is_some());
        assert!(parse_color("#00ff99cc").is_some());
    }

    #[test]
    fn evaluates_legacy_in_filter() {
        let filter = json!(["in", "class", "motorway", "trunk"]);
        let matches = evaluate_filter(&feature_with_class("trunk"), &filter).unwrap();
        assert!(matches);
    }

    #[test]
    fn evaluates_expression_get_filter() {
        let filter = json!(["==", ["get", "class"], "primary"]);
        let matches = evaluate_filter(&feature_with_class("primary"), &filter).unwrap();
        assert!(matches);
    }
}
