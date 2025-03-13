import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftMarketplace } from "../target/types/nft_marketplace";
import { assert } from "chai";


describe("nft-defi", () => {
  // // Configure the client to use the local cluster.
  // anchor.setProvider(anchor.AnchorProvider.env());

  // const program = anchor.workspace.nftDefi as Program<NftDefi>;

  // it("Is initialized!", async () => {
  //   // Add your test here.
  //   const tx = await program.methods.initialize().rpc();
  //   console.log("Your transaction signature", tx);
  // });
  // const provider = anchor.AnchorProvider.local();
  // anchor.setProvider(provider);
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.NftMarketplace as Program<NftMarketplace>;
  console.log("Program ID", program.programId.toBase58());

  it("Creates an Auction", async () => {
    console.log("Creating auction...");
    const auction = anchor.web3.Keypair.generate();
    await program.rpc.createAuction(new anchor.BN(100), new anchor.BN(86400), {
      accounts: {
        seller: provider.wallet.publicKey,
        auction: auction.publicKey,
        nftMint: anchor.web3.Keypair.generate().publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [auction],
    });

    const auctionAccount = await program.account.auction.fetch(auction.publicKey);
    assert.ok(auctionAccount.minBid.eq(new anchor.BN(100)));
  });
});
