// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use io_kit_sys::types::{io_connect_t, io_iterator_t};
use io_kit_sys::*;
use mach::kern_return::*;
use mach::traps::mach_task_self;
use std::mem;
use sysinfo::{CpuExt, System, SystemExt};
use std::thread;
use std::time::{Duration, Instant};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use metal::{Device, MTLSize, CompileOptions};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde;
use tokio::task;

const KERNEL_INDEX_SMC: u32 = 2;
const SMC_CMD_READ_BYTES: u8 = 5;
const SMC_CMD_READ_KEYINFO: u8 = 9;

// CPU 温度键
const SMC_KEY_CPU_PROXIMITY: &str = "TC0P";  // CPU 温度

// 风扇相关键
const SMC_KEY_FAN_NUM: &str = "FNum";        // 风扇数量
const SMC_KEY_FAN_SPEED_CUR: [&str; 2] = [   // 当前转速
    "F0Ac",  // Fan 0 实际转速
    "F1Ac"   // Fan 1 实际转速
];

// 使用 Lazy 和 Mutex 定义静态变量
static STRESS_TEST_HANDLE: Lazy<Mutex<Option<Arc<AtomicBool>>>> = Lazy::new(|| Mutex::new(None));
static GPU_STRESS_TEST_HANDLE: Lazy<Mutex<Option<Arc<AtomicBool>>>> = Lazy::new(|| Mutex::new(None));

// 定义缓存结构
struct StatsCache {
    last_update: Instant,
    data: (i32, i32, i32),  // GPU 使用率、温度、风扇转速
}

// GPU 缓存
static GPU_STATS_CACHE: Lazy<Mutex<StatsCache>> = Lazy::new(|| {
    Mutex::new(StatsCache {
        last_update: Instant::now(),
        data: (0, 0, 0),
    })
});

// CPU 缓存结构
struct CpuStatsCache {
    last_update: Instant,
    thread_data: Vec<i32>,  // 线程使用率
    core_data: Vec<i32>,    // 核心使用率
    total_usage: i32,       // 总使用率
}

// CPU 缓存
static CPU_STATS_CACHE: Lazy<Mutex<CpuStatsCache>> = Lazy::new(|| {
    Mutex::new(CpuStatsCache {
        last_update: Instant::now(),
        thread_data: Vec::new(),
        core_data: Vec::new(),
        total_usage: 0,
    })
});

// 修改 get_cpu_usage 函数
#[tauri::command]
async fn get_cpu_usage() -> (Vec<i32>, Vec<i32>, i32) {
    let cache_duration = Duration::from_millis(500);
    
    // 检查缓存
    {
        let cache = CPU_STATS_CACHE.lock();
        if cache.last_update.elapsed() < cache_duration {
            return (
                cache.thread_data.clone(),
                cache.core_data.clone(),
                cache.total_usage
            );
        }
    }
    
    // 在单独的线程中获取 CPU 数据
    let (thread_usage, core_usage, total_usage) = tokio::task::spawn_blocking(|| {
        get_actual_cpu_usage()
    }).await.unwrap_or_default();
    
    // 更新缓存
    {
        let mut cache = CPU_STATS_CACHE.lock();
        cache.thread_data = thread_usage.clone();
        cache.core_data = core_usage.clone();
        cache.total_usage = total_usage;
        cache.last_update = Instant::now();
    }
    
    (thread_usage, core_usage, total_usage)
}

// 修改 get_cpu_temp 函数，添加缓存
struct CpuTempCache {
    last_update: Instant,
    data: f64,
}

static CPU_TEMP_CACHE: Lazy<Mutex<CpuTempCache>> = Lazy::new(|| {
    Mutex::new(CpuTempCache {
        last_update: Instant::now(),
        data: 0.0,
    })
});

#[tauri::command]
async fn get_cpu_temp() -> f64 {
    let cache_duration = Duration::from_millis(500);
    
    // 检查缓存
    {
        let cache = CPU_TEMP_CACHE.lock();
        if cache.last_update.elapsed() < cache_duration {
            return cache.data;
        }
    }
    
    // 在单独的线程中获取温度数据
    let temp = tokio::task::spawn_blocking(|| {
        let smc = SMC::new().unwrap();
        smc.read_key(SMC_KEY_CPU_PROXIMITY).unwrap_or(0.0)
    }).await.unwrap_or(0.0);
    
    // 更新缓存
    {
        let mut cache = CPU_TEMP_CACHE.lock();
        cache.data = temp;
        cache.last_update = Instant::now();
    }
    
    temp
}

