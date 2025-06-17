use bevy::{color::palettes::basic::PURPLE, prelude::*};

use gstreamer::prelude::*;
use gstreamer::prelude::{Cast, ElementExt};
use gstreamer_app::AppSink;
// use gstreamer_app::prelude::AppSinkExt;

use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;
use std::sync::{Arc, Mutex};

fn main() {
    gstreamer::init().expect("Failed to initialize GStreamer");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Resource)]
struct VideoFrameHandle(Handle<Image>);

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

    /* --- store the handle so another system / thread can write pixels ---- */
    // commands.insert_resource(VideoTex(tex_handle));

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
