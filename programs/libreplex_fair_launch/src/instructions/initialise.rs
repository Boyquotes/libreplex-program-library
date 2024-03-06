use anchor_lang::{prelude::*, system_program};



use crate::{errors::FairLaunchError, Deployment, DeploymentConfig, InitialiseInputV2, NewDeploymentEvent, NewDeploymentV2, HYBRID_DEPLOYMENT_TYPE, OFFCHAIN_URL_LIMIT, TEMPLATE_LIMIT, TICKER_LIMIT, TOKEN2022_DEPLOYMENT_TYPE};


pub mod sysvar_instructions_program {
    use anchor_lang::declare_id;
    declare_id!("Sysvar1nstructions1111111111111111111111111");
}   

/*

    Initialise sets the main template parameters of the deployment:
    1) ticker
    2) deployment template
    3) mint template
    4) decimals
    5) limit per mint
    6) max number of tokens

    It does not create any inscriptions / mints as these are handled by the deploy endpoints.
    This method is metadata agnostic.

*/

#[derive(Accounts)]
#[instruction(input: InitialiseInput)]
pub struct InitialiseCtx<'info>  {
    #[account(init, payer = payer, space = 8 + Deployment::INIT_SPACE, 
        seeds = ["deployment".as_ref(), input.ticker.as_ref()], bump)]
    pub deployment: Account<'info, Deployment>,

    #[account(mut
        // ,
        // constraint = payer.key().to_string() == "11111111111111111111111111111111".to_owned()
    )]
    pub payer: Signer<'info>,

    #[account()]
    pub system_program: Program<'info, System>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitialiseInput {
    pub limit_per_mint: u64, // this number of SPL tokens are issued into the escrow when an op: 'mint' comes in 
    pub max_number_of_tokens: u64, // this is the max *number* of tokens
    pub decimals: u8,
    pub ticker: String,
    pub deployment_template: String,
    pub mint_template: String,
    pub offchain_url: String, // used both for the fungible and the non-fungible
    pub deployment_type: u8,
}


pub fn initialise_logic(input: InitialiseInputV2, 
    deployment: &mut Account<'_, Deployment>, 
    creator: Pubkey, config: &mut DeploymentConfig) -> Result<()> {
    let deployment_type = input.deployment_type;

    if deployment_type != TOKEN2022_DEPLOYMENT_TYPE && deployment_type != HYBRID_DEPLOYMENT_TYPE{
        panic!("Bad deployment type")
    }
    
    if deployment_type == HYBRID_DEPLOYMENT_TYPE && input.deflation_rate_per_swap > 0{
        panic!("Non-zero deflation rate requires a token-2022 deployment")
    }


    config.creator_fee_treasury = input.creator_fee_treasury;
    config.creator_fee_per_mint_lamports = input.creator_fee_per_mint_in_lamports;
    config.deflation_rate_per_swap = input.deflation_rate_per_swap;

        
    if input.ticker.len() > TICKER_LIMIT {
        return Err(FairLaunchError::TickerTooLong.into());
    }
    if input.offchain_url.len() > OFFCHAIN_URL_LIMIT {
        return Err(FairLaunchError::OffchainUrlTooLong.into());
    }
    if input.mint_template.len() > TEMPLATE_LIMIT {
        return Err(FairLaunchError::MintTemplateTooLong.into());
    }
    if input.deployment_template.len() > TEMPLATE_LIMIT {
        return Err(FairLaunchError::DeploymentTemplateTooLong.into());
    }

    deployment.require_creator_cosign = false;
    deployment.use_inscriptions = true;
    deployment.deployment_type = input.deployment_type;
    deployment.creator = creator;
    deployment.limit_per_mint = input.limit_per_mint;
    deployment.max_number_of_tokens = input.max_number_of_tokens;
    deployment.number_of_tokens_issued = 0;
    deployment.decimals = input.decimals;
    deployment.ticker = input.ticker;
    deployment.deployment_template = input.deployment_template;
    deployment.mint_template = input.mint_template;
    deployment.offchain_url = input.offchain_url;
    deployment.escrow_non_fungible_count = 0;
    deployment.migrated_from_legacy = false;
    (input.limit_per_mint).checked_mul(input.max_number_of_tokens).unwrap().checked_mul(
        (10_u64).checked_pow(input.decimals as u32).unwrap()).unwrap();
    


    deployment.require_creator_cosign = input.creator_cosign_program_id.is_some();

    config.cosigner_program_id = match input.creator_cosign_program_id {
        Some(x) => x,
        _ => system_program::ID
    };


    deployment.use_inscriptions = input.use_inscriptions;

    // Try avoid blowing up the stack
    emit_init(deployment, config);
    // for now, we limit ticker sizes to 12 bytes 

    Ok(())
}

fn emit_init(deployment: &Deployment, config: &DeploymentConfig) {
    emit!(NewDeploymentV2 {
        creator: deployment.creator,
        limit_per_mint: deployment.limit_per_mint,
        max_number_of_tokens: deployment.max_number_of_tokens,
        ticker: deployment.ticker.clone(),
        off_chain_url: deployment.offchain_url.clone(),
        require_co_sign: deployment.require_creator_cosign,
        uses_inscriptions: deployment.use_inscriptions,
        decimals: deployment.decimals,
        deployment_template: deployment.deployment_template.clone(),
        mint_template: deployment.mint_template.clone(),
        deployment_type: deployment.deployment_type,
        config: Some(config.clone())
    });
}