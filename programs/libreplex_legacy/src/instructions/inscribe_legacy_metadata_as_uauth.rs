use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use libreplex_inscriptions::{
    instructions::SignerType, program::LibreplexInscriptions,
};
// use mpl_token_metadata::types::TokenStandard;


use crate::legacy_inscription::LegacyInscription;


use super::check_metadata_uauth;
use super::create_legacy_inscription_logic::create_legacy_inscription_logic;
use super::create_legacy_inscription_logic::AuthorityType;


// Adds a metadata to a group
#[derive(Accounts)]
#[instruction(validation_hash: String)]
pub struct InscribeLegacyMetadataAsUauth<'info> {
    #[account(mut

    // constraint = payer.key().to_string() == "F1QyW2RiabaUTHYYMZs6kVQmjw3QzhRWtAJNUp6ifWAe"

)]
    pub payer: Signer<'info>,

    /// CHECK: For PDA signing only
    #[account(
        mut,
        seeds=[
            mint.key().as_ref(),
        ],
        bump
    )]
    pub legacy_signer: UncheckedAccount<'info>,

    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: Checked via a CPI call
    #[account(mut)]
    pub inscription: UncheckedAccount<'info>,

    /// CHECK: Checked via a CPI call
    #[account(mut)]
    pub inscription_data: UncheckedAccount<'info>,

    /// CHECK: Checked via a CPI call
    #[account(mut)]
    pub inscription_summary: UncheckedAccount<'info>,

    /// CHECK: Checked via a CPI call
    #[account(mut)]
    pub inscription_ranks_current_page: UncheckedAccount<'info>,

    /// CHECK: Checked via a CPI call
    #[account(mut)]
    pub inscription_ranks_next_page: UncheckedAccount<'info>,

    #[account(init,
        payer = payer,
        space = LegacyInscription::SIZE,
        seeds=[
            "legacy_inscription".as_bytes(),
            mint.key().as_ref()
        ], bump)]
    pub legacy_inscription: Account<'info, LegacyInscription>,

    // /// CHECK: Checked in logic
    // #[account()]
    // pub legacy_mint: UncheckedAccount<'info>,
    /// CHECK: Checked in logic
    #[account()]
    pub legacy_metadata: UncheckedAccount<'info>,

    /// CHECK: The token program
    #[account(
        address = anchor_spl::token::ID
    )]
    pub token_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    pub inscriptions_program: Program<'info, LibreplexInscriptions>,
}

pub fn handler(
    ctx: Context<InscribeLegacyMetadataAsUauth>,
    validation_hash: String,
) -> Result<()> {
    let inscriptions_program = &ctx.accounts.inscriptions_program;
    let inscription_summary = &mut ctx.accounts.inscription_summary;

    let inscription = &mut ctx.accounts.inscription;
    let inscription_data = &mut ctx.accounts.inscription_data;
    let system_program = &ctx.accounts.system_program;
    let mint = &ctx.accounts.mint;
    let legacy_inscription = &mut ctx.accounts.legacy_inscription;

    let inscription_ranks_current_page = &ctx.accounts.inscription_ranks_current_page;
    let inscription_ranks_next_page = &ctx.accounts.inscription_ranks_next_page;
    let legacy_metadata = &ctx.accounts.legacy_metadata;
    let payer_key = ctx.accounts.payer.key();
    // make sure we are dealing with the correct metadata object.
    // this is to ensure that the mint in question is in fact a legacy
    // metadata object
    let payer = &ctx.accounts.payer;
    let legacy_signer = &ctx.accounts.legacy_signer;

    let expected_bump = ctx.bumps.legacy_signer;

    check_metadata_uauth(legacy_metadata, mint.key(), payer_key, AuthorityType::UpdateAuthority)?;

    create_legacy_inscription_logic(
        mint,
        legacy_inscription,
        AuthorityType::UpdateAuthority,
        inscription,
        expected_bump,
        inscriptions_program,
        inscription_summary,
        legacy_signer,
        system_program,
        payer,
        inscription_data,
        inscription_ranks_current_page,
        inscription_ranks_next_page,
        validation_hash,
        SignerType::LegacyMetadataSigner,
    )?;

    Ok(())
}


