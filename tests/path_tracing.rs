use blst::min_sig::PublicKey;
use flate2::write::GzEncoder;
use flate2::Compression;
use pog::blockchain::block::{Block, Body, Header};
use pog::blockchain::path::{
    concat_tx_hash_with_to_hash_static, AggregatedSignedPaths, Path, TransactionPaths,
};
use pog::blockchain::transaction::Transaction;
use pog::wallet;
use pog::wallet::Wallet;
use std::io::Write;
use std::time::Instant;
use std::{io, path};

#[tokio::test]
async fn test_path_tracing_size_bls() {
    let mut path_size_list = vec![];
    let mut block_size_list = vec![];
    let mut block_compress_size_list = vec![];
    for n in 0..50 {
        let (path_size, block_size, compress_size) = bls_block_size(n);
        path_size_list.push(path_size);
        block_size_list.push(block_size);
        block_compress_size_list.push(compress_size);
    }
    println!("path_size_list: {:?}", path_size_list);
    println!("block_size_list: {:?}", block_size_list);
    println!("block_compress_size_list: {:?}", block_compress_size_list);
}

#[tokio::test]
async fn test_path_tracing_verify_bls() {
    let mut time_list = vec![];
    for n in 0..50 {
        let t = bls_verify(n);
        time_list.push(t);
    }
    println!("bls_verify_time_list: {:?}", time_list);
}

#[tokio::test]
async fn test_path_tracing_verify_bls_with_decompress() {
    let mut time_list = vec![];
    for n in 0..50 {
        let t = bls_verify_with_decompress(n);
        time_list.push(t);
    }
    println!("bls_verify_with_decompress_time_list: {:?}", time_list);
}

#[tokio::test]
async fn test_path_tracing_size_secp256k1() {
    let mut path_size_list = vec![];
    let mut block_size_list = vec![];
    for n in 0..50 {
        let (path_size, block_size) = secp256k1_block_size(n);
        path_size_list.push(path_size);
        block_size_list.push(block_size);
    }
    println!("path_size_list: {:?}", path_size_list);
    println!("block_size_list: {:?}", block_size_list);
}
#[tokio::test]
async fn test_path_tracing_verify_secp256k1() {
    let mut time_list = vec![];
    for n in 0..50 {
        let t = secp256k1_verify(n);
        time_list.push(t);
    }
    println!("secp256k1_verify_time_list: {:?}", time_list);
}

fn bls_block_size(n: u64) -> (u64, u64, u64) {
    let block_0 = Block::gen_genesis_block();

    let from = Wallet::new();
    let to = if n == 0 { from.clone() } else { Wallet::new() };
    let tx = Transaction::new(to.address.clone(), 0, from.clone());
    let mut tx_paths = TransactionPaths::new(tx.clone());
    let mut nodes = vec![];
    for i in 0..n {
        let node = Wallet::new();
        nodes.push(node);
    }
    if n > 0 {
        nodes.insert(0, from.clone());
    }
    nodes.push(to.clone());
    for i in 1..nodes.len() {
        tx_paths.add_path(nodes[i].address.clone(), nodes[i - 1].clone());
    }
    let aggregated = tx_paths.to_aggregated_signed_paths();

    let body = Body::new(vec![tx.clone()], vec![aggregated.clone()]);
    let block = Block::new(1, 0, 1, block_0.header.hash, body, to.clone()).unwrap();
    println!("tx size: {} B", tx.bytes());
    println!(
        "path with bls[{}]: {} B",
        aggregated.paths.len(),
        aggregated.bytes()
    );
    println!("compress path size: {} B", aggregated.compress().len());
    println!("block body size: {} B", block.body.bytes());
    println!("block body header: {} B", block.header.bytes());
    println!("block size: {} B", block.bytes());
    let txs: u64 = block.body.transactions.iter().map(|x| x.bytes()).sum();
    let block_with_compress = block.header.bytes() + txs + aggregated.compress().len() as u64;
    println!("block size with compress : {} B", block_with_compress);
    (aggregated.bytes(), block.bytes(), block_with_compress)
}

