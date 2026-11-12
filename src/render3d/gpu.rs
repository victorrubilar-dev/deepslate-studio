//! Dibujo con wgpu en una textura propia, que luego se muestra como imagen de egui.
//! La misma textura sirve para la captura a PNG.

use super::mesh::{self, Camera, Vertex};
use super::pose::Pose;
use crate::core::{Layer, Model, Part};
use eframe::egui;
use eframe::egui_wgpu::{RenderState, wgpu};
use image::RgbaImage;
use wgpu::util::DeviceExt;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const SAMPLES: u32 = 4;

pub struct Scene<'a> {
    pub model: Model,
    pub pose: &'a Pose,
    pub camera: &'a Camera,
    pub selected: Option<Part>,
    pub visible: &'a dyn Fn(Part, Layer) -> bool,
    /// Dibujar la capa de la espalda con la textura subida en `upload_cape`.
    pub cape: bool,
    /// 0 = color plano, 1 = luz direccional completa.
    pub lighting: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    mvp: [[f32; 4]; 4],
    light: [f32; 4],
    selected: [f32; 4],
}

struct Target {
    size: [u32; 2],
    color: wgpu::Texture,
    msaa: wgpu::TextureView,
    depth: wgpu::TextureView,
}

pub struct Renderer3d {
    rs: RenderState,
    base: wgpu::RenderPipeline,
    overlay: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    cape_group: wgpu::BindGroup,
    uniforms: wgpu::Buffer,
    skin: wgpu::Texture,
    cape: wgpu::Texture,
    target: Option<Target>,
    texture_id: Option<egui::TextureId>,
}