// 修改风扇速度缓存
struct FanSpeedCache {
    last_update: Instant,
    data: Vec<(usize, f64)>,
}

static FAN_SPEED_CACHE: Lazy<Mutex<FanSpeedCache>> = Lazy::new(|| {
    Mutex::new(FanSpeedCache {
        last_update: Instant::now(),
        data: Vec::new(),
    })
});

#[tauri::command]
async fn get_all_fan_speeds() -> Result<Vec<(usize, f64)>, String> {
    let cache_duration = Duration::from_millis(500);
    
    // 检查缓存
    {
        let cache = FAN_SPEED_CACHE.lock();
        if cache.last_update.elapsed() < cache_duration {
            return Ok(cache.data.clone());
        }
    }
    
    // 在单独的线程中获取风扇数据
    let speeds = tokio::task::spawn_blocking(|| {
        let smc = SMC::new()?;
        smc.get_all_fan_speeds()
    }).await.unwrap_or(Ok(vec![])).unwrap_or_default();
    
    // 更新缓存
    {
        let mut cache = FAN_SPEED_CACHE.lock();
        cache.data = speeds.clone();
        cache.last_update = Instant::now();
    }
    
    Ok(speeds)
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct SMCKeyData {
    key: u32,
    vers: [u8; 6],
    p_limit_data: [u8; 16],
    key_info: SMCKeyInfoData,
    result: u8,
    status: u8,
    data8: u8,
    data32: u32,
    bytes: [u8; 32],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct SMCKeyInfoData {
    data_size: u32,
    data_type: u32,
    data_attributes: u8,
}

#[derive(Clone)]
struct SMC {
    connection: io_connect_t,
}

impl SMC {
    fn new() -> Result<Self, String> {
        unsafe {
            let mut iterator: io_iterator_t = 0;
            let matching = IOServiceMatching(b"AppleSMC\0".as_ptr() as *const i8);
            
            if matching.is_null() {
                return Err("Failed to create matching dictionary".to_string());
            }

            let result = IOServiceGetMatchingServices(
                kIOMasterPortDefault,
                matching,
                &mut iterator,
            );

            if result != KERN_SUCCESS {
                return Err(format!("Failed to get matching services: {}", result));
            }

            let device = IOIteratorNext(iterator);
            IOObjectRelease(iterator);

            if device == 0 {
                return Err("Failed to find SMC device".to_string());
            }

            let mut connection: io_connect_t = 0;
            let result = IOServiceOpen(device, mach_task_self(), 0, &mut connection);
            IOObjectRelease(device);

            if result != KERN_SUCCESS {
                return Err(format!("Failed to open SMC connection: {}", result));
            }

            Ok(SMC { connection })
        }
    }

    fn read_key(&self, key: &str) -> Result<f64, String> {
        if key.len() != 4 {
            return Err("Invalid key length".to_string());
        }

        let mut input: SMCKeyData = unsafe { mem::zeroed() };
        let mut output: SMCKeyData = unsafe { mem::zeroed() };

        let key_bytes = key.as_bytes();
        input.key = u32::from_be_bytes([
            key_bytes[0],
            key_bytes[1],
            key_bytes[2],
            key_bytes[3],
        ]);
        input.data8 = SMC_CMD_READ_KEYINFO;

        let mut output_size = mem::size_of::<SMCKeyData>();

        let result = unsafe {
            IOConnectCallStructMethod(
                self.connection,
                KERNEL_INDEX_SMC,
                &input as *const _ as *const _,
                mem::size_of::<SMCKeyData>(),
                &mut output as *mut _ as *mut _,
                &mut output_size,
            )
        };

        if result != KERN_SUCCESS {
            return Err(format!("Failed to read key info: {}", result));
        }

        input.key_info = output.key_info;
        input.data8 = SMC_CMD_READ_BYTES;

        let result = unsafe {
            IOConnectCallStructMethod(
                self.connection,
                KERNEL_INDEX_SMC,
                &input as *const _ as *const _,
                mem::size_of::<SMCKeyData>(),
                &mut output as *mut _ as *mut _,
                &mut output_size,
            )
        };

        if result != KERN_SUCCESS {
            return Err(format!("Failed to read data: {}", result));
        }

        // 根据键的类型进行不同的转换
        if key.starts_with('F') && key.ends_with('c') {  // 风扇转速
            let raw_val = (output.bytes[0] as u16) << 8 | output.bytes[1] as u16;
            Ok(raw_val as f64 / 4.0)  // 除以 4 得到实际转速
        } else {
            // 温度等其他值需要除以256
            let int_val = ((output.bytes[0] as i32) << 8 | output.bytes[1] as i32) as f64;
            Ok(int_val / 256.0)
        }
    }

    fn get_fan_number(&self) -> Result<u8, String> {
        match self.read_key(SMC_KEY_FAN_NUM) {
            Ok(val) => Ok(val as u8),
            Err(e) => Err(e),
        }
    }

    fn get_fan_speed(&self, fan_number: u8) -> Result<f64, String> {
        if fan_number >= SMC_KEY_FAN_SPEED_CUR.len() as u8 {
            return Err(format!("Invalid fan number: {}", fan_number));
        }

        self.read_key(SMC_KEY_FAN_SPEED_CUR[fan_number as usize])
    }

    fn get_all_fan_speeds(&self) -> Result<Vec<(usize, f64)>, String> {
        let fan_count = self.get_fan_number()?;
        let mut valid_speeds = Vec::new();
        
        for i in 0..fan_count {
            if i < SMC_KEY_FAN_SPEED_CUR.len() as u8 {
                match self.get_fan_speed(i) {
                    Ok(current) => {
                        if current > 0.0 {
                            valid_speeds.push((i as usize, current));
                        }
                    }
                    Err(_) => {} // 忽略错误
                }
            }
        }
        
        Ok(valid_speeds)
    }
}

impl Drop for SMC {
    fn drop(&mut self) {
        if self.connection != 0 {
            unsafe {
                IOServiceClose(self.connection);
            }
        }
    }
}

// 添加实际的 CPU 使用率获取函数
fn get_actual_cpu_usage() -> (Vec<i32>, Vec<i32>, i32) {
    let mut sys = System::new_all();
    let num_cores = get_cpu_cores();
    let num_threads = get_cpu_threads();
    
    // 第一次采样
    sys.refresh_cpu();
    let _initial_values: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
    
    // 增加采样间隔到1秒，使数据更稳定
    thread::sleep(Duration::from_millis(1000));
    
    // 第二次采样
    sys.refresh_cpu();
    
    // 计算线程使用率，并确保值在0-100之间
    let thread_usage: Vec<i32> = sys.cpus().iter().enumerate().map(|(_i, cpu)| {
        let current = cpu.cpu_usage();
        ((current).max(0.0).min(100.0)) as i32
    }).collect();

    // 计算每个物理核心的使用率
    let mut core_usage = Vec::with_capacity(num_cores);
    let mut thread_idx = 0;

    while thread_idx < num_threads {
        if thread_idx + 1 < num_threads && thread_idx / 2 < num_cores {
            // 对于超线程的核心，取两个线程的平均值
            let core_load = (thread_usage[thread_idx] + thread_usage[thread_idx + 1]) / 2;
            core_usage.push(core_load);
            thread_idx += 2;
        } else if thread_idx < num_threads && thread_idx < num_cores {
            // 对于单线程的核心，直接使用线程使用率
            core_usage.push(thread_usage[thread_idx]);
            thread_idx += 1;
        } else {
            break;
        }
    }

    // 计算总体使用率
    let total_usage = if core_usage.is_empty() {
        0
    } else {
        core_usage.iter().sum::<i32>() / core_usage.len() as i32
    };
    
    (thread_usage, core_usage, total_usage)
}

#[tauri::command]
async fn get_actual_gpu_stats() -> Result<(i32, i32, i32), String> {
    let cache_duration = Duration::from_millis(500);
    
    // 检查缓存
    {
        let cache = GPU_STATS_CACHE.lock();
        if cache.last_update.elapsed() < cache_duration {
            return Ok(cache.data);
        }
    } // 锁在这里释放

    #[cfg(target_os = "macos")]
    {
        // 使用 tokio 的异步命令执行
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg("ioreg -l |grep \"PerformanceStatistics\" | cut -d '{' -f 2 | tr '|' ',' | tr -d '}' | tr ',' '\n'|grep 'Temp\\|Fan\\|GPU Activity'")
            .output()
            .await
            .map_err(|e| e.to_string())?;

        let output_str = String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?;

        let mut gpu_usage = 0;
        let mut gpu_temp = 0;
        let mut fan_speed = 0;

        for line in output_str.lines() {
            let line = line.trim();
            if line.contains("GPU Activity") {
                if let Some(value) = line.split('=').nth(1) {
                    gpu_usage = value.trim().parse::<f64>().unwrap_or(0.0).round() as i32;
                }
            } else if line.contains("Temp") {
                if let Some(value) = line.split('=').nth(1) {
                    gpu_temp = value.trim().parse::<f64>().unwrap_or(0.0).round() as i32;
                }
            } else if line.contains("Fan") {
                if let Some(value) = line.split('=').nth(1) {
                    fan_speed = value.trim().parse::<f64>().unwrap_or(0.0).round() as i32;
                }
            }
        }

        // 更新缓存
        {
            let mut cache = GPU_STATS_CACHE.lock();
            cache.data = (gpu_usage, gpu_temp, fan_speed);
            cache.last_update = Instant::now();
        }

        Ok((gpu_usage, gpu_temp, fan_speed))
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok((0, 0, 0))
    }
}

#[tauri::command]
fn read_key(key: &str) -> Result<i32, String> {
    let smc = SMC::new()?;
    Ok(smc.read_key(key)?.round() as i32)
}

#[tauri::command]
async fn get_all_core_temps() -> Result<Vec<(usize, i32)>, String> {
    let num_cores = get_cpu_cores();
    
    // Get all temperatures in a single task
    let results = task::spawn_blocking(move || {
        let smc = SMC::new()?;
        let mut temps = Vec::new();
        
        for core in 0..num_cores {
            let key = format!("TC{}C", core);
            if let Ok(temp) = smc.read_key(&key) {
                temps.push((core, temp.round() as i32));
            }
        }
        
        Ok::<_, String>(temps)
    }).await.unwrap()?;
    
    Ok(results)
}

#[tauri::command]
fn get_cpu_cores() -> usize {
    System::new_all().physical_core_count().unwrap_or(1)
}

#[tauri::command]
fn get_cpu_threads() -> usize {
    System::new_all().cpus().len()
}

#[tauri::command]
async fn start_stress_test() -> Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    
    let mut handle = STRESS_TEST_HANDLE.lock();
    if handle.is_some() {
        return Err("Stress test is already running".to_string());
    }
    *handle = Some(running.clone());

    // 获取CPU核心数
    let num_cores = num_cpus::get();
    
    // 为每个核心创建一个压力测试线程
    for _ in 0..num_cores {
        let running = running.clone();
        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                // CPU 密集型计算，明确指定 f64 类型
                let mut x: f64 = 0.0;
                for _ in 0..1000000 {
                    x = x.sin().cos().tan();
                }
            }
        });
    }

    Ok(())
}

