use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, MintTo, TokenAccount, Mint, Token};
use anchor_lang::solana_program::clock::Clock;

declare_id!("DUzMJWGEKxbZ3fFnmhiF2r3pzg3FvbiK1UMZp8q9VJpJ");

#[program]
pub mod stablecoin_protocolv2 {
    use super::*;

    // -------------------------------------
    // Initialization Function
    // -------------------------------------

    /// Initialize the protocol, setting the global collateral ratio.
    pub fn initialize(ctx: Context<Initialize>, collateral_ratio: u64) -> Result<()> {
        let governance = &mut ctx.accounts.governance;
        governance.collateral_ratio = collateral_ratio;
        Ok(())
    }

    // -------------------------------------
    // Minting Functions
    // -------------------------------------

    /// Mint stablecoins by depositing collateral.
    pub fn mint_stablecoin(ctx: Context<MintStablecoin>, amount: u64) -> Result<()> {
        let user_account = &ctx.accounts.user_account;

        // Ensure user has enough collateral to mint stablecoins
        let required_collateral = amount
            .checked_mul(user_account.collateral_ratio)
            .ok_or(ErrorCode::Overflow)?;
        require!(
            user_account.collateral_balance >= required_collateral,
            ErrorCode::InsufficientCollateral
        );

        // Mint the stablecoin
        let cpi_accounts = MintTo {
            mint: ctx.accounts.stablecoin_mint.to_account_info(),
            to: ctx.accounts.user_stablecoin_account.to_account_info(),
            authority: ctx.accounts.user_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount)?;

        // Update user account balance after CPI call
        let user_account = &mut ctx.accounts.user_account;
        user_account.stablecoin_balance = user_account
            .stablecoin_balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }

    /// Burn stablecoins to redeem collateral.
    pub fn burn_stablecoin(ctx: Context<BurnStablecoin>, amount: u64) -> Result<()> {
        let user_account = &ctx.accounts.user_account;

        // Ensure user has enough stablecoin balance to burn
        require!(
            user_account.stablecoin_balance >= amount,
            ErrorCode::InsufficientBalance
        );

        // Burn stablecoins
        let cpi_accounts = Burn {
            mint: ctx.accounts.stablecoin_mint.to_account_info(),
            from: ctx.accounts.user_stablecoin_account.to_account_info(),  // Burn from the user's token account
            authority: ctx.accounts.user_account.to_account_info(),        // The authority who can burn
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount)?;

        // Update user account balance after CPI call
        let user_account = &mut ctx.accounts.user_account;
        user_account.stablecoin_balance = user_account
            .stablecoin_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }

    // -------------------------------------
    // Liquidation Function
    // -------------------------------------

    /// Liquidate an under-collateralized position.
    pub fn partial_liquidate(ctx: Context<Liquidate>, liquidation_amount: u64) -> Result<()> {
        let user_account = &ctx.accounts.user_account;

        // Calculate if the user is under-collateralized
        let current_ratio = (user_account.collateral_balance * 100) / user_account.stablecoin_balance;
        require!(
            current_ratio < user_account.collateral_ratio,
            ErrorCode::NotEligibleForLiquidation
        );

        // Calculate penalty and remaining collateral
        let penalty = liquidation_amount / 10; // e.g., 10% penalty
        let remaining_collateral = liquidation_amount.checked_sub(penalty).ok_or(ErrorCode::Overflow)?;

        // Deduct the stablecoin and collateral from user's account
        let user_account = &mut ctx.accounts.user_account;
        user_account.stablecoin_balance = user_account
            .stablecoin_balance
            .checked_sub(liquidation_amount)
            .ok_or(ErrorCode::Overflow)?;

        user_account.collateral_balance = user_account
            .collateral_balance
            .checked_sub(remaining_collateral)
            .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }

    // -------------------------------------
    // Staking Functions
    // -------------------------------------

    /// Stake collateral to earn rewards.
    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        let staker_account = &mut ctx.accounts.staker_account;

        // Update staked balance
        staker_account.staked_balance = staker_account
            .staked_balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        staker_account.last_stake_time = Clock::get()?.unix_timestamp as u64; // Track the time when tokens are staked

