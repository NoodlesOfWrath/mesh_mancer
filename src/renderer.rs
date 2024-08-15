use eframe::{
    egui::{
        load::{Bytes, SizedTexture},
        pos2, vec2, Area, CentralPanel, Color32, Frame, Id, ImageSource, Pos2, Rect, Rounding,
        Sense, Shadow, Vec2,
    },
    run_native, App, HardwareAcceleration, NativeOptions, Renderer,
};
use three_d::*;

use crate::{NodeAny, NodeGraph};

use std::hash::Hash;
use std::hash::Hasher;

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
        const DEFAULT_POSITION: Pos2 = pos2(200.0, 200.0);
        const DEFAULT_SIZE: Vec2 = vec2(100.0, 100.0);
        let mut positions = vec![DEFAULT_POSITION; self.node_graph.get_nodes().len()];
        let mut sizes = vec![DEFAULT_SIZE; self.node_graph.get_nodes().len()];
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

    fn init(&mut self) {
        self.setup_positions();
    }

    fn step(&mut self) {
        const SPEED: f32 = 0.03;

        // move the nodes away from each other
        for i in 0..self.node_graph.get_nodes().len() {
            for j in 0..self.node_graph.get_nodes().len() {
                if i == j {
                    continue;
                }

                // the positions are in the left top corner of the node
                // if the nodes are overlapping, move them away from each other
                let pos_i = self.get_node_position(i);
                let pos_j = self.get_node_position(j);
                let size_i = self.get_node_size(i);
                let size_j = self.get_node_size(j);

                println!("{} {}", size_i.x, size_i.y);

                let i_right = pos_i.x + size_i.x;
                let i_left = pos_i.x;
                let i_bottom = pos_i.y + size_i.y;
                let i_top = pos_i.y;

                let j_right = pos_j.x + size_j.x;
                let j_left = pos_j.x;
                let j_bottom = pos_j.y + size_j.y;
                let j_top = pos_j.y;

                let mut movement = vec2(0.0, 0.0);
                // cover each condition where the nodes are overlapping
                if i_right > j_left && i_left < j_right && i_bottom > j_top && i_top < j_bottom {
                    // move the centers away from each other
                    let center_i = pos_i + size_i / 2.0;
                    let center_j = pos_j + size_j / 2.0;
                    movement = center_i - center_j;
                }

                self.set_node_position(i, pos_i + movement * SPEED);
            }
        }
    }
}

struct NodeGraphRenderer {
    visual_node_graph: VisualNodeGraph,
    was_dragging: bool,
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
                self.was_dragging = false;
                ctx.input(|input| {
                    if input.pointer.any_down() {
                        self.was_dragging = true;
                    }
                });

                if !self.was_dragging {
                    //self.visual_node_graph.step();
                }

                // add a node to the graph
                for i in 0..self.visual_node_graph.node_graph.get_nodes().len() {
                    let node = self.visual_node_graph.node_graph.get_node(i);

                    let response = show_node(
                        node,
                        self.visual_node_graph.get_node_position(i),
                        ctx,
                        &self.visual_node_graph.scheme,
                    );

                    self.visual_node_graph.set_node_position(i, response.pos);
                    self.visual_node_graph.set_node_size(i, response.size);
                }

                //ui.image(ImageSource::Uri("color".into()));
            });

        // i don't really feel like this is correct or necessary but it's here for now
        ctx.request_repaint();
    }
}

struct NodeResponse {
    pos: Pos2,
    size: Vec2,
}

fn show_node(
    node: &dyn NodeAny,
    pos: Pos2,
    ctx: &eframe::egui::Context,
    scheme: &ColorScheme,
) -> NodeResponse {
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

    let area = Area::new(Id::new(node.name()))
        .current_pos(pos)
        .movable(true);

    let response = area.show(ctx, |ui| {
        // display a number of spheres equal to the number of inputs on the left of the node
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for needed_type in node.needed_types_input() {
                    let (rect, painter) =
                        ui.allocate_painter(Vec2::new(10.0, 10.0), Sense::hover());

                    let center = rect.rect.center();
                    let radius = 5.0;
                    // hash the type id to get a color
                    let color = hash_type_id(needed_type);

                    painter.circle_filled(center, radius, color);
                }
            });
            container.show(ui, |ui| {
                ui.label(node.name()).on_hover_text(node.description());
            });
            ui.vertical(|ui| {
                for needed_type in node.needed_types_output() {
                    if node.name() == "Output" {
                        continue;
                    }

                    let (rect, painter) =
                        ui.allocate_painter(Vec2::new(10.0, 10.0), Sense::hover());

                    let center = rect.rect.center();
                    let radius = 5.0;
                    // hash the type id to get a color
                    let color = hash_type_id(needed_type);

                    painter.circle_filled(center, radius, color);
                }
            });
        });
    });

    if response.response.dragged() {
        let new_pos = pos + response.response.drag_delta();
        return NodeResponse {
            pos: new_pos,
            size: response.response.rect.size(),
        };
    }

    NodeResponse {
        pos,
        size: response.response.rect.size(),
    }
}

fn hash_type_id(type_id: std::any::TypeId) -> Color32 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    type_id.hash(&mut hasher);
    let hash = hasher.finish();
    Color32::from_rgb(hash as u8, (hash >> 8) as u8, (hash >> 16) as u8)
}

pub fn run() {
    let node_graph = crate::example();
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
    visual_node_graph.init();

    let app = NodeGraphRenderer {
        visual_node_graph: visual_node_graph,
        was_dragging: false,
        //three_d_info: setup_three_d(),
    };
    let mut win_options = NativeOptions::default();
    win_options.hardware_acceleration = HardwareAcceleration::Preferred;

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