#[tauri::command]
fn stop_stress_test() {
    let mut handle = STRESS_TEST_HANDLE.lock();
    if let Some(running) = handle.take() {
        running.store(false, Ordering::SeqCst);
    }
}

#[tauri::command]
async fn start_gpu_stress_test() -> Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    
    let mut handle = GPU_STRESS_TEST_HANDLE.lock();
    if handle.is_some() {
        return Err("GPU stress test is already running".to_string());
    }
    *handle = Some(running.clone());

    std::thread::spawn(move || {
        let device = Device::system_default().expect("Failed to create Metal device");
        let command_queue = device.new_command_queue();
        let compile_options = CompileOptions::new();
        
        let library = device.new_library_with_source(
            METAL_SHADER_SOURCE, 
            &compile_options
        ).unwrap_or_else(|e| panic!("Failed to create shader library: {:?}", e));

        let kernel = library.get_function("gpu_stress", None)
            .expect("Failed to get kernel function");
        let pipeline = device.new_compute_pipeline_state_with_function(&kernel)
            .expect("Failed to create pipeline state");

        while running.load(Ordering::SeqCst) {
            // 增加每帧的计算次数
            for _ in 0..8 {  // 增加到8次
                let command_buffer = command_queue.new_command_buffer();
                let compute_encoder = command_buffer.new_compute_command_encoder();
                
                compute_encoder.set_compute_pipeline_state(&pipeline);
                
                // 增加计算网格大小
                let grid_size = MTLSize::new(8192, 8192, 1);  // 显著增加网格大小
                let thread_group_size = MTLSize::new(16, 16, 1);
                
                // 增加每个编码器的调度次数
                for _ in 0..6 {  // 增加到6次
                    compute_encoder.dispatch_threads(grid_size, thread_group_size);
                }
                
                compute_encoder.end_encoding();
                command_buffer.commit();
            }
            
            // 减少等待时间，增加 GPU 使用率
            thread::sleep(Duration::from_millis(4));  // 减少到4ms
        }
    });

    Ok(())
}

