use anchor_lang::prelude::*;

use state::*;
use errors::*;

declare_id!("insFmVukT9LYVygNbdpSjbxPy4FtQ6WgcuChnxDLbAm");

pub mod state;
pub mod errors;

#[program]
pub mod libreplex_spl20 {
    use super::*;

    pub fn register_token(ctx: Context<RegisterTokenCtx>, new_deployment: TokenDeployment) -> Result<()> {
        let deployment = &mut ctx.accounts.new_deployment_account;

        if new_deployment.ticker.len() > TICKER_LIMIT {
            return Err(Spl20Error::TickerToLong.into());
        }
        
        deployment.creator = new_deployment.creator;
        deployment.limit = new_deployment.limit;
        deployment.max = new_deployment.max;
        deployment.collection = new_deployment.collection;
        deployment.ticker = new_deployment.ticker;
        deployment.root = new_deployment.root;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(new_deployment: TokenDeployment)]
pub struct RegisterTokenCtx<'info>  {
    #[account(init, payer = payer, space = 8 + TokenDeployment::INIT_SPACE + EXCESS, 
        seeds = ["spl20".as_ref(), new_deployment.ticker.as_ref()], bump)]
    pub new_deployment_account: Account<'info, TokenDeployment>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}