use eframe::{
    egui::{
        load::{Bytes, SizedTexture},
        CentralPanel, ImageSource,
    },
    run_native, App, NativeOptions, Renderer,
};
use three_d::*;

use crate::NodeGraph;

struct NodeGraphRenderer {
    node_graph: NodeGraph,
    three_d_info: ThreeDInfo,
}

impl App for NodeGraphRenderer {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        println!("Updating!");

        let context = &self.three_d_info.context;

        let output = self
            .node_graph
            .get_output()
            .into_gm_single(&self.three_d_info.context);
        println!("output created");
        let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
        let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));
        println!("lights added");
        let pixels: Vec<[u8; 4]> =
            render_three_d(&mut self.three_d_info, &[output], &[&light0, &light1]).read_color();
        let bytes = pixels.iter().flatten().copied().collect::<Vec<u8>>();
        //ctx.include_bytes("color", bytes);

        CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello World!");
            //ui.image(ImageSource::Uri("color".into()));
        });
    }
}

pub fn run() {
    let mut node_graph = crate::example();
    let app = NodeGraphRenderer {
        node_graph,
        three_d_info: setup_three_d(),
    };
    let mut win_options = NativeOptions::default();

    println!("Running!");
    let result = run_native(
        "Node Graph",
        win_options,
        Box::new(|context| {
            println!("Creating renderer!");
            Ok(Box::new(app))
        }),
    );
    println!("Result: {:?}", result);
}

struct ThreeDInfo {
    color: Texture2D,
    depth: DepthTexture2D,
    camera: Camera,
    context: HeadlessContext,
}

fn setup_three_d() -> ThreeDInfo {
    let width = 1280;
    let height = 720;
    let viewport = Viewport::new_at_origo(width, height);
    let context = HeadlessContext::new().unwrap();

    // Create a camera
    let camera = Camera::new_perspective(
        viewport,
        vec3(0.0, 0.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(60.0),
        0.1,
        10.0,
    );

    let color = Texture2D::new_empty::<[u8; 4]>(
        &context,
        width,
        height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    let depth = DepthTexture2D::new::<f32>(
        &context,
        width,
        height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    ThreeDInfo {
        color,
        depth,
        camera,
        context,
    }
}

fn render_three_d<'a>(
    info: &'a mut ThreeDInfo,
    gms: &[Gm<Mesh, PhysicalMaterial>],
    lights: &[&dyn Light],
) -> RenderTarget<'a> {
    let render_target = RenderTarget::new(
        info.color.as_color_target(None),
        info.depth.as_depth_target(),
    );

    render_target
        .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
        .render(&info.camera, gms, lights);

    render_target
}
