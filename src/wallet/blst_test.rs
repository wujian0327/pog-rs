use crate::wallet::Wallet;

#[cfg(test)]
mod tests {
    use blst::min_sig::{AggregateSignature, PublicKey, SecretKey, Signature};
    use blst::{blst_keygen, blst_scalar, BLST_ERROR};
    use rand::RngCore;

    #[test]
    fn blst_verify() {
        let mut rng = rand::thread_rng();
        let mut ikm = [0u8; 32];
        rng.fill_bytes(&mut ikm);

        let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
        let pk = sk.sk_to_pk();

        let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";
        let msg = b"blst is such a blast";
        let sig = sk.sign(msg, dst, &[]);

        let err = sig.verify(true, msg, dst, &[], &pk, true);
        assert_eq!(err, blst::BLST_ERROR::BLST_SUCCESS);
    }

    #[test]
    fn test_aggregate() {
        // 生成 3 组密钥对和消息（实际应用中消息可以是相同的或不同的）
        let mut rng = rand::thread_rng();

        // 模拟三个参与方的签名
        let messages = [
            b"message1".as_slice(),
            b"message2".as_slice(),
            b"message3".as_slice(),
        ];
        let mut signatures = Vec::new();
        let mut public_keys = Vec::new();

        // Step 1: 每个用户独立签名
        for msg in &messages {
            // 生成密钥对
            let mut sk_bytes = [0u8; 32];
            rng.fill_bytes(&mut sk_bytes);
            let sk = SecretKey::key_gen(&sk_bytes, &[]).unwrap();
            let pk = sk.sk_to_pk();

            // 对各自消息签名
            let sig = sk.sign(msg, &[], &[]);
            signatures.push(sig);
            public_keys.push(pk);
        }

        // Step 2: 聚合所有签名（支持不同公钥和不同消息的情况）
        let mut agg_sig = AggregateSignature::from_signature(&signatures[0]);
        for sig in &signatures[1..] {
            agg_sig.add_signature(sig, true).unwrap();
        }
        let messages: Vec<&[u8]> = messages.iter().map(|&m| m).collect();
        // Step 3: 构建聚合验证所需的参数（公钥和消息列表）
        let public_keys: Vec<&PublicKey> = public_keys.iter().map(|pk| pk).collect();

        // Step 4: 验证聚合签名（关键步骤）
        match agg_sig.to_signature().aggregate_verify(
            true,
            messages.as_slice(),
            &[],
            public_keys.as_slice(),
            true,
        ) {
            BLST_ERROR::BLST_SUCCESS => println!("聚合验证成功!"),
            err => eprintln!("聚合验证失败: {:?}", err),
        }
    }
}
