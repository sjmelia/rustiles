use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Style {
    #[serde(default)]
    pub version: Option<u8>,
    pub name: Option<String>,
    #[serde(default)]
    pub sources: BTreeMap<String, Value>,
    pub layers: Vec<Layer>,
}

impl Style {
    pub fn from_str(input: &str) -> serde_json::Result<Self> {
        let raw: RawStyle = serde_json::from_str(input)?;
        Ok(raw.into_style())
    }

    /// Returns true for classic pre-v8 style documents.
    pub fn is_legacy_style(&self) -> bool {
        self.version.unwrap_or_default() < 8
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum LayerType {
    Fill,
    Line,
    Symbol,
    Circle,
    Heatmap,
    FillExtrusion,
    Raster,
    RasterParticle,
    Hillshade,
    Model,
    Background,
    Sky,
    Slot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Layer {
    pub id: String,
    #[serde(rename = "type")]
    pub layer_type: Option<LayerType>,
    pub source: Option<String>,
    #[serde(rename = "source-layer")]
    pub source_layer: Option<String>,
    pub paint: Option<Value>,
    pub layout: Option<Value>,
    pub filter: Option<Value>,
    pub minzoom: Option<f64>,
    pub maxzoom: Option<f64>,
}

#[derive(Deserialize)]
struct RawStyle {
    #[serde(default)]
    version: Option<u8>,
    name: Option<String>,
    #[serde(default)]
    sources: BTreeMap<String, Value>,
    layers: RawLayers,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawLayers {
    Modern(Vec<Layer>),
    Legacy(BTreeMap<String, LegacyLayer>),
}

#[derive(Serialize, Deserialize)]
struct LegacyLayer {
    #[serde(rename = "type")]
    layer_type: Option<LayerType>,
    source: Option<String>,
    #[serde(rename = "source-layer")]
    source_layer: Option<String>,
    paint: Option<Value>,
    layout: Option<Value>,
    filter: Option<Value>,
    minzoom: Option<f64>,
    maxzoom: Option<f64>,
}

impl RawStyle {
    fn into_style(self) -> Style {
        let layers = match self.layers {
            RawLayers::Modern(layers) => layers,
            RawLayers::Legacy(legacy_layers) => legacy_layers
                .into_iter()
                .map(|(id, layer)| Layer {
                    id,
                    layer_type: layer.layer_type,
                    source: layer.source,
                    source_layer: layer.source_layer,
                    paint: layer.paint,
                    layout: layer.layout,
                    filter: layer.filter,
                    minzoom: layer.minzoom,
                    maxzoom: layer.maxzoom,
                })
                .collect(),
        };

        Style {
            version: self.version,
            name: self.name,
            sources: self.sources,
            layers,
        }
    }
}