#[tokio::test]
async fn test_compress2() {
    let n = 5;
    let block_0 = Block::gen_genesis_block();

    let from = Wallet::new();
    let to = if n == 0 { from.clone() } else { Wallet::new() };
    let tx = Transaction::new(to.address.clone(), 0, from.clone());
    let mut tx_paths = TransactionPaths::new(tx.clone());
    let mut nodes = vec![];
    for i in 0..n {
        let node = Wallet::new();
        nodes.push(node);
    }
    if n > 0 {
        nodes.insert(0, from.clone());
    }
    nodes.push(to.clone());
    for i in 1..nodes.len() {
        tx_paths.add_path(nodes[i].address.clone(), nodes[i - 1].clone());
    }
    let aggregated = tx_paths.to_aggregated_signed_paths();
    let path_list: Vec<String> = aggregated
        .paths
        .iter()
        .map(|x| x.strip_prefix("0x").unwrap().to_string())
        .collect();
    let path_list_bytes: Vec<u8> = path_list
        .iter()
        .flat_map(|x| hex::decode(x.as_bytes()).unwrap())
        .collect();
    let signature_lens = aggregated.signature.as_bytes().len();

    // let mut compressed = Vec::new();
    // let mut encoder = zstd::stream::Encoder::<&mut Vec<u8>>::new(compressed.as_mut(), 22).unwrap();
    // io::copy(&mut s.as_bytes(), &mut encoder).unwrap();
    // encoder.finish().unwrap();
    // lzma_rs::lzma2_compress(&mut path_list_bytes.as_slice(), &mut compressed).unwrap();
    // io::copy(&mut path_list_bytes.as_slice(), &mut compressed).unwrap();
    let compressed =
        zstd::stream::encode_all(path_list_bytes.as_slice(), 3).expect("Compression failed");
    // let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    // encoder
    //     .write_all(&path_list_bytes.as_slice())
    //     .expect("Compression failed");
    // let compressed = encoder.finish().expect("Failed to finish compression");
    println!("aggregated size: {} B", aggregated.bytes());
    println!("compressed size: {} B", compressed.len());
    println!(
        "Compressed size: {} (compression ratio: {:.2}%)",
        compressed.len() + signature_lens,
        ((compressed.len() + signature_lens) as f32 / aggregated.bytes() as f32) * 100.0
    );
    // // ------ 解压数据 ------
    // let mut decompressed = Vec::new();
    // lzma_rs::lzma_decompress(&mut compressed.as_slice(), &mut decompressed).unwrap();
    // assert_eq!(input_data, decompressed.as_slice());
    // println!("Decompressed successfully!");
}

#[tokio::test]
async fn test_compress_json() {
    let n = 50;
    let block_0 = Block::gen_genesis_block();

    let from = Wallet::new();
    let to = if n == 0 { from.clone() } else { Wallet::new() };
    let tx = Transaction::new(to.address.clone(), 0, from.clone());
    let mut tx_paths = TransactionPaths::new(tx.clone());
    let mut nodes = vec![];
    for i in 0..n {
        let node = Wallet::new();
        nodes.push(node);
    }
    if n > 0 {
        nodes.insert(0, from.clone());
    }
    nodes.push(to.clone());
    for i in 1..nodes.len() {
        tx_paths.add_path(nodes[i].address.clone(), nodes[i - 1].clone());
    }
    let aggregated = tx_paths.to_aggregated_signed_paths();

    let compressed = aggregated.compress();

    let decompressed = AggregatedSignedPaths::decompress(compressed);
    for (i, x) in aggregated.paths.iter().enumerate() {
        assert_eq!(x.to_string(), decompressed.paths[i].to_string());
    }
    assert_eq!(aggregated.signature, decompressed.signature);
}

