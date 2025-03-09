# Image resizing blog

Start by creating cargo project

![1_aw_nutz.png](/assets/20241106_image_resizing/images/1_aw_nutz.png)


copy shell.nix from another project & start the shell

where is my fish?

![2_where_my_fish.png](/assets/20241106_image_resizing/images/2_where_my_fish.png)


exit that shell

[![3_exit_dirty_bash.webm](/assets/video_placeholder.png)](/assets/20241106_image_resizing/images/3_exit_dirty_bash.webm)


look up youtube video
 - https://www.youtube.com/watch?v=tv9s4jhdUpU

come back in a few days and start writing the resizer again
So far, this is what we have in main.rs:
```
use axum::{
    routing::get,
    body::Bytes,
    http::StatusCode,
    extract::Query,
    Router,
    debug_handler,
};
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
struct Task {
    width: Option<u32>,
    height: Option<u32>,
    url: String,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    // tracing_subscriber::fmt()
        // .compact().with_file(true)
        // .with_line_number(true)
        // .with_thread_ids(true)
        // .with_target(true).finish();
    let sub = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(sub).unwrap();


    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/resize", get(resize));
    info!("starting");

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
#[debug_handler]
async fn resize(task: Query<Task>) -> (StatusCode, Bytes) {
    // we need to fetch the image here
    info!(
        "Received request: url: {:?} w: {:?} h: {:?}",
        task.url, task.width, task.height
    );
    (StatusCode::OK, task.url.as_bytes().to_vec().into())
}
```
when we run it, we get this in the logs: 

![4_initial_logs.png](/assets/20241106_image_resizing/images/4_initial_logs.png)

and we get this back when we send a request:

![5_add_image_crate.png](/assets/20241106_image_resizing/images/5_add_image_crate.png)


Now, we'll write a module that fetches images for us and returns them in a DynamicImage
First, we'll restructure what we have & create files for our image fetching module
It'll live with all of our http-client-like things, so we'll name the module http
We'll also introduce an error module to make using the `?` easier.
And just like that our structure looks like this:

![6_new_file_structure.png](/assets/20241106_image_resizing/images/6_new_file_structure.png)

And our `src/error.rs` looks like this:
```
// errors for the service

use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

pub enum Error {
}

```
Note that we also added the `derive_more` crate. (`cargo add derive_more --features full`)
This will come into play later when converting from external errors to our service errors.

Now, we'll start converting all of our `unwrap()`s into `?` operators.
Starting in lib.rs, we make `run` return a `Result<()>` and...
```
error[E0277]: `?` couldn't convert the error to `error::Error`
  --> src/lib.rs:50:37
   |
50 |     axum::serve(listener, app).await?;
   |                                     ^ the trait `From<std::io::Error>` is not implemented for `error::Error`, which is required by `std::result::Result<(), error::Error>: FromResidual<std::result::Result<Infallible, std::io::Error>>`
   |
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
   = help: the trait `FromResidual<std::result::Result<Infallible, E>>` is implemented for `std::result::Result<T, F>`
   = note: required for `std::result::Result<(), error::Error>` to implement `FromResidual<std::result::Result<Infallible, std::io::Error>>`
```
now it yells at us to make our error implement `From` for `std::io::Error`.
This should be easy with `derive_more`.
```
#[derive(From, Debug)]
pub enum Error {
    // External
    #[from]
    TracingGlobalDefault(tracing::subscriber::SetGlobalDefaultError),
    #[from]
    ServerBinding(std::io::Error),
}
```
And no more errors! We'll need to add more to this enum while implementing our image fetcher...
So we'll do that now
We want to be caching as much as possible in order to speed this thing up because image processing pretty slow.
In order to fetch images faster, we should actually send two requests - 
    1. Grab the data from the provided image url
    2. Grab the data from a local image store if possible
Typically this would be done in order, but we can use tokio to just fire off two requests and take whichever comes back successfully first.
I'm not certain this is actually a performance improvement for these reasons:
    - Requesting a local image that doesn't exist will 404 very quickly anyway
    - Always sending off two requests rather than sometimes only sending off one could be detrimental under heavy load
I'm going to do it anyway for these reasons:
    - Learning experience
    - Sometimes you have to get fancy and confusing to discover simplicity
    - Gives me an opportunity to find out which is faster later
So, I'll be creating two different implementations of our image fetching function
`serial_fetch`
`parallel_fetch`
Lets get into it...

