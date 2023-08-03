use std::time::Duration;

use bevy::{prelude::*, render::{camera::ScalingMode, render_resource::{Texture, PipelineCache, BindGroupDescriptor, BindGroupEntry, BindingResource, RenderPassColorAttachment, Operations, RenderPassDescriptor, CachedRenderPipelineId, BindGroupLayout, Sampler, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, TextureSampleType, TextureViewDimension, SamplerBindingType, RawRenderPipelineDescriptor, SamplerDescriptor, RenderPipelineDescriptor, FragmentState, ColorTargetState, TextureFormat, ColorWrites, PrimitiveState, MultisampleState}, extract_component::{ExtractComponentPlugin, ComponentUniforms}, view::{ViewTarget, ExtractedView}, render_graph::{RenderGraph, RenderGraphContext, NodeRunError, Node, RenderGraphApp}, renderer::{RenderContext, RenderDevice}, texture::BevyDefault, RenderApp}, reflect::TypeUuid, core_pipeline::{fullscreen_vertex_shader::fullscreen_shader_vertex_state, core_3d}, asset::ChangeWatcher};

fn main() {
    // Set up the Bevy app
    App::new()
    .add_plugins((
        DefaultPlugins.set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(Duration::from_secs(1)),
            ..default()
        }),
        PostProcessPlugin,
    ))
    .init_resource::<GridWalkable>()
    .add_systems(Startup, setup)
    .add_systems(Update, player_movement)
    .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Camera;

#[derive(Resource, Default)]
struct GridWalkable {
    pub grid: Vec<Vec<bool>>,
    pub cooldown: Timer,
    pub current_position: (usize, usize),
}

struct PostProcessPlugin;

impl Plugin for PostProcessPlugin {
    fn build(&self, app: &mut App) {
        // We need to get the render app from the main app
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Bevy's renderer uses a render graph which is a collection of nodes in a directed acyclic graph.
            // It currently runs on each view/camera and executes each node in the specified order.
            // It will make sure that any node that needs a dependency from another node
            // only runs when that dependency is done.
            //
            // Each node can execute arbitrary work, but it generally runs at least one render pass.
            // A node only has access to the render world, so if you need data from the main world
            // you need to extract it manually or with the plugin like above.
            // Add a [`Node`] to the [`RenderGraph`]
            // The Node needs to impl FromWorld
            .add_render_graph_node::<PostProcessNode>(
                // Specify the name of the graph, in this case we want the graph for 3d
                core_3d::graph::NAME,
                // It also needs the name of the node
                PostProcessNode::NAME,
            )
            .add_render_graph_edges(
                core_3d::graph::NAME,
                // Specify the node ordering.
                // This will automatically create all required node edges to enforce the given ordering.
                &[
                    core_3d::graph::node::TONEMAPPING,
                    PostProcessNode::NAME,
                    core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
                ],
            );
    }

    fn finish(&self, app: &mut App) {
        // We need to get the render app from the main app
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            // Initialize the pipeline
            .init_resource::<PostProcessPipeline>();
    }
}

struct PostProcessNode {
    query: QueryState<&'static ViewTarget, With<ExtractedView>>,
}

impl PostProcessNode {
    pub const NAME: &'static str = "post_process";
}