fn bls_verify(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let from = Wallet::new();
    let to = if n == 0 { from.clone() } else { Wallet::new() };
    let tx = Transaction::new(to.address.clone(), 0, from.clone());
    let mut tx_paths = TransactionPaths::new(tx.clone());
    let mut nodes = vec![];
    for i in 0..n {
        let node = Wallet::new();
        nodes.push(node);
    }
    if n > 0 {
        nodes.insert(0, from.clone());
    }
    nodes.push(to.clone());
    for i in 1..nodes.len() {
        tx_paths.add_path(nodes[i].address.clone(), nodes[i - 1].clone());
    }
    let aggregated = tx_paths.to_aggregated_signed_paths();

    let mut time = 0;

    {
        //聚合签名验证
        //先还原message
        let mut messages: Vec<Vec<u8>> = vec![];
        for (i, p) in aggregated.paths.iter().enumerate() {
            //发起者是对下一个节点进行的签名
            if i == 0 {
                continue;
            }
            let hash = concat_tx_hash_with_to_hash_static(tx.hash.clone(), p.clone());
            messages.push(hash.to_vec());
        }

        //再去找公钥
        let mut pks: Vec<PublicKey> = aggregated
            .paths
            .iter()
            .map(|p| wallet::get_bls_pub_key(p.clone()).unwrap())
            .collect();
        //miner并没有传播交易，所以去掉
        pks.remove(pks.len() - 1);
        let start = Instant::now();
        Wallet::bls_aggregated_verify(messages, pks, aggregated.signature.clone());
        let duration = start.elapsed();
        time = time + duration.as_micros();
    }
    // aggregated.verify(tx.clone(), to.address.clone());

    println!("{}个bls签名验证时间: {} 微秒", aggregated.paths.len(), time);
    time as u64
}

fn bls_verify_with_decompress(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let from = Wallet::new();
    let to = if n == 0 { from.clone() } else { Wallet::new() };
    let tx = Transaction::new(to.address.clone(), 0, from.clone());
    let mut tx_paths = TransactionPaths::new(tx.clone());
    let mut nodes = vec![];
    for i in 0..n {
        let node = Wallet::new();
        nodes.push(node);
    }
    if n > 0 {
        nodes.insert(0, from.clone());
    }
    nodes.push(to.clone());
    for i in 1..nodes.len() {
        tx_paths.add_path(nodes[i].address.clone(), nodes[i - 1].clone());
    }
    let aggregated = tx_paths.to_aggregated_signed_paths();

    let mut time = 0;

    {
        //聚合签名验证
        //先还原message
        let mut messages: Vec<Vec<u8>> = vec![];
        for (i, p) in aggregated.paths.iter().enumerate() {
            //发起者是对下一个节点进行的签名
            if i == 0 {
                continue;
            }
            let hash = concat_tx_hash_with_to_hash_static(tx.hash.clone(), p.clone());
            messages.push(hash.to_vec());
        }

        //再去找公钥
        let mut pks: Vec<PublicKey> = aggregated
            .paths
            .iter()
            .map(|p| wallet::get_bls_pub_key(p.clone()).unwrap())
            .collect();
        //miner并没有传播交易，所以去掉
        pks.remove(pks.len() - 1);
        let start = Instant::now();
        Wallet::bls_aggregated_verify(messages, pks, aggregated.signature.clone());
        let duration = start.elapsed();
        time = time + duration.as_micros();
    }
    // aggregated.verify(tx.clone(), to.address.clone());
    let data = aggregated.compress();
    let start = Instant::now();
    let decompressed = AggregatedSignedPaths::decompress(data);
    let duration = start.elapsed();
    time = time + duration.as_micros();
    println!("{}个bls签名验证时间: {} 微秒", aggregated.paths.len(), time);
    time as u64
}

