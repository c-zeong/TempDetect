use metal::{Device, MTLResourceOptions, MTLSize, CommandBuffer};
use std::thread;
use std::time::Duration;

fn main() {
    let device = Device::system_default().expect("No Metal device found");
    println!("Using device: {}", device.name());

    let command_queue = device.new_command_queue();

    // 优化计算内核，减少系统压力
    let source = r#"
        #include <metal_stdlib>
        using namespace metal;

        kernel void heavy_compute(
            device float4 *buffer [[ buffer(0) ]],
            uint id [[ thread_position_in_grid ]]
        ) {
            // 使用 float4 向量化计算
            float4 result = float4(0.0);
            float4 base = float4(id * 4, id * 4 + 1, id * 4 + 2, id * 4 + 3);
            
            // 平衡计算量和系统负载
            for (int i = 0; i < 5000; i++) {  // 减少循环次数
                // 使用更简单的计算
                result = fma(result, result, sin(base + i));
                
                // 每隔一定次数进行复杂计算
                if (i % 10 == 0) {  // 降低复杂计算的频率
                    result = sqrt(abs(result));
                }
            }
            
            buffer[id] = result;
        }
    "#;

    let library = device
        .new_library_with_source(source, &metal::CompileOptions::new())
        .expect("Failed to create library");
    let kernel = library
        .get_function("heavy_compute", None)
        .expect("Failed to get kernel function");
    let pipeline = device
        .new_compute_pipeline_state_with_function(&kernel)
        .expect("Failed to create pipeline state");

    // 使用适中的缓冲区大小
    let buffer_length = 2048 * 512;  // 减小缓冲区
    let buffer = device.new_buffer(
        (buffer_length * std::mem::size_of::<f32>() * 4) as u64,
        MTLResourceOptions::StorageModeShared,
    );

    println!("Starting GPU load test...");
    println!("Press Ctrl+C to stop");

    // 减少并行任务数量
    let mut command_buffers = Vec::new();
    let max_buffers_in_flight = 3;  // 减少并行任务数量

    loop {
        // 清理已完成的命令缓冲区
        command_buffers.retain(|buf: &CommandBuffer| {
            let status = buf.status();
            status != metal::MTLCommandBufferStatus::Completed && 
            status != metal::MTLCommandBufferStatus::Error
        });

        // 控制任务创建速度
        if command_buffers.len() < max_buffers_in_flight {
            let command_buffer = command_queue.new_command_buffer().to_owned();
            let compute_encoder = command_buffer.new_compute_command_encoder();

            compute_encoder.set_compute_pipeline_state(&pipeline);
            compute_encoder.set_buffer(0, Some(&buffer), 0);

            // 使用适中的线程数量
            let thread_group_size = MTLSize::new(256, 1, 1);  // 减少线程组大小
            let thread_groups = MTLSize::new(buffer_length as u64 / 256, 1, 1);
            compute_encoder.dispatch_thread_groups(thread_groups, thread_group_size);

            compute_encoder.end_encoding();
            command_buffer.commit();

            command_buffers.push(command_buffer);
        }

        // 增加休眠时间，减轻系统压力
        thread::sleep(Duration::from_millis(16));  // 约60fps的刷新率
    }
}
