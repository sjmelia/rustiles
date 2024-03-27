use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Style {
    pub name: String,
    pub layers: Vec<Layer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LayerType {
    Fill,
    Line,
    Symbol,
    Circle,
    Heatmap,
    #[serde(rename = "fill-extrusion")]
    FillExtrusion,
    Raster,
    Hillshade,
    Model,
    Background,
    Sky,
    Slot,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum StringOrNumberOrSequenceOrBool {
    String(String),
    Number(i64),
    Bool(bool),
    Sequence(Vec<StringOrNumberOrSequenceOrBool>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum StringOrNumber {
    String(String),
    Number(i64),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FillColorWithStops {
    stops: Vec<Vec<StringOrNumber>>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FillColor {
    String(String),
    FillColorWithStops(FillColorWithStops)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Layer {
    pub id: String,
    pub source: Option<String>,
    #[serde(rename = "source-layer")]
    pub source_layer: Option<String>,
    #[serde(rename = "type")]
    pub layer_type: String,
    pub paint: Option<Paint>,
    pub filter: Option<Vec<StringOrNumberOrSequenceOrBool>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Paint {
    #[serde(rename = "background-color")]
    pub background_color: Option<String>,
    #[serde(rename = "fill-color")]
    pub fill_color: Option<FillColor>,
    #[serde(rename = "line-color")]
    pub line_color: Option<FillColor>,
}