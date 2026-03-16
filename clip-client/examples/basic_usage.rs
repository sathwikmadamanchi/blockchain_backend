use clip_client::ClipClient;
use clip_client::proto::Transaction;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClipClient::connect("http://[::1]:50051".to_string()).await?;
    println!("Connected to CLIP Ledger!");

    let txs = vec![Transaction {
        hash: vec![1u8; 32],
        timestamp: 123456789,
        metadata: b"Hello CLIP".to_vec(),
    }];

    let response = client.submit_batch(txs).await?;
    println!("Submitted batch: {:#?}", response);

    if response.success {
        let proof = client.verify_proof(response.block_index, vec![1u8; 32]).await?;
        println!("Proof verified: {}", proof.is_valid);
    }

    Ok(())
}
