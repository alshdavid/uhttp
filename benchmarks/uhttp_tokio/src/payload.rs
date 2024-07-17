use once_cell::sync::Lazy;

pub const PAYLOAD: &str = "Hello World!";

pub const HEADERS: Lazy<Vec<(&str, &str)>> = Lazy::new(|| {
  vec![
    ("Accept", "*/*"),
    ("Date", "Mon, 15 Jul 2024 11:18:56 GMT"),
    // ("Connection", "close"),
    ("Content-Type", "text/plain"),
  ]
});

pub const DATA: Lazy<Vec<u8>> = Lazy::new(|| {
  let mut headers = String::new();
  
  headers.push_str("HTTP/1.1 200 OK\r\n");
  
  headers.push_str(&HEADERS.iter()
    .map(|v| format!("{}: {}", v.0, v.1))
    .collect::<Vec<String>>()
    .join("\r\n"));

  headers.push_str("\r\nContent-Length: ");
  headers.push_str(&PAYLOAD.as_bytes().len().to_string());
  headers.push_str("\r\n");

  headers.push_str("\r\n");
  headers.push_str(&PAYLOAD);
  
  headers.as_bytes().to_vec()
});
