pub type KeyValue = (String, JsonElement);

pub enum JsonElement {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonElement>),
    Object(Vec<KeyValue>),
}