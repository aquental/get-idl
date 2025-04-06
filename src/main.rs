use anchor_lang::idl::IdlAccount;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::ParsePubkeyError;
use solana_sdk::hash::hash;
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io::Write;

// Custom error type to handle all possible errors
#[derive(Debug)]
pub enum IdlError {
    PubkeyParseError(ParsePubkeyError),
    ClientError(solana_client::client_error::ClientError),
    IoError(std::io::Error),
    SerdeJsonError(serde_json::Error),
    AnchorError(anchor_lang::error::Error),
    CustomError(String),
}

impl fmt::Display for IdlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdlError::PubkeyParseError(err) => write!(f, "Pubkey parse error: {}", err),
            IdlError::ClientError(err) => write!(f, "Solana client error: {}", err),
            IdlError::IoError(err) => write!(f, "IO error: {}", err),
            IdlError::SerdeJsonError(err) => write!(f, "JSON serialization error: {}", err),
            IdlError::AnchorError(err) => write!(f, "Anchor error: {}", err),
            IdlError::CustomError(err) => write!(f, "Error: {}", err),
        }
    }
}

impl StdError for IdlError {}

// Implement From for all error types we use
impl From<ParsePubkeyError> for IdlError {
    fn from(error: ParsePubkeyError) -> Self {
        IdlError::PubkeyParseError(error)
    }
}

impl From<solana_client::client_error::ClientError> for IdlError {
    fn from(error: solana_client::client_error::ClientError) -> Self {
        IdlError::ClientError(error)
    }
}

impl From<std::io::Error> for IdlError {
    fn from(error: std::io::Error) -> Self {
        IdlError::IoError(error)
    }
}

impl From<serde_json::Error> for IdlError {
    fn from(error: serde_json::Error) -> Self {
        IdlError::SerdeJsonError(error)
    }
}

impl From<anchor_lang::error::Error> for IdlError {
    fn from(error: anchor_lang::error::Error) -> Self {
        IdlError::AnchorError(error)
    }
}

impl From<&str> for IdlError {
    fn from(error: &str) -> Self {
        IdlError::CustomError(error.to_string())
    }
}

pub enum Cluster {
    Devnet,
    Testnet,
    Mainnet,
}

impl Cluster {
    fn url(&self) -> String {
        match self {
            Cluster::Devnet => "https://api.devnet.solana.com".to_string(),
            Cluster::Testnet => "https://api.testnet.solana.com".to_string(),
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com".to_string(),
        }
    }
}

pub fn generate_local_idl(program_address: &str, cluster: Cluster) -> std::result::Result<(), IdlError> {
    // Convert program address string to Pubkey
    let program_id = program_address.parse::<solana_sdk::pubkey::Pubkey>()?;

    // Set up RPC client for the specified cluster
    let rpc_url = cluster.url();
    let client = RpcClient::new(rpc_url);

    // Fetch the account data for the program
    let account = client.get_account(&program_id)?;

    // Check if the account is a program account
    if !account.executable {
        return Err("The provided address does not correspond to an executable program".into());
    }

    // Fetch the IDL from the program (assuming it's stored in a standard location)
    // Note: Anchor stores IDL in a specific account derived from the program ID
    // Note: Anchor stores IDL in a specific account derived from the program ID
    let idl_address = IdlAccount::address(&program_id);
    let idl_account = client.get_account_data(&idl_address)?;

    // Parse the IDL account data according to Anchor's format:
    // - First 8 bytes: account discriminator
    // - Next 32 bytes: authority (Pubkey)
    // - Next 8 bytes: data length (u64)
    // - Remaining bytes: actual IDL data serialized with borsh
    
    // Skip the discriminator (8 bytes)
    if idl_account.len() <= 8 {
        return Err(IdlError::CustomError("Invalid IDL account data: too short".to_string()));
    }
    
    // Verify the discriminator
    // Verify the discriminator
    // Anchor's discriminator is first 8 bytes of the SHA256 hash of "anchor:idl"
    let disc_bytes = &idl_account[0..8];
    let expected_discriminator = {
        let preimage = "anchor:idl";
        let hash = hash(preimage.as_bytes());
        &hash.to_bytes()[0..8]
    };
    
    if disc_bytes != expected_discriminator {
        return Err(IdlError::CustomError("Invalid IDL account: wrong discriminator".to_string()));
    }
    // Skip the discriminator and authority bytes (8 + 32 = 40)
    let data_len_bytes = &idl_account[40..48];
    let data_len = u64::from_le_bytes(data_len_bytes.try_into().unwrap()) as usize;
    
    // The actual IDL data starts at byte 48
    if idl_account.len() < 48 + data_len {
        return Err(IdlError::CustomError("Invalid IDL account data: truncated".to_string()));
    }
    
    let idl_data = &idl_account[48..48 + data_len];
    
    // Deserialize the IDL using serde_json
    let idl: serde_json::Value = serde_json::from_slice(idl_data)
        .map_err(|e| IdlError::CustomError(format!("Failed to parse IDL data: {}", e)))?;
    // Serialize the IDL to JSON
    let idl_json = serde_json::to_string_pretty(&idl)?;

    // Write the IDL to a local file
    let mut file = File::create(format!("{}.json", program_address))?;
    file.write_all(idl_json.as_bytes())?;

    println!("IDL successfully saved to {}.json", program_address);
    Ok(())
}

// Example usage
fn main() -> std::result::Result<(), IdlError> {
    let program_address = "ADcaide4vBtKuyZQqdU689YqEGZMCmS4tL35bdTv9wJa";
    let cluster = Cluster::Devnet;

    generate_local_idl(program_address, cluster)?;
    Ok(())
}