In order to write a simple http client we'll use hyper, since axum already uses it
And I'll be starting from this example: `https://hyper.rs/guides/1/client/basic/`
Making a single request to begin with, we have something like this:
```
#[tracing::instrument]
pub async fn serial_fetch(url: String) -> Result<DynamicImage> {
    // parse our url
    let url = url.parse::<hyper::Uri>()?;

    // get the host and the port
    let host = url.host().unwrap();
    let port = url.port_u16().unwrap_or(80);

    let address = format!("{}:{}", host, port);

    // Open TCP connection to the remote host
    let stream = TcpStream::connect(address).await?;

    let io = TokioIo::new(stream);

    // create the Hyper Client
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    // spawn a task to poll the connection, driving the http state
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            error!("Connection failed: {:?}", err);
        };
    });

    // the authority of our url will be the host
    let authority = url.authority().unwrap().clone();

    // create an http request with an empty body & host header
    let req = Request::builder()
        .uri(url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    // await the response
    let mut res = sender.send_request(req).await?;

    let mut image_bytes = Vec::<u8>::new();

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            image_bytes.extend(chunk);
        }

    }

    let mut img = ImageReader::new(std::io::Cursor::new(image_bytes));
    img.set_format(ImageFormat::Png);
    let img = img.decode()?;

    Ok(img)

}
```
Some TODO's for this thing:
    - abstract away the 'client' construct
    - parsing the bytes into an image should be separate
        - this is going to get more complicated with more formats

For now, we can start using this thing by updating our handler in lib.rs like this:
```
async fn resize(task: Query<Task>) -> impl IntoResponse {
    // we need to fetch the image here
    info!(
        "Received request: url: {:?} w: {:?} h: {:?}",
        task.url, task.width, task.height
    );
    match image_fetcher::serial_fetch(task.url.clone()).await {
        Ok(img) => {
            let mut bytes = Vec::<u8>::new();
            img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();
            return (
                StatusCode::OK,
                AppendHeaders([
                    ("Content-Type", "image/png")
                ]),
                bytes.into()
            );
        },
        Err(err) => {
            error!("Could not get the image {:?}", err);
            return (
                StatusCode::NOT_FOUND,
                AppendHeaders([
                    ("Content-Type", "image/png"),
                ]),
                Bytes::new()
            );
        },
    }
}
```
For convenience, we write a little test script `test_curl`:
```
#! /bin/sh

curl \
    "localhost:3000/resize?url=http%3A%2F%2Fstatic.wikia.nocookie.net%2Fadventuretimewithfinnandjake%2Fimages%2F9%2F9e%2FPeppermint_Butler.png" \
    --verbose \
    --output out.png
```
and then we can `cargo r` and `./test_curl` to see if it works

![7_test_curl_logs.png](/assets/20241106_image_resizing/images/7_test_curl_logs.png)


![8_test_curl_output.png](/assets/20241106_image_resizing/images/8_test_curl_output.png)

Great!
Now to actually resize the image...
This functionality should definitely be put into another module - we're no longer dealing with http requests.
So we create a new module img which will contain all of our image parsing and manipulation stuff

![9_new_img_module.png](/assets/20241106_image_resizing/images/9_new_img_module.png)

And insize `img/resizer.rs` we can add something like this to get it resizing images...
```
// image manipulation functionality

use image::DynamicImage;

pub async fn resize(image: &mut DynamicImage, width: u32, height: u32) -> DynamicImage {
    image.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
}
```
And then update our `resize` handler in lib.rs to call to a function that uses our new `resizer::resize` function:
```
async fn handle_resize(task: Task) -> Result<Vec<u8>> {
    match image_fetcher::serial_fetch(task.url.clone()).await {
        Ok(img) => {
            let img = resizer::resize(
                &mut img.clone(), 
                task.width.expect("NO WIDTH FOUND!"),
                task.height.expect("NO HEIGHT FOUND!")
            ).await;
            let mut bytes = Vec::<u8>::new();
            img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();
            return Ok(bytes);
        },
        Err(err) => {
            error!("Could not get the image {:?}", err);
            return Err(err)
        },
    }
}

#[debug_handler]
async fn resize(Query(task): Query<Task>) -> impl IntoResponse {
    // we need to fetch the image here
    info!(
        "Received request: url: {:?} w: {:?} h: {:?}",
        task.url, task.width, task.height
    );
    match handle_resize(task.into()).await {
        Ok(bytes) => {
            (
                StatusCode::OK,
                AppendHeaders([
                    ("Content-Type", "image/png")
                ]),
                bytes.into()
            )
        },
        Err(_err) => {
            (
                StatusCode::NOT_FOUND,
                AppendHeaders([
                    ("Content-Type", "image/png"),
                ]),
                Bytes::new()
            )
        },
    }
}
```
And now we can update our `test_curl` script to include a `width` and `height` parameters
And just like that we have a slightly wider peppermint butler

![10_wide_pep_but.png](/assets/20241106_image_resizing/images/10_wide_pep_but.png)


