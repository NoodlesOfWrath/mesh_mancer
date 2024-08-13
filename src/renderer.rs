use eframe::{
    egui::{
        load::{Bytes, SizedTexture},
        pos2, vec2, Area, CentralPanel, Color32, Frame, Id, ImageSource, Pos2, Rect, Rounding,
        Shadow,
    },
    run_native, App, NativeOptions, Renderer,
};
use three_d::*;

use crate::{NodeAny, NodeGraph};

struct ColorScheme {
    background: Color32,
    node_background: Color32,
    node_text: Color32,
}

// a node graph but with extra information for rendering
struct VisualNodeGraph {
    node_graph: NodeGraph,
    positions: Vec<Pos2>,
    sizes: Vec<eframe::egui::Vec2>,
    scheme: ColorScheme,
}

impl VisualNodeGraph {
    fn setup_positions(&mut self) {
        let mut positions = Vec::new();
        let mut sizes = Vec::new();
        for _ in 0..self.node_graph.get_nodes().len() {
            positions.push(pos2(0.0, 0.0));
            sizes.push(vec2(100.0, 100.0));
        }
        self.positions = positions;
        self.sizes = sizes;
    }

    fn get_node_position(&self, node_index: usize) -> Pos2 {
        self.positions[node_index]
    }

    fn set_node_position(&mut self, node_index: usize, position: Pos2) {
        self.positions[node_index] = position;
    }

    fn get_node_size(&self, node_index: usize) -> eframe::egui::Vec2 {
        self.sizes[node_index]
    }

    fn set_node_size(&mut self, node_index: usize, size: eframe::egui::Vec2) {
        self.sizes[node_index] = size;
    }

    fn step(&mut self) {
        const SPEED: f32 = 0.1;

        // move the nodes away from each other
        for i in 0..self.node_graph.get_nodes().len() {
            for j in 0..self.node_graph.get_nodes().len() {
                if i == j {
                    continue;
                }

                let pos_i = self.get_node_position(i);
                let pos_j = self.get_node_position(j);
                let distance = pos_i.distance(pos_j);
                if distance == 0.0 {
                    let new_pos_i = pos_i + vec2(1.0, 1.0) * SPEED;
                    self.set_node_position(i, new_pos_i);
                } else if distance < 100.0 {
                    let direction = pos_i - pos_j;
                    let direction = direction / distance;
                    let new_pos_i = pos_i + direction * SPEED * (100.0 - distance) / 2.0;

                    self.set_node_position(i, new_pos_i);
                }
            }
        }
    }
}

struct NodeGraphRenderer {
    visual_node_graph: VisualNodeGraph,
    //three_d_info: ThreeDInfo,
}

impl App for NodeGraphRenderer {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        /*
        let context = &self.three_d_info.context;

        let output = self
            .node_graph
            .get_output()
            .into_gm_single(&self.three_d_info.context);

        let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
        let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));
        let pixels: Vec<[u8; 4]> =
            render_three_d(&mut self.three_d_info, &[output], &[&light0, &light1]).read_color();
        let bytes = pixels.iter().flatten().copied().collect::<Vec<u8>>();
        ctx.include_bytes("color", bytes);
        */

        CentralPanel::default()
            .frame(Frame::default().fill(self.visual_node_graph.scheme.background))
            .show(ctx, |ui| {
                self.visual_node_graph.step();
                // add a node to the graph
                for (i, node) in self
                    .visual_node_graph
                    .node_graph
                    .get_nodes()
                    .iter()
                    .enumerate()
                {
                    spawn_node(
                        *node,
                        self.visual_node_graph.get_node_position(i),
                        ui,
                        ctx,
                        &self.visual_node_graph.scheme,
                    );
                }

                //ui.image(ImageSource::Uri("color".into()));
            });
    }
}

fn spawn_node(
    node: &dyn NodeAny,
    pos: Pos2,
    ui: &mut eframe::egui::Ui,
    ctx: &eframe::egui::Context,
    scheme: &ColorScheme,
) {
    // Create a frame with rounded corners
    let container = Frame::default()
        .rounding(12.0)
        .inner_margin(12.0)
        .fill(scheme.node_background)
        .shadow(Shadow {
            offset: vec2(0.0, 0.0),
            blur: 4.0,
            spread: 0.0,
            color: Color32::BLACK,
        });

    Area::new(Id::new(node.name()))
        .fixed_pos(pos)
        .show(ctx, |ui| {
            container.show(ui, |ui| {
                ui.label(node.name()).on_hover_text(node.description());
            });
        });
}

pub fn run() {
    let mut node_graph = crate::example();
    let midnight_scheme = ColorScheme {
        background: Color32::from_gray(50),
        node_background: Color32::from_gray(0),
        node_text: Color32::from_gray(255),
    };
    let mut visual_node_graph = VisualNodeGraph {
        node_graph,
        positions: Vec::new(),
        sizes: Vec::new(),
        scheme: midnight_scheme,
    };
    visual_node_graph.setup_positions();

    let app = NodeGraphRenderer {
        visual_node_graph: visual_node_graph,
        //three_d_info: setup_three_d(),
    };
    let mut win_options = NativeOptions::default();

    let result = run_native(
        "Node Graph",
        win_options,
        Box::new(|context| Ok(Box::new(app))),
    );
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