#[tauri::command]
fn stop_gpu_stress_test() {
    let mut handle = GPU_STRESS_TEST_HANDLE.lock();
    if let Some(running) = handle.take() {
        running.store(false, Ordering::SeqCst);
    }
}

// Metal shader 源代码
const METAL_SHADER_SOURCE: &str = r#"
#include <metal_stdlib>
using namespace metal;

kernel void gpu_stress(
    uint2 gid [[thread_position_in_grid]]
) {
    float4 result = float4(0.0);
    float4 temp = float4(gid.x, gid.y, 1.0, 1.0);
    float3 temp3 = float3(1.0, 1.0, 1.0);
    
    // 大幅增加循环次数
    for(int i = 0; i < 50000; i++) {  // 增加到50000次
        // 复杂的数学运算
        temp = sin(temp) * 0.5 + cos(temp) * 0.5;
        result += temp;
        
        // 密集的数学运算
        temp = pow(temp, 2.0) + float4(0.1);
        temp = fmod(temp * 1.5, 3.14159);
        temp = log(abs(temp) + 1.0);
        temp = exp(temp * 0.5);
        
        // 条件分支和向量运算
        if(length(temp) > 2.0) {
            temp = normalize(temp);
            temp3 = cross(temp3, float3(0.5, 0.7, 0.3));
            temp.xyz = temp3;
            temp = sqrt(abs(temp)) + 0.5;
        } else {
            temp = mix(temp, float4(1.0), 0.5);
            temp = reflect(temp, normalize(float4(1.0)));
            temp = floor(temp * 3.0) / 3.0;
        }
        
        // 密集的矩阵运算
        float4x4 matrix = float4x4(
            cos(temp.x), sin(temp.y), -sin(temp.z), cos(temp.w),
            sin(temp.x), cos(temp.y), cos(temp.z), -sin(temp.w),
            -sin(temp.x), cos(temp.y), cos(temp.z), sin(temp.w),
            cos(temp.x), -sin(temp.y), sin(temp.z), cos(temp.w)
        );
        temp = matrix * temp;
        
        // 额外的数学运算
        temp = smoothstep(float4(-1.0), float4(1.0), temp);
        temp = fract(temp * 1.5) * 2.0 - 1.0;
        temp = atan2(temp + 0.1, float4(1.0));
        
        // 更多的条件分支
        if(any(temp > 0.5)) {
            temp = pow(temp, 3.0);
        }
        
        result += temp;
    }
    
    // 防止编译器优化
    threadgroup_barrier(mem_flags::mem_device);
}
"#;

