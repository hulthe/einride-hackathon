use std::sync::mpsc;
use opencv::videoio::{CAP_FFMPEG, VideoCapture, VideoCaptureTrait};
use opencv::core::{Mat, MatTrait, Vector, CV_64FC1, CV_8UC3};


fn capture(mut cap: VideoCapture, tx: mpsc::Sender<Mat>) -> anyhow::Result<()> {
    loop {
        let mut frame = Mat::default();
        if !cap.read(&mut frame)? {
            println!("capture: no more frames");
            return Ok(());
        }

        tx.send(frame)?;
    }
}
pub fn start_capture() -> anyhow::Result<mpsc::Receiver<Mat>> {
    let (tx, rx) = mpsc::channel();

    let cap = VideoCapture::from_file("http://192.168.81.154:8887/video", CAP_FFMPEG)?;

    std::thread::spawn(move || {
        if let Err(e) = capture(cap, tx) {
            eprintln!("Error in capture thread: {}", e);
        }
    });

    Ok(rx)
}