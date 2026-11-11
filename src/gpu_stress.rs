//! GPU stress test: runs a heavy compute shader in a loop through wgpu
//! (Vulkan). Works on NVIDIA, AMD and Intel GPUs.
//!
//! Each GPU job is kept short (well under a second) so the driver's watchdog
//! never resets the GPU, and so the stop button reacts quickly.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Number of shader invocations per dispatch (each works on one number).
const INVOCATIONS: u64 = 1 << 20;
/// Invocations per workgroup; must match `@workgroup_size` in the shader.
const WORKGROUP_SIZE: u64 = 256;
/// Compute passes sent to the GPU in one batch.
const PASSES_PER_BATCH: usize = 4;

/// Shader written in WGSL. Every invocation does a long chain of math.
const SHADER: &str = r#"
@group(0) @binding(0) var<storage, read_write> data: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if (index >= arrayLength(&data)) {
        return;
    }

    var value = data[index] + f32(index) * 0.000001;
    for (var step = 0u; step < 2048u; step = step + 1u) {
        value = fma(value, 1.0000001, 0.0000001);
        value = sin(value) * 0.5 + value * 0.5;
    }
    data[index] = value;
}
"#;

pub struct GpuStress {
    stop_flag: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    batches: Arc<AtomicU64>,
    adapter_name: String,
}

impl GpuStress {
    /// Starts the test. Waits briefly while the GPU is being set up and
    /// returns an error message if no usable GPU is found.
    pub fn start() -> Result<Self, String> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let batches = Arc::new(AtomicU64::new(0));
        let (ready_sender, ready_receiver) = mpsc::channel::<Result<String, String>>();

        let worker = {
            let stop = Arc::clone(&stop_flag);
            let batches = Arc::clone(&batches);
            thread::spawn(move || run_gpu_loop(&stop, &batches, ready_sender))
        };

        match ready_receiver.recv_timeout(Duration::from_secs(15)) {
            Ok(Ok(adapter_name)) => {
                Ok(Self { stop_flag, worker: Some(worker), batches, adapter_name })
            }
            Ok(Err(message)) => {
                let _ = worker.join();
                Err(message)
            }
            Err(_) => {
                stop_flag.store(true, Ordering::Relaxed);
                Err("GPU setup timed out".to_string())
            }
        }
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    /// Number of finished batches (a rough sign that the GPU is working).
    pub fn batches(&self) -> u64 {
        self.batches.load(Ordering::Relaxed)
    }

    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Everything needed to keep sending work to the GPU.
struct GpuSetup {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    adapter_name: String,
}

fn run_gpu_loop(
    stop: &AtomicBool,
    batches: &AtomicU64,
    ready: mpsc::Sender<Result<String, String>>,
) {
    let setup = match pollster::block_on(setup_gpu()) {
        Ok(setup) => setup,
        Err(message) => {
            let _ = ready.send(Err(message));
            return;
        }
    };
    let _ = ready.send(Ok(setup.adapter_name.clone()));

    let workgroups = (INVOCATIONS / WORKGROUP_SIZE) as u32;

    while !stop.load(Ordering::Relaxed) {
        let mut encoder = setup
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("stress") });

        for _ in 0..PASSES_PER_BATCH {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&setup.pipeline);
            pass.set_bind_group(0, &setup.bind_group, &[]);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        setup.queue.submit(Some(encoder.finish()));
        // Wait until the GPU finished, so the stop flag is checked once per batch.
        let _ = setup.device.poll(wgpu::Maintain::Wait);
        batches.fetch_add(1, Ordering::Relaxed);
    }
}

async fn setup_gpu() -> Result<GpuSetup, String> {
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .ok_or("No GPU found. Is a Vulkan driver installed? (try: vulkaninfo)")?;

    let info = adapter.get_info();
    if info.device_type == wgpu::DeviceType::Cpu {
        return Err(format!("Only a software renderer was found ({})", info.name));
    }

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .map_err(|error| format!("Cannot open the GPU: {error}"))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("stress-shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("stress-pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    // One 32-bit number per invocation (4 MB in total).
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("stress-data"),
        size: INVOCATIONS * 4,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("stress-bind-group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
    });

    Ok(GpuSetup { device, queue, pipeline, bind_group, adapter_name: info.name })
}
