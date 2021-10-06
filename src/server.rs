mod backend;

use clap::Clap;

#[derive(Clap)]
struct Opts {
 #[clap(short, long)]
 cert_path: Option<String>,
 #[clap(short, long)]
 key_path: Option<String>,
 #[clap(short, long, default_value = "8080")]
 port: u16,
}

#[tokio::main]
async fn main() -> Result<(), std::boxed::Box<dyn std::error::Error>> {
 #[derive(rust_embed::RustEmbed)]
 #[folder = "build"]
 struct Frontend;

 pretty_env_logger::init_timed();
 let opts = Opts::parse();

 log::warn!("warn enabled");
 log::info!("info enabled");
 log::debug!("debug enabled");
 log::trace!("trace enabled");

 let _ = std::thread::spawn(|| {
  librclone::initialize();
  dbg!(librclone::rpc("operations/list", r#"{"fs":"putio:","remote":""}"#)).unwrap();
 });

 match (opts.key_path, opts.cert_path) {
  (Some(key_path), Some(cert_path)) => {
   eprintln!("Serving HTTPS on port {}", opts.port);
   warp::serve(turbocharger::warp_routes(Frontend))
    .tls()
    .cert_path(cert_path)
    .key_path(key_path)
    .run(([0, 0, 0, 0], opts.port))
    .await;
  }
  (None, None) => {
   eprintln!("Serving (unsecured) HTTP on port {}", opts.port);
   opener::open(format!("http://127.0.0.1:{}", opts.port)).ok();
   warp::serve(turbocharger::warp_routes(Frontend)).run(([0, 0, 0, 0], opts.port)).await;
  }
  _ => eprintln!("Both key-path and cert-path must be specified for HTTPS."),
 }

 Ok(())
}

use four_cc::FourCC;
use nom::bytes::streaming::take;
use nom::number::streaming::{be_u32, be_u64};
use nom::IResult;
use std::convert::TryInto;

#[derive(Debug)]
struct MediaBox {
 size: u64,
 ty: FourCC,
}

impl MediaBox {
 fn parse(i: &[u8]) -> IResult<&[u8], MediaBox> {
  let (i, size) = be_u32(i)?;
  let (i, ty) = take(4usize)(i)?;
  let ty: FourCC = ty.try_into().unwrap();
  let (i, size) = match size {
   1 => be_u64(i)?,
   _ => (i, size.into()),
  };

  Ok((i, MediaBox { size, ty }))
 }
}
