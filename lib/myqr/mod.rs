use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct MyqrModule;
impl MyqrModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "generate" | "create" => Self::generate(args),
            "to_ascii" => Self::to_ascii(args),
            "to_svg" => Self::to_svg(args),
            "to_html" => Self::to_html(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown myqr function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn generate(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let size = match args.get(1) { Some(Value::Number(n)) => *n as usize, _ => 21 };
        let mut matrix = vec![vec![false; size]; size];
        Self::add_finder_pattern(&mut matrix, 0, 0);
        Self::add_finder_pattern(&mut matrix, size - 7, 0);
        Self::add_finder_pattern(&mut matrix, 0, size - 7);
        let data_bytes = data.as_bytes();
        for (i, &byte) in data_bytes.iter().enumerate() {
            let x = 8 + (i % (size - 16));
            let y = 8 + (i / (size - 16));
            if x < size && y < size {
                for bit in 0..8 {
                    let bx = x + (bit % 4);
                    let by = y + (bit / 4);
                    if bx < size - 8 && by < size - 8 {
                        matrix[by][bx] = (byte >> (7 - bit)) & 1 == 1;
                    }
                }
            }
        }
        let result: Vec<Value> = matrix.iter().map(|row| {
            Value::Array(row.iter().map(|&b| Value::Boolean(b)).collect())
        }).collect();
        Ok(Value::Array(result))
    }
    fn add_finder_pattern(matrix: &mut Vec<Vec<bool>>, x: usize, y: usize) {
        for dy in 0..7 {
            for dx in 0..7 {
                let is_border = dx == 0 || dx == 6 || dy == 0 || dy == 6;
                let is_center = dx >= 2 && dx <= 4 && dy >= 2 && dy <= 4;
                if x + dx < matrix[0].len() && y + dy < matrix.len() {
                    matrix[y + dy][x + dx] = is_border || is_center;
                }
            }
        }
    }
    fn to_ascii(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let qr = Self::generate(&[Value::String(data)])?;
        if let Value::Array(rows) = qr {
            let mut result = String::new();
            for row in rows {
                if let Value::Array(cols) = row {
                    for col in cols {
                        if let Value::Boolean(b) = col {
                            result.push_str(if b { "██" } else { "  " });
                        }
                    }
                    result.push('\n');
                }
            }
            return Ok(Value::String(result));
        }
        Ok(Value::Empty)
    }
    fn to_svg(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let size = match args.get(1) { Some(Value::Number(n)) => *n as usize, _ => 200 };
        let qr = Self::generate(&[Value::String(data)])?;
        if let Value::Array(rows) = qr {
            let cell_size = size / rows.len().max(1);
            let mut svg = format!(r#"<svg xmlns="http:
            svg.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);
            for (y, row) in rows.iter().enumerate() {
                if let Value::Array(cols) = row {
                    for (x, col) in cols.iter().enumerate() {
                        if let Value::Boolean(true) = col {
                            svg.push_str(&format!(r#"<rect x="{}" y="{}" width="{}" height="{}" fill="black"/>"#,
                                x * cell_size, y * cell_size, cell_size, cell_size));
                        }
                    }
                }
            }
            svg.push_str("</svg>");
            return Ok(Value::String(svg));
        }
        Ok(Value::Empty)
    }
    fn to_html(args: &[Value]) -> MintasResult<Value> {
        let data = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Empty) };
        let qr = Self::generate(&[Value::String(data)])?;
        if let Value::Array(rows) = qr {
            let mut html = String::from(r#"<div style="display:inline-block;background:white;padding:10px;">"#);
            for row in rows {
                if let Value::Array(cols) = row {
                    html.push_str("<div style=\"height:5px;\">");
                    for col in cols {
                        if let Value::Boolean(b) = col {
                            let color = if b { "black" } else { "white" };
                            html.push_str(&format!(r#"<span style="display:inline-block;width:5px;height:5px;background:{}"></span>"#, color));
                        }
                    }
                    html.push_str("</div>");
                }
            }
            html.push_str("</div>");
            return Ok(Value::String(html));
        }
        Ok(Value::Empty)
    }
}