#![cfg(target_os = "linux")]

use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::thread;

use base64::Engine as _;
use crunch_nix_gateway::server::authenticate_ticket_envelope;
use crunch_nix_gateway_core::Reject;

// Test the real repository verifier, not an echo callback or a fabricated
// authorization boolean. Production imports this module from the host shell.
#[allow(dead_code)]
#[path = "../../../src/remote_credentials.rs"]
mod remote_credentials;

use remote_credentials::RemoteTicket;
use remote_credentials::RemoteTicketState;
use remote_credentials::TicketAuthorization;
use remote_credentials::TicketIssueInput;
use remote_credentials::TicketPolicyFacts;
use remote_credentials::TicketVerifierKey;
use remote_credentials::apply_ticket_issue;
use remote_credentials::authorize_ticket;
use remote_credentials::plan_ticket_issue;

const AUDIENCE: &str = "gateway-a";
const CREATED: u64 = 100;
const CHALLENGE_A: [u8; 32] = [0x35; 32];
const CHALLENGE_B: [u8; 32] = [0x53; 32];

fn issued_ticket() -> (RemoteTicket, String) {
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0x41_u8; 32]);
    let key = Arc::new(TicketVerifierKey::parse(&format!("ticket-key-1:{encoded}")).unwrap());
    let mut state = RemoteTicketState::default();
    let plan = plan_ticket_issue(
        &state,
        TicketIssueInput {
            display_name: "gateway-operator".to_string(),
            now_unix_s: CREATED,
            ttl_secs: 300,
            uses: 1,
            max_build_time_secs: 60,
            max_upload_bytes: 1_024,
            bound_client_endpoint: Some(AUDIENCE.to_string()),
        },
        &[0x23_u8; 32],
        &key,
    )
    .unwrap();
    let credential = plan.with_bearer_credential(str::to_string);
    let ticket_id = credential.split_once(':').unwrap().0.to_string();
    apply_ticket_issue(&mut state, &plan).unwrap();
    (state.tickets.remove(&ticket_id).unwrap(), credential)
}

fn envelope(challenge: &[u8; 32], credential: &str, audience: &str) -> Vec<u8> {
    let (id, bearer) = credential.split_once(':').unwrap();
    let mut frame = Vec::with_capacity(32 + 3 + id.len() + bearer.len() + audience.len());
    frame.extend_from_slice(challenge);
    for field in [id, bearer, audience] {
        frame.push(u8::try_from(field.len()).unwrap());
        frame.extend_from_slice(field.as_bytes());
    }
    frame
}

fn attempt(
    ticket: RemoteTicket,
    challenge: [u8; 32],
    frame: &[u8],
    endpoint: &'static str,
    now: u64,
) -> Result<(), Reject> {
    let (mut client, mut server) = UnixStream::pair().unwrap();
    let service = thread::spawn(move || {
        authenticate_ticket_envelope(&mut server, &challenge, AUDIENCE, |id, bearer| {
            let facts = TicketPolicyFacts {
                ticket_id: id,
                client_endpoint: Some(endpoint),
                now_unix_s: now,
            };
            match authorize_ticket(&ticket, bearer, &facts) {
                TicketAuthorization::Authorized => Ok(()),
                TicketAuthorization::Rejected(_) => Err(Reject::Authority),
            }
        })
    });
    let mut greeting = [0_u8; 40];
    client.read_exact(&mut greeting).unwrap();
    assert_eq!(&greeting[..8], b"MNTLAUT1");
    assert_eq!(&greeting[8..], &challenge);
    client.write_all(&(frame.len() as u64).to_le_bytes()).unwrap();
    client.write_all(frame).unwrap();
    service.join().unwrap()
}

#[test]
fn ticket_envelope_checks_real_secret_expiration_endpoint_audience_and_replayed_frame() {
    let (ticket, credential) = issued_ticket();
    let presented = envelope(&CHALLENGE_A, &credential, AUDIENCE);
    assert_eq!(attempt(ticket.clone(), CHALLENGE_A, &presented, AUDIENCE, CREATED), Ok(()));
    assert_eq!(attempt(ticket.clone(), CHALLENGE_A, &presented, AUDIENCE, CREATED + 300), Err(Reject::Authority));
    assert_eq!(attempt(ticket.clone(), CHALLENGE_A, &presented, "gateway-b", CREATED), Err(Reject::Authority));
    assert_eq!(
        attempt(ticket.clone(), CHALLENGE_A, &envelope(&CHALLENGE_A, &credential, "gateway-b"), AUDIENCE, CREATED),
        Err(Reject::Audience)
    );
    assert_eq!(attempt(ticket.clone(), CHALLENGE_B, &presented, AUDIENCE, CREATED), Err(Reject::Authority));
    let id = credential.split_once(':').unwrap().0;
    let forged_secret = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0x34_u8; 32]);
    let wrong_bearer = format!("{id}:{forged_secret}");
    assert_eq!(
        attempt(ticket, CHALLENGE_A, &envelope(&CHALLENGE_A, &wrong_bearer, AUDIENCE), AUDIENCE, CREATED),
        Err(Reject::Authority)
    );
}

#[test]
fn credential_frame_limit_rejects_before_reading_or_authenticating_secret() {
    let (mut client, mut server) = UnixStream::pair().unwrap();
    let service = thread::spawn(move || {
        authenticate_ticket_envelope(&mut server, &CHALLENGE_A, AUDIENCE, |_, _| {
            panic!("oversized frame reached ticket verifier")
        })
    });
    let mut greeting = [0_u8; 40];
    client.read_exact(&mut greeting).unwrap();
    client.write_all(&513_u64.to_le_bytes()).unwrap();
    assert_eq!(service.join().unwrap(), Err(Reject::Bound));
}
