use bevy::{color::palettes::basic::PURPLE, prelude::*};

use gstreamer::prelude::*;
use gstreamer::prelude::{Cast, ElementExt};
use gstreamer_app::AppSink;
// use gstreamer_app::prelude::AppSinkExt;
use gstreamer_app::AppSinkCallbacks;

use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use std::sync::{Arc, Mutex};

fn main() {
    gstreamer::init().expect("Failed to initialize GStreamer");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, upload_frame)
        .run();
}

#[derive(Resource)]
struct VideoFrameHandle(Handle<Image>);

/// Handle so we can fetch the Image asset inside the update system
#[derive(Resource)]
struct VideoHandle(Handle<Image>);

/// A frame buffer shared between the GStreamer thread and the Bevy world
#[derive(Resource, Clone)]
struct SharedFrame(std::sync::Arc<std::sync::Mutex<Option<Vec<u8>>>>);

use bevy::asset::*;
use bevy::image::ImageSampler;
use bevy::render::render_resource::Extent3d;
use bevy::render::render_resource::*;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Create an empty RGBA8 texture (placeholder)
    let size = Extent3d {
        width: 640,
        height: 480,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();

    let image_handle = images.add(image);
    commands.insert_resource(VideoFrameHandle(image_handle.clone()));

    commands.spawn((
        Sprite::from_image(image_handle.clone()),
        Transform::from_scale(Vec3::splat(1.0)),
    ));

    // Shared buffer for frames
    let shared = SharedFrame(std::sync::Arc::new(std::sync::Mutex::new(None)));
    commands.insert_resource(shared.clone());

    // start the pipeline in a background thread
    std::thread::spawn(move || start_gst_pipeline(shared));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        Transform::default().with_scale(Vec3::splat(128.)),
    ));
}

fn start_gst_pipeline(shared: SharedFrame) {
    gst::init().unwrap();

    let pipeline_str = "\
  udpsrc port=5000 caps=\
    \"application/x-rtp,media=video,encoding-name=H264,\
      payload=96,clock-rate=90000,packetization-mode=1\" ! \
  rtph264depay ! avdec_h264 ! videoconvert ! \
  video/x-raw,format=RGBA ! \
  appsink name=sink";

    let pipeline = gst::parse::launch(pipeline_str)
        .unwrap()
        .downcast::<gst::Pipeline>()
        .unwrap();

    let appsink = pipeline
        .by_name("sink")
        .unwrap()
        .downcast::<gst_app::AppSink>()
        .unwrap();

    appsink.set_property("emit-signals", &true);
    appsink.set_property("sync", &false);

    let shared_clone = shared.clone(); // capture once
    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = sink.pull_sample().unwrap();
                let buffer = sample.buffer().unwrap();
                let map = buffer.map_readable().unwrap();

                *shared_clone.0.lock().unwrap() = Some(map.as_slice().to_vec());
                // println!("got frame {}", map.as_slice().len());
                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    // ── Also print bus errors / state changes ───────────────────────────
    let bus = pipeline.bus().unwrap();
    pipeline.set_state(gst::State::Playing).unwrap();

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView::*;
        match msg.view() {
            Error(e) => eprintln!("BUS ERROR {:?}", e.error()),
            StateChanged(s) if s.src().map(|s| s == &pipeline).unwrap_or(false) => {
                println!("state to {:?}", s.current());
            }
            _ => {}
        }
    }

    pipeline.set_state(gst::State::Playing).unwrap();
    // Keep the thread alive
    let _ = pipeline.state(gst::ClockTime::NONE);
}

fn upload_frame(
    shared: Res<SharedFrame>,
    video: Res<VideoFrameHandle>,
    mut images: ResMut<Assets<Image>>,
) {
    // 1. take the newest frame out of the mutex
    if let Some(frame) = shared.0.lock().unwrap().take() {
        // 2. get the Image we want to modify
        if let Some(image) = images.get_mut(&video.0) {
            // 3. ensure `image.data` exists
            let data = image.data.get_or_insert_with(Vec::new);

            // 4. resize if needed and copy the bytes
            if data.len() != frame.len() {
                data.resize(frame.len(), 0);
            }
            data.copy_from_slice(&frame);
        }
    }
}
