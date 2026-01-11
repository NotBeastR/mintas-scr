use crate::errors::{MintasError, MintasResult, SourceLocation};
use crate::evaluator::Value;
use std::collections::HashMap;
pub struct MqttModule;
impl MqttModule {
    pub fn call_function(name: &str, args: &[Value]) -> MintasResult<Value> {
        match name {
            "connect" => Self::connect(args),
            "publish" => Self::publish(args),
            "subscribe" => Self::subscribe(args),
            "unsubscribe" => Self::unsubscribe(args),
            "disconnect" => Self::disconnect(args),
            "on_message" => Self::on_message(args),
            _ => Err(MintasError::RuntimeError {
                message: format!("Unknown mqtt function: {}", name),
                location: SourceLocation::new(0, 0),
            }),
        }
    }
    fn connect(args: &[Value]) -> MintasResult<Value> {
        let host = match args.get(0) { Some(Value::String(s)) => s.clone(), _ => "localhost".to_string() };
        let port = match args.get(1) { Some(Value::Number(n)) => *n as u16, _ => 1883 };
        let mut client = HashMap::new();
        client.insert("host".to_string(), Value::String(host));
        client.insert("port".to_string(), Value::Number(port as f64));
        client.insert("connected".to_string(), Value::Boolean(true));
        client.insert("subscriptions".to_string(), Value::Array(vec![]));
        client.insert("__type__".to_string(), Value::String("MQTTClient".to_string()));
        Ok(Value::Table(client))
    }
    fn publish(args: &[Value]) -> MintasResult<Value> {
        let _client = match args.get(0) { Some(Value::Table(c)) => c.clone(), _ => return Ok(Value::Boolean(false)) };
        let topic = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Boolean(false)) };
        let message = match args.get(2) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Boolean(false)) };
        let qos = match args.get(3) { Some(Value::Number(n)) => *n as u8, _ => 0 };
        println!("[MQTT] Publishing to {}: {} (QoS: {})", topic, message, qos);
        Ok(Value::Boolean(true))
    }
    fn subscribe(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Table(mut client)) = args.get(0).cloned() {
            let topic = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Table(client)) };
            if let Some(Value::Array(ref mut subs)) = client.get_mut("subscriptions") {
                subs.push(Value::String(topic.clone()));
            }
            println!("[MQTT] Subscribed to: {}", topic);
            return Ok(Value::Table(client));
        }
        Ok(Value::Boolean(false))
    }
    fn unsubscribe(args: &[Value]) -> MintasResult<Value> {
        let topic = match args.get(1) { Some(Value::String(s)) => s.clone(), _ => return Ok(Value::Boolean(false)) };
        println!("[MQTT] Unsubscribed from: {}", topic);
        Ok(Value::Boolean(true))
    }
    fn disconnect(args: &[Value]) -> MintasResult<Value> {
        if let Some(Value::Table(mut client)) = args.get(0).cloned() {
            client.insert("connected".to_string(), Value::Boolean(false));
            println!("[MQTT] Disconnected");
            return Ok(Value::Table(client));
        }
        Ok(Value::Boolean(true))
    }
    fn on_message(_args: &[Value]) -> MintasResult<Value> {
        Ok(Value::Boolean(true))
    }
}