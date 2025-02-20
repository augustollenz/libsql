use crate::hrana::pipeline::{ExecuteStreamReq, StreamRequest};
use crate::hrana::proto::{Batch, BatchResult, Stmt, StmtResult};
use crate::hrana::stream::HranaStream;
use crate::hrana::{HttpSend, Result};
use crate::TransactionBehavior;

#[derive(Debug, Clone)]
pub(crate) struct HttpTransaction<T>
where
    T: HttpSend,
{
    stream: HranaStream<T>,
}

impl<T> HttpTransaction<T>
where
    T: HttpSend,
{
    pub fn stream(&self) -> &HranaStream<T> {
        &self.stream
    }

    pub async fn open(stream: HranaStream<T>, tx_behavior: TransactionBehavior) -> Result<Self> {
        let begin_stmt = match tx_behavior {
            TransactionBehavior::Deferred => "BEGIN DEFERRED",
            TransactionBehavior::Immediate => "BEGIN IMMEDIATE",
            TransactionBehavior::Exclusive => "BEGIN EXCLUSIVE",
            TransactionBehavior::ReadOnly => "BEGIN READONLY",
        };
        stream.execute(Stmt::new(begin_stmt, false)).await?;
        Ok(HttpTransaction { stream })
    }

    pub async fn execute(&self, stmt: Stmt) -> Result<StmtResult> {
        self.stream.execute(stmt).await
    }

    pub async fn execute_batch(
        &self,
        stmts: impl IntoIterator<Item = Stmt>,
    ) -> Result<BatchResult> {
        let batch = Batch::from_iter(stmts, false);
        self.stream.batch(batch).await
    }

    pub async fn commit(&mut self) -> Result<()> {
        let stmt = Stmt::new("COMMIT", false);
        self.stream
            .finalize(StreamRequest::Execute(ExecuteStreamReq { stmt }))
            .await?;
        Ok(())
    }

    pub async fn rollback(&mut self) -> Result<()> {
        let stmt = Stmt::new("ROLLBACK", false);
        self.stream
            .finalize(StreamRequest::Execute(ExecuteStreamReq { stmt }))
            .await?;
        Ok(())
    }
}