impl Renderer3d {
    pub fn new(rs: &RenderState) -> Self {
        let device = &rs.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("skin3d"),
            source: wgpu::ShaderSource::Wgsl(include_str!("skin.wgsl").into()),
        });
        let texture = |label, height| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: 64,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };
        let skin = texture("skin", 64);
        let cape = texture("cape", 32);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default()); // Nearest por defecto
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniforms"),
            size: size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("skin3d"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let group = |tex: &wgpu::Texture| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("skin3d"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniforms.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &tex.create_view(&Default::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
        };
        let (bind_group, cape_group) = (group(&skin), group(&cape));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("skin3d"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |fs: &str, blend: Option<wgpu::BlendState>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(fs),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3, 3 => Float32],
                    })],
                },
                // Sin descarte de caras traseras: el orden de vértices no importa.
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState { count: SAMPLES, ..Default::default() },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: FORMAT,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        // El resultado queda premultiplicado, que es lo que espera egui.
        let blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        };
        Self {
            base: pipeline("fs_base", None),
            overlay: pipeline("fs_overlay", Some(blend)),
            rs: rs.clone(),
            bind_group,
            cape_group,
            uniforms,
            skin,
            cape,
            target: None,
            texture_id: None,
        }
    }

    pub fn upload_skin(&self, pixels: &RgbaImage) {
        self.upload(&self.skin, pixels);
    }

    /// Textura de 64×32 para la capa de la espalda.
    pub fn upload_cape(&self, pixels: &RgbaImage) {
        self.upload(&self.cape, pixels);
    }

    fn upload(&self, tex: &wgpu::Texture, pixels: &RgbaImage) {
        let size = tex.size();
        debug_assert_eq!((pixels.width(), pixels.height()), (size.width, size.height));
        self.rs.queue.write_texture(
            tex.as_image_copy(),
            pixels.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(size.width * 4),
                rows_per_image: Some(size.height),
            },
            size,
        );
    }

    /// Dibuja la escena al tamaño pedido (en píxeles físicos) y devuelve la textura para egui.
    pub fn render(&mut self, size: [u32; 2], scene: &Scene) -> egui::TextureId {
        let size = size.map(|v| v.clamp(1, 4096));
        if self.target.as_ref().is_none_or(|t| t.size != size) {
            self.target = Some(self.make_target(size));
        }
        let target = self.target.as_ref().expect("recién creado");
        self.draw(target, scene);
        let view = target.color.create_view(&Default::default());
        let mut renderer = self.rs.renderer.write();
        match self.texture_id {
            Some(id) => renderer.update_egui_texture_from_wgpu_texture(
                &self.rs.device,
                &view,
                wgpu::FilterMode::Linear,
                id,
            ),
            None => {
                self.texture_id = Some(renderer.register_native_texture(
                    &self.rs.device,
                    &view,
                    wgpu::FilterMode::Linear,
                ))
            }
        }
        self.texture_id.expect("registrada arriba")
    }

    /// Dibuja la escena a un tamaño fijo y la lee de vuelta como imagen con transparencia.
    /// `None` si la GPU no devolvió los datos.
    pub fn capture(&self, size: u32, scene: &Scene) -> Option<RgbaImage> {
        let target = self.make_target([size, size]);
        self.draw(&target, scene);
        let row = (size * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.rs.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("captura"),
            size: (row * size) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut enc = self.rs.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            target.color.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(size),
                },
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        );
        self.rs.queue.submit([enc.finish()]);
        buffer.map_async(wgpu::MapMode::Read, .., |_| {});
        self.rs
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .ok()?;
        let data = buffer.get_mapped_range(..).ok()?;
        let mut img = RgbaImage::new(size, size);
        for (y, line) in data.chunks(row as usize).take(size as usize).enumerate() {
            for (x, px) in line[..(size * 4) as usize]
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
            {
                img.put_pixel(x as u32, y as u32, image::Rgba(unpremultiply(*px)));
            }
        }
        Some(img)
    }

    fn make_target(&self, [w, h]: [u32; 2]) -> Target {
        let tex = |label, format, samples, usage| {
            self.rs.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let attach = wgpu::TextureUsages::RENDER_ATTACHMENT;
        let color = tex(
            "3d",
            FORMAT,
            1,
            attach | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
        );
        let msaa = tex("3d-msaa", FORMAT, SAMPLES, attach).create_view(&Default::default());
        let depth = tex("3d-depth", DEPTH, SAMPLES, attach).create_view(&Default::default());
        Target {
            size: [w, h],
            color,
            msaa,
            depth,
        }
    }

    fn draw(&self, target: &Target, scene: &Scene) {
        let device = &self.rs.device;
        let (base, overlay) = mesh::build(scene.model, scene.pose, scene.camera, scene.visible);
        let cape = if scene.cape {
            mesh::cape(scene.pose, scene.camera)
        } else {
            Vec::new()
        };
        let [w, h] = target.size;
        let u = Uniforms {
            mvp: mesh::projection(w as f32 / h as f32, scene.camera),
            light: [-0.45, 0.6, 0.65, scene.lighting],
            selected: [
                scene.selected.map_or(-1.0, |p| p as usize as f32),
                0.0,
                0.0,
                0.0,
            ],
        };
        self.rs
            .queue
            .write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&u));
        let buffer = |v: &[Vertex]| {
            (!v.is_empty()).then(|| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(v),
                    usage: wgpu::BufferUsages::VERTEX,
                })
            })
        };
        let (base_buf, overlay_buf, cape_buf) = (buffer(&base), buffer(&overlay), buffer(&cape));
        let resolve = target.color.create_view(&Default::default());
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("skin3d"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.msaa,
                    depth_slice: None,
                    resolve_target: Some(&resolve),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &target.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            for (pipeline, group, buf, n) in [
                (&self.base, &self.cape_group, &cape_buf, cape.len()),
                (&self.base, &self.bind_group, &base_buf, base.len()),
                (&self.overlay, &self.bind_group, &overlay_buf, overlay.len()),
            ] {
                if let Some(buf) = buf {
                    pass.set_bind_group(0, group, &[]);
                    pass.set_pipeline(pipeline);
                    pass.set_vertex_buffer(0, buf.slice(..));
                    pass.draw(0..n as u32, 0..1);
                }
            }
        }
        self.rs.queue.submit([enc.finish()]);
    }
}

