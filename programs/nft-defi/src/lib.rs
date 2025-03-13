use anchor_lang::prelude::*;
use anchor_lang::system_program::ID;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[program]
pub mod nft_marketplace {
    use super::*;

    pub fn create_auction(ctx: Context<CreateAuction>, min_bid: u64, duration: i64) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        auction.seller = *ctx.accounts.seller.key;
        auction.nft_mint = *ctx.accounts.nft_mint.key;
        auction.min_bid = min_bid;
        auction.end_time = Clock::get()?.unix_timestamp + duration;
        auction.highest_bid = 0;
        auction.highest_bidder = Pubkey::default();
        Ok(())
    }

    pub fn place_bid(ctx: Context<PlaceBid>, bid_amount: u64) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        require!(
            Clock::get()?.unix_timestamp < auction.end_time,
            AuctionError::AuctionEnded
        );
        require!(bid_amount > auction.highest_bid, AuctionError::BidTooLow);

        auction.highest_bid = bid_amount;
        auction.highest_bidder = *ctx.accounts.bidder.key;
        Ok(())
    }

    pub fn finalize_auction(ctx: Context<FinalizeAuction>) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        require!(
            Clock::get()?.unix_timestamp >= auction.end_time,
            AuctionError::AuctionNotEnded
        );

        // Transfer NFT to highest bidder
        let cpi_accounts = Transfer {
            from: ctx.accounts.nft_vault.to_account_info(),
            to: ctx.accounts.winner_nft_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, 1)?;

        auction.finalized = true;
        Ok(())
    }

    pub fn lend_nft(ctx: Context<LendNft>, loan_amount: u64) -> Result<()> {
        let lending = &mut ctx.accounts.lending;
        lending.lender = *ctx.accounts.lender.key;
        lending.nft_mint = *ctx.accounts.nft_mint.key;
        lending.loan_amount = loan_amount;
        lending.borrower = Pubkey::default();
        lending.is_active = true;
        Ok(())
    }

    pub fn borrow_nft(ctx: Context<BorrowNft>, collateral_amount: u64) -> Result<()> {
        let lending = &mut ctx.accounts.lending;
        require!(lending.is_active, LendingError::NotAvailable);
        require!(
            collateral_amount >= lending.loan_amount,
            LendingError::InsufficientCollateral
        );

        lending.borrower = *ctx.accounts.borrower.key;
        lending.is_active = false;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    pub nft_mint: AccountInfo<'info>,
    #[account(init, payer = seller, space = 8 + 96)]
    pub auction: Account<'info, Auction>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(mut)]
    pub auction: Account<'info, Auction>,
}

#[derive(Accounts)]
pub struct FinalizeAuction<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut)]
    pub auction: Account<'info, Auction>,
    #[account(mut)]
    pub nft_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub winner_nft_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct LendNft<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,
    pub nft_mint: AccountInfo<'info>,
    #[account(init, payer = lender, space = 8 + 64)]
    pub lending: Account<'info, Lending>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BorrowNft<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(mut)]
    pub lending: Account<'info, Lending>,
}

#[account]
pub struct Auction {
    pub seller: Pubkey,
    pub nft_mint: Pubkey,
    pub min_bid: u64,
    pub highest_bid: u64,
    pub highest_bidder: Pubkey,
    pub end_time: i64,
    pub finalized: bool,
}

#[account]
pub struct Lending {
    pub lender: Pubkey,
    pub nft_mint: Pubkey,
    pub loan_amount: u64,
    pub borrower: Pubkey,
    pub is_active: bool,
}

#[error_code]
pub enum AuctionError {
    #[msg("Auction has already ended.")]
    AuctionEnded,
    #[msg("Bid is too low.")]
    BidTooLow,
    #[msg("Auction is not ended yet.")]
    AuctionNotEnded,
}

#[error_code]
pub enum LendingError {
    #[msg("NFT is not available for lending.")]
    NotAvailable,
    #[msg("Insufficient collateral.")]
    InsufficientCollateral,
}
