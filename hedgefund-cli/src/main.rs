use clap::{Arg, Command};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use ethers::prelude::*;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use ethers::types::{TransactionRequest, Bytes};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::utils::keccak256;
use std::io::Write;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("AI Hedge Fund CLI")
        .version("0.1.0")
        .about("A cool CLI for the on-chain AI Hedge Fund (split among LINK, WBTC, Lending).")
        .arg(Arg::new("endpoint")
            .long("endpoint")
            .required(true)
            .help("Arbitrum RPC endpoint. E.g. https://sepolia-rollup.arbitrum.io/rpc")
            .value_name("URL"))
        .arg(Arg::new("private_key")
            .long("private-key")
            .help("Your private key for state-changing ops")
            .value_name("KEY"))
        .arg(Arg::new("contract")
            .long("contract")
            .required(true)
            .help("The deployed HedgeFund contract address")
            .value_name("ADDR"))
        .get_matches();

    let endpoint = matches.get_one::<String>("endpoint").expect("Required argument");
    let pk = matches.get_one::<String>("private_key").map(|s| s.as_str()).unwrap_or("");
    let contract_str = matches.get_one::<String>("contract").expect("Required argument");

    // Setup blockchain connection
    let provider = Provider::<Http>::try_from(endpoint.to_string())?;
    let chain_id = provider.get_chainid().await?.as_u64();

    let wallet = if !pk.is_empty() {
        LocalWallet::from_str(pk)?.with_chain_id(chain_id)
    } else {
        println!("{}", "\nWarning: No private key provided. Running in read-only mode.".yellow().bold());
        // minimal dummy key
        LocalWallet::from_str("0000000000000000000000000000000000000000000000000000000000000001")?
            .with_chain_id(chain_id)
    };

    let client = Arc::new(SignerMiddleware::new(provider, wallet));
    let contract_addr = contract_str.parse::<Address>()?;

    // Welcome message
    println!(
        "\n{}",
        "Welcome to the AI Hedge Fund CLI! 💼"
            .bright_green()
            .bold()
    );
    println!(
        "Endpoint: {}, Contract: {}",
        endpoint.bright_yellow(),
        contract_str.bright_yellow()
    );

    loop {
        let options = vec![
            "Deposit ETH",
            "Withdraw Shares",
            "Get User Info",
            "Rebalance Portfolio",
            "Status of Agents",
            "Exit",
        ];

        println!("\nMain Menu:");
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an operation")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => handle_deposit(&client, contract_addr).await?,
            1 => handle_withdraw(&client, contract_addr).await?,
            2 => handle_get_info(&client, contract_addr).await?,
            3 => handle_rebalance(&client, contract_addr).await?,
            4 => handle_status(&client, contract_addr).await?,
            5 => {
                println!("\nExiting... goodbye!");
                break;
            }
            _ => println!("{}", "Invalid option".red()),
        }
    }

    Ok(())
}

//////////////////////////////////////////
// Each CLI handler
//////////////////////////////////////////

async fn handle_deposit<M: Middleware + 'static>(
    client: &Arc<SignerMiddleware<M, LocalWallet>>,
    contract_addr: Address
) -> Result<(), Box<dyn Error>> {
    println!("\n{}", "📥 Deposit ETH".bright_cyan().bold());
    let amount_str: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter ETH amount (e.g. 0.5)")
        .interact_text()?;

    let amount_f = amount_str.parse::<f64>()?;
    let wei_amount = (amount_f * 1e18) as u64;

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Confirm deposit of {} ETH?", amount_str))
        .interact()?;

    if confirm {
        // Show a simple spinner
        print!("Processing deposit");
        std::io::stdout().flush()?;
        for _ in 0..3 {
            print!(".");
            std::io::stdout().flush()?;
            thread::sleep(Duration::from_millis(300));
        }
        println!();

        let deposit_sig = function_selector("deposit()");

        let tx_req = TransactionRequest::new()
            .to(contract_addr)
            .data(deposit_sig)
            .value(wei_amount)
            .gas(3_000_000);

        let pending = client.send_transaction(tx_req, None).await?;
        let receipt = pending.await?.ok_or("No receipt")?;

        println!(
            "{} {}",
            "Deposit TX:".bright_green().bold(),
            format!("0x{:x}", receipt.transaction_hash).yellow()
        );
    }

    Ok(())
}

