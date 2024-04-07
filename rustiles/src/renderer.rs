use anyhow::{Result, anyhow};
use geo_types::{Geometry, Point, Line, LineString};
use log::{trace, warn};
use mvt_reader::Reader;
use mvt_reader::feature::Feature;
use tiny_skia::*;

pub fn render_tile(reader: Reader) -> Result<Pixmap> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 127, 0, 200);
    paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = 6.0;
    stroke.line_cap = LineCap::Round;
    stroke.dash = StrokeDash::new(vec![20.0, 40.0], 0.0);

    let pixmap_option = Pixmap::new(4096, 4096);
    let Some(mut pixmap) = pixmap_option else {
        return Err(anyhow!("Could not create pixmap"));
    };

    let Ok(layer_names) = reader.get_layer_names() else {
        return Err(anyhow!("Could not get layer names"));
    };

    render_layers(&mut pixmap, paint, stroke, reader, layer_names)?;
    
    Ok(pixmap)
}

fn expression_get(attribute_name: String) -> Box<dyn Fn(Feature) -> Option<String>> {
    Box::new(move |feature: Feature| {
        let properties = feature.properties?;
        let attribute_value = properties.get(&attribute_name)?;
        Some(attribute_value.to_string())
    })
}

#[cfg(test)]
mod tests {
    use crate::renderer::expression_get;
    use crate::renderer::Feature;
    use geo_types::Point;
    use std::collections::HashMap;

    #[test]
    fn expression_get_retrieves_class() {
        let mut properties = HashMap::<String, String>::new();
        properties.insert("class".to_string(), "minor".to_string());

        let feature = Feature {
            geometry: geo_types::Geometry::Point(Point::<f32>::new(0.0,0.0)),
            properties: Some(properties),
        };

        let sut = expression_get("class".to_string());
        let result = sut(feature);
        assert_eq!(result, Some("minor".to_string()));
    }
}

/*
struct Filter {}

struct CompiledStyle
{
    filter: Filter,
}

pub fn render_style(style: CompiledStyle, features: Vec<Feature>) {
    let predicate = |feature: &Feature| {
        let properties_option = &feature.properties;
        if let Some(properties) = properties_option {
            let class_option = properties.get("class");
            if let Some(class) = class_option {
                return class == "tertiary"
            }
        }

        false
    };

    let features = features.iter().filter(predicate);
}
 */

fn render_layers(pixmap: &mut Pixmap, paint: Paint, stroke: Stroke, reader: Reader, layer_names: Vec<String>) -> Result<()> {
    for (index, _name) in layer_names.iter().enumerate() {
        let Ok(features) = reader.get_features(index) else {
            return Err(anyhow!("Could not get features"));
        };

        render_features(pixmap, &paint, &stroke, features)?;
    }

    Ok(())
}

fn render_features(pixmap: &mut Pixmap, paint: &Paint, stroke: &Stroke, features: Vec<Feature>) -> Result<()> {
    let mut red = Paint::default();
    red.set_color_rgba8(127, 0, 0, 200);
    red.anti_alias = true;

    fn matches_pattern(feature: &Feature) -> bool {
        let properties_option = &feature.properties;
        if let Some(properties) = properties_option {
            let class_option = properties.get("class");
            if let Some(class) = class_option {
                //println!("Class was {}", class);
                if class == "tertiary" || class == "track" || class == "minor" || class == "secondary" || class == "primary" || class == "trunk" || class == "motorway" {
                    return true;
                }
            }
        }

        false
    }

    for feature in features {
        let geometry = feature.get_geometry();
        if matches_pattern(&feature) {
            render_geometry(pixmap, &red, stroke, geometry)?;
        } else {
            render_geometry(pixmap, paint, stroke, geometry)?;
        }
    }

    Ok(())
}

fn render_point(pixmap: &mut Pixmap, paint: &Paint, point: &Point<f32>) -> Result<()> {
    let x = point.x() as f32;
    let y = point.y() as f32;
    let Some(rect) = tiny_skia::Rect::from_xywh(x, y, 3.0, 3.0) else {
        return Err(anyhow!("Could not create rect from point"));
    };

    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    Ok(())
}

fn render_line(pixmap: &mut Pixmap, paint: &Paint, stroke: &Stroke, line: &Line<f32>) -> Result<()> {
    let start_point = line.start_point();
    let start_x = start_point.x() as f32;
    let start_y = start_point.y() as f32;
    let end_point = line.end_point();
    let end_x = end_point.x() as f32;
    let end_y = end_point.y() as f32;
    
    let path_option = {
        let mut pb = PathBuilder::new();
        pb.move_to(start_x, start_y);
        pb.line_to(end_x, end_y);
        pb.finish()
    };

    if let Some(path) = path_option {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        return Ok(())
    } else {
        return Err(anyhow!("Could not draw line"));
    };
}

fn render_line_string(pixmap: &mut Pixmap, paint: &Paint, stroke: &Stroke, line_string: &LineString<f32>) -> Result<()> {
    let path_option = {
        let mut pb = PathBuilder::new();
        let points = line_string.points();
        for (index, point) in points.enumerate() {
            trace!("\tPoint at {}, {}", point.x(), point.y());
            let x = point.x();
            let y = point.y();
            if index == 0 {
                pb.move_to(x, y);
            } else {
                pb.line_to(x, y);
            }
        }

        pb.finish()
    };
    
    if let Some(path) = path_option {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        return Ok(())
    } else {
        return Err(anyhow!("Could not draw line string"));
    };
}

fn render_geometry(pixmap: &mut Pixmap, paint: &Paint, stroke: &Stroke, geometry: &Geometry<f32>) -> Result<()> {
    match geometry {
        Geometry::Point(point) => render_point(pixmap, paint, point)?,
        Geometry::MultiPoint(multi_point) => {
            for point in multi_point { 
                render_point(pixmap, paint, point)?
            }
        }
        Geometry::Line(line) => render_line(pixmap, paint, stroke, line)?,
        Geometry::LineString(line_string) => render_line_string(pixmap, paint, stroke, line_string)?,
        Geometry::MultiLineString(multi_line_string) => {
            for line_string in multi_line_string {
                render_line_string(pixmap, paint, stroke, line_string)?;
            }
        }
        Geometry::Polygon(polygon) => {
            // warn only exterior of polygons painted!
            let exterior = polygon.exterior();
            render_line_string(pixmap, paint, stroke, exterior)?;
        }
        Geometry::MultiPolygon(multi_polygon) => {
            for polygon in multi_polygon {
                let exterior = polygon.exterior();
                render_line_string(pixmap, paint, stroke, exterior)?;
            }
        }
        Geometry::Rect(_rect) => {
            warn!("Rect - not implemented");
        }
        Geometry::Triangle(_triangle) => {
            warn!("Triangle - not implemented");
        }
        Geometry::GeometryCollection(_geometry_collection) => {
            warn!("GeometryCollection - not implemented");
        }
    }

    Ok(())
}