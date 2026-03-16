pub mod proto {
    tonic::include_proto!("clip");
}

pub use proto::clip_ledger_client::ClipLedgerClient;
pub use proto::{
    SubmitBatchRequest, Transaction, VerifyProofRequest, GetBlockRequest, 
    SubmitBatchResponse, VerifyProofResponse, GetBlockResponse
};

use tonic::transport::Channel;

pub struct ClipClient {
    inner: ClipLedgerClient<Channel>,
}

impl ClipClient {
    pub async fn connect(addr: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(addr)?.connect().await?;
        Ok(Self { inner: ClipLedgerClient::new(channel) })
    }

    pub async fn submit_batch(&mut self, transactions: Vec<Transaction>) -> Result<SubmitBatchResponse, tonic::Status> {
        let request = tonic::Request::new(SubmitBatchRequest { transactions });
        Ok(self.inner.submit_batch(request).await?.into_inner())
    }

    pub async fn verify_proof(&mut self, block_index: u64, tx_hash: Vec<u8>) -> Result<VerifyProofResponse, tonic::Status> {
        let request = tonic::Request::new(VerifyProofRequest { block_index, transaction_hash: tx_hash });
        Ok(self.inner.verify_proof(request).await?.into_inner())
    }

    pub async fn get_block(&mut self, block_index: u64) -> Result<GetBlockResponse, tonic::Status> {
        let request = tonic::Request::new(GetBlockRequest { block_index });
        Ok(self.inner.get_block(request).await?.into_inner())
    }
}