fn secp256k1_block_size(n: u64) -> (u64, u64) {
    let block_0 = Block::gen_genesis_block();

    let from = Wallet::new();
    let to = if n == 0 { from.clone() } else { Wallet::new() };
    let tx = Transaction::new(to.address.clone(), 0, from.clone());
    let mut tx_paths = TransactionPaths::new(tx.clone());
    let mut nodes = vec![];
    for i in 0..n {
        let node = Wallet::new();
        nodes.push(node);
    }
    if n > 0 {
        nodes.insert(0, from.clone());
    }
    nodes.push(to.clone());
    for i in 1..nodes.len() {
        // data-> H(tx) || H(to)
        let hash = concat_tx_hash_with_to_hash_static(tx.hash.clone(), nodes[i].address.clone());
        let sign = nodes[i - 1].sign(hash);
        tx_paths.paths.push(Path {
            to: nodes[i].address.clone(),
            signature: sign.clone(),
        });
    }
    // no aggregate
    // let aggregated = tx_paths.to_aggregated_signed_paths();
    let mut path_trace_size = 0;
    tx_paths.paths.iter().for_each(|path| {
        path_trace_size = path_trace_size + path.to.as_bytes().len();
        path_trace_size = path_trace_size + path.signature.as_bytes().len();
    });

    let merkle_root = Block::cal_merkle_root(vec![tx.hash.clone()]);
    let header = Header::new(
        1,
        0,
        1,
        merkle_root,
        to.address.clone(),
        block_0.header.hash,
    );
    println!("block body header: {} B", header.bytes());
    let body_size = path_trace_size as u64 + tx.bytes();
    println!("block size: {} B", header.bytes() + body_size);
    (path_trace_size as u64, header.bytes() + body_size)
}

fn secp256k1_verify(n: u64) -> u64 {
    let block_0 = Block::gen_genesis_block();

    let from = Wallet::new();
    let to = if n == 0 { from.clone() } else { Wallet::new() };
    let tx = Transaction::new(to.address.clone(), 0, from.clone());
    let mut tx_paths = TransactionPaths::new(tx.clone());
    let mut nodes = vec![];
    for i in 0..n {
        let node = Wallet::new();
        nodes.push(node);
    }
    if n > 0 {
        nodes.insert(0, from.clone());
    }
    nodes.push(to.clone());
    for i in 1..nodes.len() {
        // data-> H(tx) || H(to)
        let hash = concat_tx_hash_with_to_hash_static(tx.hash.clone(), nodes[i].address.clone());
        let sign = nodes[i - 1].sign(hash);
        tx_paths.paths.push(Path {
            to: nodes[i].address.clone(),
            signature: sign.clone(),
        });
    }
    let mut time = 0;
    for i in 1..tx_paths.paths.len() {
        let path = &tx_paths.paths[i];
        let hash = concat_tx_hash_with_to_hash_static(tx.hash.clone(), nodes[i].address.clone());
        let start = Instant::now();
        nodes[i - 1].verify(hash, path.signature.clone());
        let duration = start.elapsed();
        time = time + duration.as_micros();
    }

    println!(
        "{}个secp256k1签名验证时间: {} 微秒",
        tx_paths.paths.len(),
        time
    );
    time as u64
}

