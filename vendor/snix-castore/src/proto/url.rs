use crate::proto;
use data_encoding::BASE64URL_NOPAD;
use prost::Message;
use tracing::warn;

/// From a given root node and nar_size, writes a castore-infused NAR path to the writer.
pub fn write_infused_nar_path(
    w: &mut impl std::fmt::Write,
    node: crate::Node,
    nar_size: u64,
) -> Result<(), std::fmt::Error> {
    let proto_node = proto::Entry::from_name_and_node("".into(), node).encode_to_vec();

    write!(
        w,
        "nar/snix-castore/{}?narsize={}",
        BASE64URL_NOPAD.encode(&proto_node),
        nar_size,
    )
}

/// Detects and parses a castore-infused NAR path.
#[tracing::instrument(level = "trace")]
pub fn parse_infused_nar_path(s: &str) -> Option<(crate::Node, u64)> {
    let s = s.strip_prefix("nar/snix-castore/")?;
    let (node_enc, nar_size_str) = s.split_once("?narsize=")?;
    let nar_size: u64 = nar_size_str
        .parse()
        .inspect_err(|err| {
            warn!(%err, "got invalid narsize");
        })
        .ok()?;

    let node = parse_urlsafe_proto(node_enc)?;

    Some((node, nar_size))
}

/// Takes a urlsafe base64-encoded castore node (in proto) and attempts to parse it.
pub fn parse_urlsafe_proto(node_enc: impl AsRef<[u8]>) -> Option<crate::Node> {
    // check the proto size to be somewhat reasonable before parsing it.
    if node_enc.as_ref().len() > BASE64URL_NOPAD.encode_len(4096) {
        warn!("rejected too large root node");
        return None;
    }

    let node_bytes = BASE64URL_NOPAD
        .decode(node_enc.as_ref())
        .inspect_err(|err| {
            warn!(%err, "unable to decode node b64");
        })
        .ok()?;

    // parse the proto
    let node_proto: proto::Entry = Message::decode(node_bytes.as_slice())
        .inspect_err(|err| {
            warn!(%err, "unable to decode node proto");
        })
        .ok()?;

    let root_node = node_proto
        .try_into_anonymous_node()
        .inspect_err(|err| {
            warn!(%err, "node validation failed");
        })
        .ok()?;

    Some(root_node)
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use crate::Node;
    use crate::fixtures::{
        DIRECTORY_COMPLICATED, HELLOWORLD_BLOB_CONTENTS, HELLOWORLD_BLOB_DIGEST,
    };
    use crate::proto::{parse_infused_nar_path, write_infused_nar_path};

    #[rstest]
    #[case::directory_complicated(
        Node::Directory { digest: DIRECTORY_COMPLICATED.digest(), size: DIRECTORY_COMPLICATED.size() },
    )]
    #[case::blob_helloworld(
        Node::File { digest: *HELLOWORLD_BLOB_DIGEST, size: HELLOWORLD_BLOB_CONTENTS.len() as u64, executable: false }
    )]
    fn roundtrip(#[case] node: Node) {
        let nar_size: u64 = 42; // dummy value
        let path = {
            let mut path = String::new();
            write_infused_nar_path(&mut path, node.clone(), nar_size).expect("write string");
            path
        };

        assert_eq!(
            (node, nar_size),
            parse_infused_nar_path(&path).expect("to parse"),
            "expected to roundtrip"
        );
    }
}
