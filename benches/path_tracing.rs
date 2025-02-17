use criterion::{criterion_group, criterion_main, Criterion};
use pog::blockchain::path::concat_tx_hash_with_to_hash_static;
use pog::blockchain::transaction::Transaction;
use pog::wallet::Wallet;

fn sign_paths_with_bls(wallets: Vec<Wallet>, tx_hash: String, n: usize) {
    for i in 1..n + 1 {
        let next = wallets.get(i).unwrap();
        let from = wallets.get(i - 1).unwrap();
        let hash = concat_tx_hash_with_to_hash_static(tx_hash.clone(), next.address.clone());
        from.sign_by_bls(hash);
    }
}

fn sign_paths_with_secp256k1(wallets: Vec<Wallet>, tx_hash: String, n: usize) {
    for i in 1..n + 1 {
        let next = wallets.get(i).unwrap();
        let from = wallets.get(i - 1).unwrap();
        let hash = concat_tx_hash_with_to_hash_static(tx_hash.clone(), next.address.clone());
        from.sign(hash);
    }
}

fn bench_bls_sign(c: &mut Criterion) {
    let mut wallets = vec![];
    for i in 0..101 {
        wallets.push(Wallet::new());
    }
    let from = wallets.get(0).unwrap();
    let transaction = Transaction::new("123".to_string(), 32, from.clone());

    c.bench_function("bls Sign 1 time", |b| {
        b.iter(|| sign_paths_with_bls(wallets.clone(), transaction.hash.clone(), 1))
    });

    c.bench_function("bls Sign 10 times", |b| {
        b.iter(|| sign_paths_with_bls(wallets.clone(), transaction.hash.clone(), 10))
    });

    c.bench_function("bls Sign 50 times", |b| {
        b.iter(|| sign_paths_with_bls(wallets.clone(), transaction.hash.clone(), 50))
    });

    c.bench_function("bls Sign 100 times", |b| {
        b.iter(|| sign_paths_with_bls(wallets.clone(), transaction.hash.clone(), 100))
    });
}

fn bench_secp256k1_sign(c: &mut Criterion) {
    let mut wallets = vec![];
    for i in 0..101 {
        wallets.push(Wallet::new());
    }
    let from = wallets.get(0).unwrap();
    let transaction = Transaction::new("123".to_string(), 32, from.clone());

    c.bench_function("secp256k1 sign 1 time", |b| {
        b.iter(|| sign_paths_with_secp256k1(wallets.clone(), transaction.hash.clone(), 1))
    });

    c.bench_function("secp256k1 sign 10 times", |b| {
        b.iter(|| sign_paths_with_secp256k1(wallets.clone(), transaction.hash.clone(), 10))
    });

    c.bench_function("secp256k1 sign 50 times", |b| {
        b.iter(|| sign_paths_with_secp256k1(wallets.clone(), transaction.hash.clone(), 50))
    });

    c.bench_function("secp256k1 sign 100 times", |b| {
        b.iter(|| sign_paths_with_secp256k1(wallets.clone(), transaction.hash.clone(), 100))
    });
}

criterion_group!(benches, bench_bls_sign, bench_secp256k1_sign);
criterion_main!(benches);