#[tokio::test]
async fn test_compresse() {
    let mut input_data: [&str; 52] = [
        "0x1ce7b039c2866587e59ac747799960c56b59625c",
        "0xbc6113df9d9fd7949014942826ff18e275b14e79",
        "0xdf1de705c86fbc531e156c918ad3340a93c5982f",
        "0x3bdca8bef0960f0de46797c9c473d51539986d30",
        "0x253f108cf961765b48ca9f7478810f30d8247e73",
        "0xcf6af967562c8dec50c0b988ec75bc8ddb220678",
        "0xdba171dc21bb691845d4862366fe3922eee9d84c",
        "0x07c7169ba5b5ccd276d942a18317a96e86592240",
        "0x72883a9f152b8fc8af922ad48349b2cd8e7ea63d",
        "0x634365b9cd5013caf95d4b0f6c24aad442c2f1d0",
        "0x220cef4c7ed331382f8e8afbb3bd9540853fc6b6",
        "0x2ec3b2d2f59ebd545dfdf2adabb88aa357e70ad3",
        "0xe44983a2c1da287c14094cf84f9ef8fb89ead8c6",
        "0x9c6454256c364f092d12840f1e9bb19b50071c8a",
        "0xd24e79896449fe51f189bd13aa61acb5e5bd8503",
        "0x5e5441a0e4d5e66edf4fe013a45acef7d182500d",
        "0xdbf446a503da09887e8880160939f850f0a92279",
        "0xe63715a7f7c23093a2fd49a09b876c3414d4f6ab",
        "0xf3a8486c120a9daae36321f3cba3576ae4b3e243",
        "0xa7ed39ffb8185f2b4d980b242dc78d29b31de268",
        "0x08df68754dc84312f7d00584b11b46515cac44f4",
        "0x641dcaac117fd2b9041910a080349962bc41bde4",
        "0x52c529c59d418139898293c1f85b33bb986c667f",
        "0xf1f86a68b53bcaf54626e174a0961fa6ba7144cf",
        "0x2e78d55f98019ef9f25f7b5028240e51686a1336",
        "0x0327dd04899c3fca0854e66455a6e44834e19a71",
        "0x52476c94915d456759b08082ecf612ec7a31dd4c",
        "0x5a6c24912d41010d7244c21fbc3729216fe993a7",
        "0x7e33dde061b13d810cd5d45596e24ad6fca38639",
        "0x3ad4fdc80bdf08c73be2c0b9b7589219bc266915",
        "0x17417e3d2f78ec1190eaf706f51f30955de20361",
        "0xe85458c1a6bfb759a4981be4ae228ed72104f7ac",
        "0x92c4846308641e7e1f779ff9339ca4c2c8f47231",
        "0x66da6924307cf91cbf1d421667398bbba920821b",
        "0x5d21d4eae2fe3eb4402fadf6d6c362b448f5d3d7",
        "0x8bb150ea4c997a8880171fe6376aa78391878260",
        "0x2b901ce92727f9912552c73c69f3bfeccad9eab2",
        "0xe35ddd5ff103f9527f4a90e480d960eb5b5dfa3f",
        "0xc0496ee2dd6484c673b29c425b0bd1a517a7f75e",
        "0xbe091df7eb50dcc5f7b899f15cc09091ffd2fcb6",
        "0xf02a315724f9d52bf9a5c2eb0aa9548ba832d420",
        "0x454e84e9d2997468c3fa58c6841c351b591c2ac6",
        "0x27a855e76d0e91128a052b479df1e4fde065e177",
        "0xaf5893087901b12326c94d6f21874f3261833703",
        "0x98a8286c62d844c0c278cdd0bf2809151b83c986",
        "0x059053b532a5b7fabda627f41930c345c608b219",
        "0xd6036d15fc61f04af6820680ea9c1a85bab7f765",
        "0xa737fe84a56759b0e3a7b02ebb2aec0681ba78bd",
        "0x05e33ee4fcac57d8d33a64f64ed2a573c162110d",
        "0xbdfe8d6eb695dbf7fc14a8bbc8aafbc00a74fb51",
        "0x33f26dd31c098b869833416642f86ef82f50cb00",
        "0xc47406d4983ecd838d00910d2bcb899e67411bb1",
    ];
    let input_data: Vec<u8> = input_data
        .iter()
        .map(|x| x.strip_prefix("0x").unwrap().to_string())
        .flat_map(|x| x.as_bytes().to_vec())
        .collect();
    // ------ 压缩数据 ------
    // let mut compressed = Vec::new();

    // lzma_rs::lzma_compress(&mut input_data.as_slice(), &mut compressed).unwrap();
    let compressed =
        zstd::stream::encode_all(input_data.as_slice(), 22).expect("Compression failed");
    println!("input size: {}", input_data.len());
    println!(
        "Compressed size: {} (compression ratio: {:.2}%)",
        compressed.len(),
        (compressed.len() as f32 / input_data.len() as f32) * 100.0
    );
    // ------ 解压数据 ------
    // let mut decompressed = Vec::new();
    // lzma_rs::lzma_decompress(&mut compressed.as_slice(), &mut decompressed).unwrap();
    // assert_eq!(input_data, decompressed.as_slice());
    // println!("Decompressed successfully!");
}
