use super::Stack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    x: f32,
    y: f32,
    stack: Stack,
}
