use opencv::core::{bitwise_and, in_range, ElemMul, Mat, MatTrait, Vector, CV_64FC1, CV_8UC3};
use opencv::cvv::{show_image, CallMetaData};
use opencv::features2d::{
    draw_keypoints, DrawMatchesFlags, Feature2DTrait, SimpleBlobDetector, SimpleBlobDetector_Params,
};
use opencv::imgproc::{cvt_color, threshold, COLOR_RGB2BGR, COLOR_RGB2HSV, THRESH_BINARY};
use opencv::prelude::VideoCaptureTrait;
use opencv::videoio::{VideoCapture, CAP_FFMPEG};
use serde::Serialize;
use websocket::client::builder::ClientBuilder;
use websocket::sender::Sender;
use websocket::ws::sender::Sender as SenderTrait;
use websocket::Message;

const IMG_W: i32 = 160;
const IMG_H: i32 = 120;

#[derive(Debug, Serialize)]
struct Command {
    angle: f64,
    throttle: f64,
    drive_mode: &'static str,
    recording: bool,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            angle: 0.0,
            throttle: 0.0,
            drive_mode: "user",
            recording: false,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut ws = ClientBuilder::new("ws://192.168.81.154:8887/wsDrive")?.connect_insecure()?;
    let mut cap = VideoCapture::from_file("http://192.168.81.154:8887/video", CAP_FFMPEG)?;

    let mut frame = Mat::default();

    let mut detector = SimpleBlobDetector::create(SimpleBlobDetector_Params {
        filter_by_area: false,
        filter_by_inertia: false,
        filter_by_convexity: false,
        filter_by_circularity: false,
        min_area: 200.0,
        min_threshold: 1.0,
        max_threshold: 255.0,
        ..SimpleBlobDetector_Params::default()?
    })?;

    let deep_red: Vector<u8> = vec![15, 255, 255].into();
    let light_red: Vector<u8> = vec![0, 0, 0].into();

    loop {
        let got_frame = cap.read(&mut frame)?;

        let mut frame_bgr = Mat::default();
        cvt_color(&frame, &mut frame_bgr, COLOR_RGB2BGR, 0)?;

        let mut frame_hsv = Mat::default();
        cvt_color(&frame_bgr, &mut frame_hsv, COLOR_RGB2HSV, 0)?;

        let mut mask = Mat::default();
        in_range(&frame_hsv, &light_red, &deep_red, &mut mask)?;
        let mut masked = Mat::default();
        bitwise_and(&frame_bgr, &frame_bgr, &mut masked, &mask)?;

        let mut thresh = Mat::default();
        threshold(&masked, &mut thresh, 40.0, 255.0, THRESH_BINARY)?;

        let mut keypoints = Vector::new();
        detector.detect(&masked, &mut keypoints, &mask)?;

        let mut img_w_keypoints = Mat::default();
        draw_keypoints(
            &frame,
            &keypoints,
            &mut img_w_keypoints,
            (0.0, 255.0, 255.0).into(),
            DrawMatchesFlags::DRAW_RICH_KEYPOINTS,
        )?;

        let keypoints: Vec<_> = keypoints.to_vec();
        let best_point = keypoints
            .iter()
            .map(|keypoint| (keypoint.pt.x as i32, keypoint.pt.y as i32))
            .filter(|&(_, y)| y > IMG_H / 2)
            .next();

        let mut cmd = Command::default();
        if let Some((x, _y)) = best_point {
            let x_from_mid = IMG_W / 2 - x;
            cmd.angle = (x_from_mid as f64) / (IMG_W / 2) as f64;
            cmd.throttle = 0.00;
            println!("angle: {}, throttle: {}", cmd.angle, cmd.throttle);
        } else {
            println!("no points found");
        }
        let cmd = serde_json::to_string(&cmd)?;

        let msg = Message::text(&cmd);
        let mut sender = Sender::new(true);
        let mut buf = Vec::new();
        sender.send_message(&mut buf, &msg)?;
        ws.writer_mut().write_all(&buf)?;

        //show_image(&frame, &CallMetaData::default()?, "frame", "blah")?;
        //show_image(&frame_hsv, &CallMetaData::default()?, "hsv", "blah")?;
        //show_image(&masked, &CallMetaData::default()?, "masked", "blah")?;
        //show_image(&thresh, &CallMetaData::default()?, "thresholded", "blah")?;
        //show_image(
        //    &img_w_keypoints,
        //    &CallMetaData::default()?,
        //    "w_keypoints",
        //    "blah",
        //)?;
    }
}
