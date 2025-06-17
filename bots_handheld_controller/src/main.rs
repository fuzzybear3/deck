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
        &[0, 50, 0, 255],
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

fn setup_gst_pipeline() -> Result<gst_app::AppSink, Box<dyn std::error::Error>> {
    gst::init()?; // must be called before using GStreamer

    let pipeline = gst::parse::launch(
        "videotestsrc ! videoconvert ! video/x-raw,format=RGBA,width=640,height=480 ! appsink name=sink",
    )?;

    let pipeline = pipeline.downcast::<gst::Pipeline>().unwrap();
    let appsink = pipeline
        .by_name("sink")
        .expect("Sink element not found")
        .downcast::<gst_app::AppSink>()
        .expect("Element is not an appsink");

    appsink.set_property("emit-signals", &true);
    appsink.set_property("sync", &false);

    pipeline.set_state(gst::State::Playing)?;

    Ok(appsink)
}

fn start_gst_pipeline(shared: SharedFrame) {
    gst::init().unwrap();

    let pipeline = gst::parse::launch(
        "videotestsrc ! videoconvert ! video/x-raw,format=RGBA,width=640,height=480 ! appsink name=sink",
    )
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

    // // Whenever a new sample arrives, copy it into the shared buffer
    // appsink.connect_new_sample(move |sink| {
    //     let sample = sink.pull_sample().unwrap();
    //     let buffer = sample.buffer().unwrap();
    //     let map = buffer.map_readable().unwrap();
    //     let data = map.as_slice();
    //
    //     let mut guard = shared.0.lock().unwrap();
    //     *guard = Some(data.to_vec());
    //     gst::FlowSuccess::Ok
    // });

    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = sink.pull_sample().unwrap();
                let buffer = sample.buffer().unwrap();
                let map = buffer.map_readable().unwrap();
                let bytes = map.as_slice();

                let mut guard = shared.0.lock().unwrap();
                *guard = Some(bytes.to_vec());

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

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