fn unpremultiply([r, g, b, a]: [u8; 4]) -> [u8; 4] {
    if a == 0 {
        return [0, 0, 0, 0];
    }
    let f = |c: u8| ((c as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
    [f(r), f(g), f(b), a]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{FACES, Face, LAYERS, PARTS, Skin, rect};
    use crate::render3d::Preset;
    use eframe::egui::mutex::RwLock;
    use eframe::egui_wgpu;
    use std::sync::Arc;

    /// Los futuros de wgpu nativo se resuelven solos; basta con sondearlos.
    fn block_on<F: Future>(f: F) -> F::Output {
        let mut f = std::pin::pin!(f);
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        loop {
            if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut cx) {
                return v;
            }
            std::thread::yield_now();
        }
    }

    /// Cada cara de un color, con la esquina de arriba a la izquierda en negro,
    /// para revisar a ojo la orientación de las texturas.
    fn test_skin() -> Skin {
        let mut s = Skin::blank(Model::Steve);
        for part in PARTS {
            for layer in LAYERS.into_iter().take(1) {
                for face in FACES {
                    let r = rect(part, layer, face, Model::Steve);
                    let c = match face {
                        Face::Front => [220, 60, 60],
                        Face::Back => [60, 90, 220],
                        Face::Right => [60, 180, 80],
                        Face::Left => [230, 200, 60],
                        Face::Top => [240, 240, 240],
                        Face::Bottom => [120, 120, 120],
                    };
                    for (x, y) in r.pixels() {
                        let corner = x < r.x + 2 && y < r.y + 2;
                        let [r, g, b] = if corner { [0, 0, 0] } else { c };
                        s.pixels.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                    }
                }
            }
        }
        s
    }

    /// Dibuja sin ventana y guarda PNG en la carpeta de DEEPSLATE_RENDER_OUT.
    #[test]
    #[ignore = "necesita GPU; correr con --ignored"]
    fn render_to_png() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = block_on(instance.request_adapter(&Default::default())).expect("sin GPU");
        let (device, queue) = block_on(adapter.request_device(&Default::default())).unwrap();
        let renderer = egui_wgpu::Renderer::new(&device, FORMAT, Default::default());
        let rs = RenderState {
            adapter,
            available_adapters: Vec::new(),
            instance,
            device,
            queue,
            target_format: FORMAT,
            renderer: Arc::new(RwLock::new(renderer)),
        };
        let r = Renderer3d::new(&rs);
        r.upload_skin(&test_skin().pixels);
        // Capa: exterior roja con esquina negra, interior azul.
        let mut cape = RgbaImage::from_pixel(64, 32, image::Rgba([60, 90, 220, 255]));
        for (x, y, p) in cape.enumerate_pixels_mut() {
            if (1..11).contains(&x) && (1..17).contains(&y) {
                let corner = x < 3 && y < 3;
                *p = image::Rgba(if corner {
                    [0, 0, 0, 255]
                } else {
                    [220, 60, 60, 255]
                });
            }
        }
        r.upload_cape(&cape);
        let out = std::path::PathBuf::from(
            std::env::var("DEEPSLATE_RENDER_OUT").unwrap_or_else(|_| ".".into()),
        );
        let shots = [
            ("frente", Preset::Standing, Camera::default()),
            (
                "espalda",
                Preset::Waving,
                Camera {
                    yaw: 160.0,
                    ..Camera::default()
                },
            ),
            (
                "agachado",
                Preset::Crouching,
                Camera {
                    yaw: -70.0,
                    ..Camera::default()
                },
            ),
            ("miniatura", Preset::Running, Camera::thumbnail()),
        ];
        for (name, preset, camera) in shots {
            let pose = preset.pose();
            let scene = Scene {
                model: Model::Steve,
                pose: &pose,
                camera: &camera,
                selected: Some(Part::Head),
                visible: &|_, _| true,
                cape: true,
                lighting: 1.0,
            };
            let img = r.capture(384, &scene).expect("captura");
            assert!(img.pixels().any(|p| p[3] == 255), "no se dibujó nada");
            img.save(out.join(format!("{name}.png"))).unwrap();
        }
    }
}