// 添加新的结构体来存储GPU信息
#[derive(serde::Serialize)]
pub struct GpuInfo {
    vendor: String,
    model: String,
}

#[tauri::command]
fn get_gpu_info() -> Result<GpuInfo, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .output()
            .map_err(|e| e.to_string())?;

        let output_str = String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?;

        let mut model = String::new();

        // 只获取 Chipset Model
        for line in output_str.lines() {
            let line = line.trim();
            if line.contains("Chipset Model:") {
                model = line.replace("Chipset Model:", "").trim().to_string();
                break;
            }
        }

        // 如果没有找到型号，使用默认值
        if model.is_empty() {
            model = "Unknown GPU".to_string();
        }

        // 从型号判断 vendor，并从型号中移除 vendor 名称
        let vendor = if model.contains("AMD") {
            model = model.replace("AMD", "").trim().to_string();
            "AMD".to_string()
        } else if model.contains("NVIDIA") {
            model = model.replace("NVIDIA", "").trim().to_string();
            "NVIDIA".to_string()
        } else if model.contains("Intel") {
            model = model.replace("Intel", "").trim().to_string();
            "Intel".to_string()
        } else if model.contains("Apple") {
            model = model.replace("Apple", "").trim().to_string();
            "Apple".to_string()
        } else {
            "Unknown".to_string()
        };

        Ok(GpuInfo { vendor, model })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(GpuInfo {
            vendor: "Unknown".to_string(),
            model: "GPU".to_string(),
        })
    }
}

