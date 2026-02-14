use rustiles_style::Style;

#[test]
fn deserializes_modern_mapbox_style_v8() {
    let data = r##"
    {
      "version": 8,
      "name": "Modern",
      "sources": {
        "openmaptiles": {
          "type": "vector",
          "tiles": ["https://example.com/{z}/{x}/{y}.pbf"]
        }
      },
      "layers": [
        {
          "id": "water",
          "type": "fill",
          "source": "openmaptiles",
          "source-layer": "water",
          "paint": {
            "fill-color": "#9ecae1"
          }
        }
      ]
    }
    "##;

    let style = Style::from_str(data).expect("style should deserialize");
    assert_eq!(style.version, Some(8));
    assert_eq!(style.name.as_deref(), Some("Modern"));
    assert_eq!(style.layers.len(), 1);
    assert!(!style.is_legacy_style());
}

#[test]
fn deserializes_legacy_mapbox_style_v7() {
    let data = r##"
    {
      "version": 7,
      "name": "Legacy",
      "layers": {
        "road-major": {
          "type": "line",
          "source": "mapbox",
          "source-layer": "road",
          "filter": ["in", "class", "motorway", "trunk"],
          "paint": {
            "line-color": {
              "stops": [[5, "#dddddd"], [10, "#aaaaaa"]]
            }
          }
        }
      }
    }
    "##;

    let style = Style::from_str(data).expect("legacy style should deserialize");
    assert_eq!(style.version, Some(7));
    assert_eq!(style.name.as_deref(), Some("Legacy"));
    assert_eq!(style.layers.len(), 1);
    assert_eq!(style.layers[0].id, "road-major");
    assert!(style.is_legacy_style());
}
