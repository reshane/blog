use tokio::runtime::Runtime;
use std::thread;


pub async fn setup() {
    use blog::run;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let _ = thread::spawn(move || {
        let runtime = Runtime::new().expect("Unable to create tokio runtime");
        println!("--> starting server for tests");
        runtime.block_on(async {
            run(listener).await;
        });
    });
    println!("--> finished setup");
}