// 添加新的结构体来存储CPU信息
#[derive(serde::Serialize)]
pub struct CpuInfo {
    vendor: String,
    model: String,
    cores: usize,
    threads: usize,
}

#[tauri::command]
fn get_cpu_info() -> Result<CpuInfo, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
            .map_err(|e| e.to_string())?;

        let brand_string = String::from_utf8(output.stdout)
            .map_err(|e| e.to_string())?
            .trim()
            .to_string();

        let cores = get_cpu_cores();
        let threads = get_cpu_threads();

        // 提取 vendor
        let vendor = if brand_string.contains("Intel") {
            "Intel".to_string()
        } else if brand_string.contains("AMD") {
            "AMD".to_string()
        } else if brand_string.contains("Apple") {
            "Apple".to_string()
        } else {
            "Unknown".to_string()
        };

        // 提取型号，处理不同类型的处理器
        let model = if brand_string.contains("Intel") {
            // 处理 Intel 处理器
            let parts: Vec<&str> = brand_string.split(' ').collect();
            let mut model_parts = Vec::new();
            let mut found_model = false;

            for part in parts {
                // 查找处理器型号的起始位置（i3/i5/i7/i9、Xeon 等）
                if part.starts_with('i') || part == "Xeon" || part == "Celeron" || part == "Pentium" {
                    found_model = true;
                }
                
                // 收集型号信息直到遇到 @、CPU 等终止词
                if found_model {
                    if part.contains('@') || part == "CPU" {
                        break;
                    }
                    model_parts.push(part);
                }
            }

            if model_parts.is_empty() {
                "Unknown".to_string()
            } else {
                model_parts.join(" ")
            }
        } else if brand_string.contains("AMD") {
            // 处理 AMD 处理器
            let parts: Vec<&str> = brand_string.split(' ').collect();
            let mut model_parts = Vec::new();
            let mut found_model = false;

            for part in parts {
                // 查找处理器型号的起始位置（Ryzen、EPYC 等）
                if part == "Ryzen" || part == "EPYC" || part == "Athlon" {
                    found_model = true;
                }
                
                // 收集型号信息直到遇到 @、CPU 等终止词
                if found_model {
                    if part.contains('@') || part == "CPU" {
                        break;
                    }
                    model_parts.push(part);
                }
            }

            if model_parts.is_empty() {
                "Unknown".to_string()
            } else {
                model_parts.join(" ")
            }
        } else if brand_string.contains("Apple") {
            // 处理 Apple Silicon
            if let Some(m_pos) = brand_string.find('M') {
                let model_str = &brand_string[m_pos..];
                if let Some(end) = model_str.find(' ') {
                    model_str[..end].to_string()
                } else {
                    model_str.to_string()
                }
            } else {
                "Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };

        Ok(CpuInfo {
            vendor,
            model,
            cores,
            threads,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(CpuInfo {
            vendor: "Unknown".to_string(),
            model: "CPU".to_string(),
            cores: get_cpu_cores(),
            threads: get_cpu_threads(),
        })
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_cpu_usage,
            read_key,
            get_all_core_temps,
            get_all_fan_speeds,
            get_actual_gpu_stats,
            get_cpu_cores,
            get_cpu_threads,
            get_cpu_temp,
            get_gpu_info,
            start_stress_test,
            stop_stress_test,
            start_gpu_stress_test,
            stop_gpu_stress_test,
            get_gpu_info,
            get_cpu_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
