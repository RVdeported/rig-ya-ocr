use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultOcr
{
  #[serde(rename = "textAnnotation")]
  pub text_ann: Annotation,
  pub page: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Annotation
{
  pub width: String,
  pub height: String,
  pub blocks: Vec<Block>,
  pub entities: Vec<Entity>,
  pub tables: Vec<Table>,
  #[serde(rename = "fullText")]
  pub full_text: String,
  pub rotate: String,
  pub markdown: String,
  pub pictures: Vec<Picture>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Block
{
  #[serde(rename = "boundingBox")]
  pub bounding_box: BoundingBox,
  pub lines: Vec<Line>,
  pub languages: Vec<Language>,
  #[serde(rename = "textSegments")]
  pub text_segments: Vec<TextSegment>,
  #[serde(rename = "layoutType")]
  pub layout_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BoundingBox
{
  pub vertices: Vec<Vertex>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Vertex
{
  pub x: String,
  pub y: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Line
{
  #[serde(rename = "boundingBox")]
  pub bounding_box: BoundingBox,
  pub text: String,
  pub words: Vec<Word>,
  #[serde(rename = "textSegments")]
  pub text_segments: Vec<TextSegment>,
  pub orientation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Word
{
  #[serde(rename = "boundingBox")]
  pub bounding_box: BoundingBox,
  pub text: String,
  #[serde(rename = "entityIndex")]
  pub entity_index: String,
  #[serde(rename = "textSegments")]
  pub text_segments: Vec<TextSegment>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TextSegment
{
  #[serde(rename = "startIndex")]
  pub start_index: String,
  pub length: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Language
{
  #[serde(rename = "languageCode")]
  pub language_code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Entity
{
  pub name: String,
  pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Table
{
  #[serde(rename = "boundingBox")]
  pub bounding_box: BoundingBox,
  #[serde(rename = "rowCount")]
  pub row_count: String,
  #[serde(rename = "columnCount")]
  pub column_count: String,
  pub cells: Vec<Cell>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Cell
{
  #[serde(rename = "boundingBox")]
  pub bounding_box: BoundingBox,
  #[serde(rename = "rowIndex")]
  pub row_index: String,
  #[serde(rename = "columnIndex")]
  pub column_index: String,
  #[serde(rename = "columnSpan")]
  pub column_span: String,
  #[serde(rename = "rowSpan")]
  pub row_span: String,
  pub text: String,
  #[serde(rename = "textSegments")]
  pub text_segments: Vec<TextSegment>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Picture
{
  #[serde(rename = "boundingBox")]
  pub bounding_box: BoundingBox,
  pub score: String,
}
