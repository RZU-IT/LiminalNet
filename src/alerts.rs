use std::{
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
    time::Duration,
};

pub struct Delivery {
    pub requests: SyncSender<(String, String)>,
    pub errors: Receiver<String>,
}
impl Delivery {
    pub fn new() -> Self {
        let (requests, rx) = mpsc::sync_channel::<(String, String)>(16);
        let (tx, errors) = mpsc::sync_channel(16);
        thread::spawn(move || {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(5)))
                .build()
                .into();
            while let Ok((url, message)) = rx.recv() {
                if let Err(error) = agent
                    .post(&url)
                    .send_json(serde_json::json!({"application":"LiminalNet","message":message}))
                {
                    let _ = tx.try_send(format!("Webhook delivery failed: {error}"));
                }
            }
        });
        Self { requests, errors }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delivers_a_webhook_to_a_local_test_server() {
        use std::io::{Read, Write};
        let server = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = server.local_addr().unwrap();
        let receiver = thread::spawn(move || {
            let (mut socket, _) = server.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut buffer = vec![0u8; 4096];
            let mut request = String::new();
            loop {
                let count = socket.read(&mut buffer).unwrap();
                assert!(count > 0);
                request.push_str(&String::from_utf8_lossy(&buffer[..count]));
                if request.contains("test-event") {
                    break;
                }
            }
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .unwrap();
            request
        });
        let delivery = Delivery::new();
        delivery
            .requests
            .send((format!("http://{address}/events"), "test-event".into()))
            .unwrap();
        let request = receiver.join().unwrap();
        assert!(request.contains("LiminalNet"));
        assert!(request.contains("test-event"));
    }
}
