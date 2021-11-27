mod ws;
mod dead_mans;
mod gui;
mod track;

use opencv::core::{bitwise_and, in_range, ElemMul, Mat, MatTrait, Vector, CV_64FC1, CV_8UC3};
use opencv::features2d::{
    draw_keypoints, DrawMatchesFlags, Feature2DTrait, SimpleBlobDetector, SimpleBlobDetector_Params,
};
use opencv::imgproc::{cvt_color, threshold, COLOR_RGB2BGR, COLOR_RGB2HSV, THRESH_BINARY};
use opencv::prelude::VideoCaptureTrait;
use opencv::videoio::{VideoCapture, CAP_FFMPEG};

use opencv::highgui;

const IMG_W: i32 = 160;
const IMG_H: i32 = 120;


fn main() -> anyhow::Result<()> {
    let mut cap = VideoCapture::from_file("http://192.168.81.154:8887/video", CAP_FFMPEG)?;
    let ws = ws::connect_ws()?;
    let ws = dead_mans::start_dms(ws)?;

    const WINDOW_NAME: &str = "video";

    highgui::named_window(WINDOW_NAME, 0)?;


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
        if !cap.read(&mut frame)? {
            println!("no more frames");
            return Ok(());
        }

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

        let mut cmd = ws::Command::default();
        if let Some((x, _y)) = best_point {
            let x_from_mid = IMG_W / 2 - x;
            cmd.angle = (x_from_mid as f64) / (IMG_W / 2) as f64;
            cmd.throttle = 0.20;
            eprintln!("angle: {}, throttle: {}", cmd.angle, cmd.throttle);
        } else {
            eprintln!("no points found");
        }
        ws.send(cmd)?;

        highgui::imshow(WINDOW_NAME, &img_w_keypoints)?;
        highgui::wait_key(50)?;
        //std::thread::sleep(std::time::Duration::from_millis(50))
    }
}
