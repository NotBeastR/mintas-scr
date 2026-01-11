use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct MypdfModule;
impl MypdfModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "create" => Self::create(args),
            "add_page" => Self::add_page(args),
            "add_text" => Self::add_text(args),
            "add_image" => Self::add_image(args),
            "add_line" => Self::add_line(args),
            "add_rect" => Self::add_rect(args),
            "set_font" => Self::set_font(args),
            "save" => Self::save(args),
            "to_string" => Self::to_string(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown mypdf function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn create(args: &[Value]) -> MintasResult<Value> {
        let title = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "Document".to_string() };
        let mut doc = HashMap::new();
        doc.insert("title".to_string(), Value::String(title));
        doc.insert("pages".to_string(), Value::Array(vec![]));
        doc.insert("font".to_string(), Value::String("Helvetica".to_string()));
        doc.insert("font_size".to_string(), Value::Number(12.0));
        doc.insert("__type__".to_string(), Value::String("PDF".to_string()));
        Ok(Value::Table(doc))
    }
    fn add_page(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Table(mut doc)) = args.get(0).cloned() {
            let width = match args.get(1) { Some(Value::Number(n)) => *n, _ => 612.0 };
            let height = match args.get(2) { Some(Value::Number(n)) => *n, _ => 792.0 };
            let mut page = HashMap::new();
            page.insert("width".to_string(), Value::Number(width));
            page.insert("height".to_string(), Value::Number(height));
            page.insert("content".to_string(), Value::Array(vec![]));
            if let Some(Value::Array(ref mut pages)) = doc.get_mut("pages") {
                pages.push(Value::Table(page));
            }
            return Ok(Value::Table(doc));
        }
        Ok(Value::Empty)
    }
    fn add_text(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Table(mut doc)) = args.get(0).cloned() {
            let text = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Table(doc)) };
            let x = match args.get(2) { Some(Value::Number(n)) => *n, _ => 50.0 };
            let y = match args.get(3) { Some(Value::Number(n)) => *n, _ => 700.0 };
            let mut item = HashMap::new();
            item.insert("type".to_string(), Value::String("text".to_string()));
            item.insert("text".to_string(), Value::String(text));
            item.insert("x".to_string(), Value::Number(x));
            item.insert("y".to_string(), Value::Number(y));
            if let Some(Value::Array(ref mut pages)) = doc.get_mut("pages") {
                if let Some(Value::Table(ref mut page)) = pages.last_mut() {
                    if let Some(Value::Array(ref mut content)) = page.get_mut("content") {
                        content.push(Value::Table(item));
                    }
                }
            }
            return Ok(Value::Table(doc));
        }
        Ok(Value::Empty)
    }
    fn add_image(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(true)) }
    fn add_line(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(true)) }
    fn add_rect(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(true)) }
    fn set_font(_args: &[Value]) -> MintasResult<Value> { Ok(Value::Boolean(true)) }
    fn save(args: &[Value]) -> MintasResult<Value> {
        let doc = match args.get(0) { Some(Value::Table(d)) => d.clone(), _ => return Ok(Value::Boolean(false)) };
        let path = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => "output.pdf".to_string() };
        let pdf_content = Self::generate_pdf(&doc);
        std::fs::write(&path, pdf_content).map_err(|e| MintasError::RuntimeError {
            message: format!("Failed to save PDF: {}", e),
            location: SourceLocation::new(0, 0),
        })?;
        Ok(Value::Boolean(true))
    }
    fn to_string(args: &[Value]) -> MintasResult<Value> {
        let doc = match args.get(0) { Some(Value::Table(d)) => d.clone(), _ => return Ok(Value::Empty) };
        Ok(Value::String(Self::generate_pdf(&doc)))
    }
    fn generate_pdf(doc: &HashMap<String, Value>) -> String {
        let mut pdf = String::new();
        pdf.push_str("%PDF-1.4\n");
        pdf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        pdf.push_str("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        pdf.push_str("3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n");
        let mut content = String::from("BT\n/F1 12 Tf\n");
        if let Some(Value::Array(pages)) = doc.get("pages") {
            for page in pages {
                if let Value::Table(p) = page {
                    if let Some(Value::Array(items)) = p.get("content") {
                        for item in items {
                            if let Value::Table(i) = item {
                                if let (Some(Value::String(text)), Some(Value::Number(x)), Some(Value::Number(y))) = 
                                    (i.get("text"), i.get("x"), i.get("y")) {
                                    content.push_str(&format!("{} {} Td\n({}) Tj\n", x, y, text));
                                }
                            }
                        }
                    }
                }
            }
        }
        content.push_str("ET\n");
        pdf.push_str(&format!("4 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n", content.len(), content));
        pdf.push_str("xref\n0 5\ntrailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n0\n%%EOF");
        pdf
    }
}