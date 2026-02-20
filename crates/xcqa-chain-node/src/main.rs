use clap::Parser;
use xcqa_chain_core::{Blockchain, Block, BlockHeader};
use xcqa_chain_pow::{CpuSolver, GpuSolver, check_difficulty};
use std::time::{SystemTime, UNIX_EPOCH};

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

    #[arg(long)]
    mine: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("XCQA Chain Node v0.1.0");
    println!("Port: {}", cli.port);
    println!("Layers: {}", cli.layers);

    let mut blockchain = Blockchain::new(Blockchain::genesis());
    println!("Genesis block created at height {}", blockchain.height());

    if cli.mine {
        println!("Starting mining...");

        if cli.gpu {
            match GpuSolver::new(cli.layers, cli.gpu_vram_mb) {
                Ok(solver) => {
                    println!("GPU solver initialized ({}MB VRAM)", cli.gpu_vram_mb);
                    mine_loop(&mut blockchain, solver).await;
                }
                Err(e) => {
                    println!("GPU failed: {}, using CPU", e);
                    let solver = CpuSolver::new(cli.layers);
                    mine_loop(&mut blockchain, solver).await;
                }
            }
        } else {
            let solver = CpuSolver::new(cli.layers);
            println!("CPU solver initialized");
            mine_loop(&mut blockchain, solver).await;
        }
    } else {
        println!("Node ready (not mining)");
    }
}

async fn mine_loop<S: Solver>(blockchain: &mut Blockchain, solver: S) {
    loop {
        let latest = blockchain.latest_block();
        let block_hash = latest.hash();

        let header = BlockHeader {
            height: blockchain.height() + 1,
            prev_hash: block_hash,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            difficulty_tier: 0,
            fine_difficulty: 1,
        };

        println!("Mining block {}...", header.height);

        let header_bytes = {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&header.height.to_le_bytes());
            bytes.extend_from_slice(&header.prev_hash);
            bytes.extend_from_slice(&header.timestamp.to_le_bytes());
            bytes.push(header.difficulty_tier);
            bytes.push(header.fine_difficulty);
            bytes
        };

        match solver.mine(&header_bytes, &block_hash, header.fine_difficulty) {
            Ok((sig, nonce)) => {
                let block = Block {
                    header,
                    transactions: vec![],
                    xcqa_sig: sig,
                    xcqa_nonce: nonce,
                };

                blockchain.add_block(block).unwrap();
                println!("Block {} mined!", blockchain.height());
            }
            Err(e) => {
                println!("Mining error: {}", e);
                break;
            }
        }
    }
}

trait Solver {
    fn mine(&self, header: &[u8], block_hash: &[u8; 64], difficulty: u8)
        -> Result<(xcqa_crypto::XcqaSignature, [u8; 32]), xcqa_chain_pow::PowError>;
}

impl Solver for CpuSolver {
    fn mine(&self, header: &[u8], block_hash: &[u8; 64], difficulty: u8)
        -> Result<(xcqa_crypto::XcqaSignature, [u8; 32]), xcqa_chain_pow::PowError> {
        CpuSolver::mine(self, header, block_hash, difficulty)
    }
}

impl Solver for GpuSolver {
    fn mine(&self, header: &[u8], block_hash: &[u8; 64], difficulty: u8)
        -> Result<(xcqa_crypto::XcqaSignature, [u8; 32]), xcqa_chain_pow::PowError> {
        GpuSolver::mine(self, header, block_hash, difficulty)
    }
}