        Ok(())
    }

    /// Withdraw staked tokens with optional early withdrawal penalty.
    pub fn withdraw_stake(ctx: Context<WithdrawStake>, amount: u64) -> Result<()> {
        let staker_account = &mut ctx.accounts.staker_account;
        let current_time = Clock::get()?.unix_timestamp as u64;

        // Apply penalty if withdrawn before lock-up period ends
        let penalty = if current_time < staker_account.lockup_period {
            amount * staker_account.early_withdrawal_penalty / 100
        } else {
            0
        };

        // Transfer tokens and update staked balance
        let final_amount = amount.checked_sub(penalty).ok_or(ErrorCode::Overflow)?;
        staker_account.staked_balance = staker_account
            .staked_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }

    /// Distribute staking rewards based on staking duration.
    pub fn distribute_rewards(ctx: Context<StakeTokens>) -> Result<()> {
        let staker_account = &mut ctx.accounts.staker_account;
        let current_time = Clock::get()?.unix_timestamp as u64;

        // Calculate rewards based on staking duration
        let staking_duration = current_time - staker_account.last_stake_time;
        let reward_rate = 100;  // Example reward rate per second

        let rewards = staking_duration
            .checked_mul(reward_rate)
            .ok_or(ErrorCode::Overflow)?;

        // Add the rewards to the user's balance
        staker_account.staked_balance = staker_account
            .staked_balance
            .checked_add(rewards)
            .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }

    // -------------------------------------
    // Governance Functions
    // -------------------------------------

    /// Create a new governance proposal.
    pub fn create_proposal(ctx: Context<CreateProposal>, description: String) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        proposal.description = description;
        proposal.status = ProposalStatus::Pending;

        Ok(())
    }

    /// Vote on an existing proposal with a minimum quorum requirement.
    pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, approve: bool) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;

        if approve {
            proposal.approval_votes += 1;
        } else {
            proposal.reject_votes += 1;
        }

        let total_votes = proposal.approval_votes + proposal.reject_votes;
        let quorum_required = 10;  // Example: minimum of 10 votes required

        if total_votes >= quorum_required {
            if proposal.approval_votes > proposal.reject_votes {
                proposal.status = ProposalStatus::Approved;
            } else {
                proposal.status = ProposalStatus::Rejected;
            }
        }

        Ok(())
    }
}

// -------------------------------------
// Account Structures
// -------------------------------------

#[account]
pub struct Governance {
    pub collateral_ratio: u64,
}

#[account]
pub struct UserAccount {
    pub collateral_balance: u64,
    pub stablecoin_balance: u64,
    pub collateral_ratio: u64,
}

#[account]
pub struct StakerAccount {
    pub staked_balance: u64,
    pub lockup_period: u64,
    pub early_withdrawal_penalty: u64,
    pub last_stake_time: u64,  // Added to track staking time
}

#[account]
pub struct Proposal {
    pub description: String,
    pub approval_votes: u32,
    pub reject_votes: u32,
    pub status: ProposalStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
}

// -------------------------------------
// Instruction Contexts
// -------------------------------------

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 32)]
    pub governance: Account<'info, Governance>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintStablecoin<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub stablecoin_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnStablecoin<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub stablecoin_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user_stablecoin_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub liquidator_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub staker_account: Account<'info, StakerAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawStake<'info> {
    #[account(mut)]
    pub staker_account: Account<'info, StakerAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(init, payer = proposer, space = 8 + 128)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub governance: Account<'info, Governance>,
    #[account(mut)]
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub governance: Account<'info, Governance>,
    pub voter: Signer<'info>,
}

// -------------------------------------
// Error Codes
// -------------------------------------

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient collateral to mint stablecoin")]
    InsufficientCollateral,
    #[msg("Insufficient stablecoin balance to burn")]
    InsufficientBalance,
    #[msg("Calculation overflow")]
    Overflow,
    #[msg("Not eligible for liquidation")]
    NotEligibleForLiquidation,
}
