use clap::Parser;
use xcqa_chain_pow::{CpuSolver, GpuSolver};

#[derive(Parser)]
#[command(name = "xcqa-node")]
#[command(about = "XCQA Chain Node", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "8333")]
    port: u16,

    #[arg(long)]
    gpu: bool,

    #[arg(long, default_value = "2048")]
    gpu_vram_mb: usize,

    #[arg(long, default_value = "8")]
    layers: usize,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("XCQA Chain Node v0.1.0");
    println!("Port: {}", cli.port);
    println!("Layers: {}", cli.layers);

    if cli.gpu {
        match GpuSolver::new(cli.layers, cli.gpu_vram_mb) {
            Ok(_solver) => println!("GPU solver initialized ({}MB VRAM required)", cli.gpu_vram_mb),
            Err(e) => println!("GPU solver failed: {}, falling back to CPU", e),
        }
    } else {
        let _solver = CpuSolver::new(cli.layers);
        println!("CPU solver initialized");
    }

    println!("Node ready");
}