async fn handle_withdraw<M: Middleware + 'static>(
    client: &Arc<SignerMiddleware<M, LocalWallet>>,
    contract_addr: Address
) -> Result<(), Box<dyn Error>> {
    println!("\n{}", "📤 Withdraw Shares".bright_magenta().bold());

    let shares_str: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter number of shares to withdraw")
        .interact_text()?;
    let shares_u256 = U256::from_dec_str(&shares_str)?;

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Confirm withdrawal of {} shares?", shares_str))
        .interact()?;

    if confirm {
        // "withdraw(uint256)"
        let sig = function_selector("withdraw(uint256)");
        let param = encode_u256(shares_u256);

        let mut data = sig;
        data.extend_from_slice(&param);

        let tx_req = TransactionRequest::new()
            .to(contract_addr)
            .data(data)
            .gas(3_000_000u64);

        let pending = client.send_transaction(tx_req, None).await?;
        let receipt = pending.await?.ok_or("No receipt")?;
        println!(
            "{} {}",
            "Withdraw TX:".bright_green().bold(),
            format!("0x{:x}", receipt.transaction_hash).yellow()
        );
    }

    Ok(())
}

async fn handle_get_info<M: Middleware + 'static>(
    client: &Arc<SignerMiddleware<M, LocalWallet>>,
    contract_addr: Address
) -> Result<(), Box<dyn Error>> {
    println!("\n{}", "📊 Get User Info".bright_blue().bold());

    let address: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter user address (or press enter to skip for your address):")
        .allow_empty(true)
        .interact_text()?;

    let user_addr = if address.is_empty() {
        client.address()
    } else {
        address.parse::<Address>()?
    };

    // get_user_info(address)
    let sig = function_selector("get_user_info(address)");
    let enc = encode_address(user_addr);

    let mut data = sig;
    data.extend_from_slice(&enc);

    let tx: TypedTransaction = TransactionRequest::new()
        .to(contract_addr)
        .data(Bytes::from(data))
        .into();
    let result = client.call(&tx, None).await?;

    if result.is_empty() {
        println!("No data returned from get_user_info.");
    } else {
        let s = String::from_utf8_lossy(&result);
        println!(
            "{}\n{}",
            "User Info:".blue().bold(),
            s.bright_cyan()
        );
    }

    Ok(())
}

async fn handle_rebalance<M: Middleware + 'static>(
    client: &Arc<SignerMiddleware<M, LocalWallet>>,
    contract_addr: Address
) -> Result<(), Box<dyn Error>> {
    println!("\n{}", "⚖️  Rebalancing Portfolio".bright_yellow().bold());

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure you want to rebalance the portfolio?")
        .interact()?;

    if confirm {
        let sig = function_selector("rebalance()");
        let tx_req = TransactionRequest::new()
            .to(contract_addr)
            .data(sig)
            .gas(3_000_000);

        let pending = client.send_transaction(tx_req, None).await?;
        let receipt = pending.await?.ok_or("No receipt")?;
        println!(
            "{} {}",
            "Rebalance TX:".bright_green().bold(),
            format!("0x{:x}", receipt.transaction_hash).yellow()
        );
    }

    Ok(())
}

async fn handle_status<M: Middleware + 'static>(
    client: &Arc<SignerMiddleware<M, LocalWallet>>,
    contract_addr: Address
) -> Result<(), Box<dyn Error>> {
    println!("\n{}", "🤖 Agent Status".bright_yellow().bold());

    let sig = function_selector("get_agent_invests()");
    let tx: TypedTransaction = TransactionRequest::new()
        .to(contract_addr)
        .data(Bytes::from(sig))
        .into();
    let result = client.call(&tx, None).await?;

    if result.is_empty() {
        println!("No data returned from get_agent_invests.");
    } else {
        let s = String::from_utf8_lossy(&result);
        println!(
            "{}\n{}",
            "Agent Status:".yellow().bold(),
            s.bright_cyan()
        );
    }

    Ok(())
}

//////////////////////////////////////////
// Helper Functions
//////////////////////////////////////////

fn function_selector(signature: &str) -> Vec<u8> {
    keccak256(signature.as_bytes())[..4].to_vec()
}

fn encode_u256(value: U256) -> Vec<u8> {
    let mut buf = vec![0u8; 32];
    value.to_big_endian(&mut buf);
    buf
}

fn encode_address(addr: Address) -> Vec<u8> {
    let mut buf = vec![0u8; 32];
    buf[12..].copy_from_slice(&addr.0);
    buf
}
