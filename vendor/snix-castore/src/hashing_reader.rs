use pin_project_lite::pin_project;
use tokio::io::AsyncRead;

pin_project! {
    /// Wraps an existing AsyncRead, and allows querying for the digest of all
    /// data read "through" it.
    /// The hash function is configurable by type parameter.
    pub struct HashingReader<R, H>
    where
        R: AsyncRead,
        H: digest::Digest,
    {
        #[pin]
        inner: R,
        hasher: H,
    }
}

pin_project! {
    /// Wraps an existing AsyncRead and hashes all data read through it with BLAKE3.
    pub struct B3HashingReader<R>
    where
        R: AsyncRead,
    {
        #[pin]
        inner: R,
        hasher: blake3::Hasher,
    }
}

impl<R: AsyncRead> B3HashingReader<R> {
    pub fn from(r: R) -> Self {
        Self {
            inner: r,
            hasher: blake3::Hasher::new(),
        }
    }

    /// Return the digest.
    pub fn digest(self) -> blake3::Hash {
        self.hasher.finalize()
    }
}

impl<R: AsyncRead> AsyncRead for B3HashingReader<R> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let buf_filled_len_before = buf.filled().len();

        let this = self.project();
        let ret = this.inner.poll_read(cx, buf);

        // Hash only bytes appended by this read, regardless of its poll result.
        this.hasher.update(&buf.filled()[buf_filled_len_before..]);

        ret
    }
}

impl<R, H> HashingReader<R, H>
where
    R: AsyncRead,
    H: digest::Digest,
{
    pub fn from(r: R) -> Self {
        Self {
            inner: r,
            hasher: H::new(),
        }
    }

    /// Return the digest.
    pub fn digest(self) -> digest::Output<H> {
        self.hasher.finalize()
    }
}

impl<R, H> tokio::io::AsyncRead for HashingReader<R, H>
where
    R: AsyncRead,
    H: digest::Digest,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let buf_filled_len_before = buf.filled().len();

        let this = self.project();
        let ret = this.inner.poll_read(cx, buf);

        // write everything new filled into the hasher.
        this.hasher.update(&buf.filled()[buf_filled_len_before..]);

        ret
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use rstest::rstest;
    use tokio::io::AsyncRead as _;

    use crate::B3Digest;
    use crate::B3HashingReader;
    use crate::fixtures::BLOB_A;
    use crate::fixtures::BLOB_A_DIGEST;
    use crate::fixtures::BLOB_B;
    use crate::fixtures::BLOB_B_DIGEST;
    use crate::fixtures::EMPTY_BLOB_DIGEST;

    #[rstest]
    #[case::blob_a(&BLOB_A, &BLOB_A_DIGEST)]
    #[case::blob_b(&BLOB_B, &BLOB_B_DIGEST)]
    #[case::empty_blob(&[], &EMPTY_BLOB_DIGEST)]
    #[tokio::test]
    async fn test_b3_hashing_reader(#[case] data: &[u8], #[case] b3_digest: &B3Digest) {
        let r = Cursor::new(data);
        let mut hr = B3HashingReader::from(r);

        tokio::io::copy(&mut hr, &mut tokio::io::sink()).await.expect("read must succeed");

        assert_eq!(*b3_digest, hr.digest().into());
    }

    #[tokio::test]
    async fn test_b3_hashing_reader_prefilled_partial_reads() {
        let r = tokio_test::io::Builder::new().read(b"abc").read(b"def").build();
        let mut hr = B3HashingReader::from(r);
        let mut storage = [0; 16];
        let mut buf = tokio::io::ReadBuf::new(&mut storage);
        buf.put_slice(b"prefix:");

        // Keep the prefix and earlier reads filled, including on the EOF poll.
        for expected in ["prefix:abc", "prefix:abcdef", "prefix:abcdef"] {
            std::future::poll_fn(|cx| std::pin::Pin::new(&mut hr).poll_read(cx, &mut buf))
                .await
                .expect("read must succeed");
            assert_eq!(buf.filled(), expected.as_bytes());
        }

        let digest = hr.digest();
        assert_eq!(digest, blake3::hash(b"abcdef"));
        assert_ne!(digest, blake3::hash(b"prefix:abcdef"));
        assert_ne!(digest, blake3::hash(b"abcabcdef"));
    }
}
