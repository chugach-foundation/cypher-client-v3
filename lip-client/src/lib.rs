pub mod constants;
pub mod instructions;
pub mod utils;

anchor_gen::generate_cpi_interface!(idl_path = "idl.json",);

#[cfg(feature = "mainnet-beta")]
declare_id!("9ZoxkdieCwUWu57SyNkd4PR2fb7azpNiefeZjwDbzBQo");
#[cfg(not(feature = "mainnet-beta"))]
declare_id!("9ZoxkdieCwUWu57SyNkd4PR2fb7azpNiefeZjwDbzBQo");