impl FromWorld for PostProcessNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for PostProcessNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph_context: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Get the entity of the view for the render graph where this node is running
        let view_entity = graph_context.view_entity();

        // We get the data we need from the world based on the view entity passed to the node.
        // The data is the query that was defined earlier in the [`PostProcessNode`]
        let Ok(view_target) = self.query.get_manual(world, view_entity) else {
            return Ok(());
        };

        // Get the pipeline resource that contains the global data we need to create the render pipeline
        let post_process_pipeline = world.resource::<PostProcessPipeline>();

        // The pipeline cache is a cache of all previously created pipelines.
        // It is required to avoid creating a new pipeline each frame, which is expensive due to shader compilation.
        let pipeline_cache = world.resource::<PipelineCache>();

        // Get the pipeline from the cache
        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id) else {
            return Ok(());
        };

        // This will start a new "post process write", obtaining two texture
        // views from the view target - a `source` and a `destination`.
        // `source` is the "current" main texture and you _must_ write into
        // `destination` because calling `post_process_write()` on the
        // [`ViewTarget`] will internally flip the [`ViewTarget`]'s main
        // texture to the `destination` texture. Failing to do so will cause
        // the current main texture information to be lost.
        let post_process = view_target.post_process_write();

        // The bind_group gets created each frame.
        //
        // Normally, you would create a bind_group in the Queue set, but this doesn't work with the post_process_write().
        // The reason it doesn't work is because each post_process_write will alternate the source/destination.
        // The only way to have the correct source/destination for the bind_group is to make sure you get it during the node execution.
        let bind_group = render_context
            .render_device()
            .create_bind_group(&BindGroupDescriptor {
                label: Some("post_process_bind_group"),
                layout: &post_process_pipeline.layout,
                // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        // Make sure to use the source view
                        resource: BindingResource::TextureView(post_process.source),
                    },
                    BindGroupEntry {
                        binding: 1,
                        // Use the sampler created for the pipeline
                        resource: BindingResource::Sampler(&post_process_pipeline.sampler),
                    },
                ],
            });

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                // We need to specify the post process destination view here
                // to make sure we write to the appropriate texture.
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        });

        // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
        // using the pipeline/bind_group created above
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
    
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("post_process_bind_group_layout"),
            entries: &[
                // The screen texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // The sampler that will be used to sample the screen texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world
            .resource::<AssetServer>()
            .load("pixel_art.wgsl");

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("post_process_pipeline".into()),
                layout: vec![layout.clone()],
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all field can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid_walkable: ResMut<GridWalkable>,
) {
    // camera
    commands.spawn((Camera3dBundle {
        projection: OrthographicProjection {
            scale: 4.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },
    Camera));

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // cube
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    },
    Player));

    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..default()
    });

    // grid
    let grid: Vec<Vec<bool>> = vec![
        vec![false, false, false, false, false, false, false],
        vec![false, true, true, true, true, true, false],
        vec![false, true, true, true, true, true, false],
        vec![false, true, true, true, true, true, false],
        vec![false, true, true, true, true, true, false],
        vec![false, true, true, true, true, true, false],
        vec![false, false, false, false, false, false, false],
    ];
    grid_walkable.grid = grid;
    grid_walkable.cooldown = Timer::from_seconds(0.15, TimerMode::Once);
    grid_walkable.current_position = (3, 3);
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut grid: ResMut<GridWalkable>,
    time: Res<Time>,
    mut query: Query<&mut Transform, Or<(With<Player>, With<Camera>)>>,
) {
    if grid.cooldown.tick(time.delta()).finished() {
        let mut moved = false;

        if keyboard_input.pressed(KeyCode::W) && grid.grid[grid.current_position.0 - 1][grid.current_position.1] {
            grid.current_position.0 -= 1;
            for mut transform in query.iter_mut() {
                transform.translation.z -= 1.0;
            }
            moved = true;
        }
        if keyboard_input.pressed(KeyCode::S) && grid.grid[grid.current_position.0 + 1][grid.current_position.1] {
            grid.current_position.0 += 1;
            for mut transform in query.iter_mut() {
                transform.translation.z += 1.0;
            }
            moved = true;
        }
        if keyboard_input.pressed(KeyCode::A) && grid.grid[grid.current_position.0][grid.current_position.1 - 1] {
            grid.current_position.1 -= 1;
            for mut transform in query.iter_mut() {
                transform.translation.x -= 1.0;
            }
            moved = true;
        }
        if keyboard_input.pressed(KeyCode::D) && grid.grid[grid.current_position.0][grid.current_position.1 + 1] {
            grid.current_position.1 += 1;
            for mut transform in query.iter_mut() {
                transform.translation.x += 1.0;
            }
            moved = true;
        }

        if moved {
            grid.cooldown.reset();
        }
    }
}