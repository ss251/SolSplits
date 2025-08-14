import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorSolsplits } from "../target/types/anchor_solsplits";
import { 
  Keypair, 
  PublicKey, 
  SystemProgram,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  mintTo,
  getAssociatedTokenAddress,
  getAccount,
  createAssociatedTokenAccount
} from "@solana/spl-token";
import { assert } from "chai";
import crypto from "crypto";

describe("anchor-solsplits", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorSolsplits as Program<AnchorSolsplits>;
  
  const creator = provider.wallet as anchor.Wallet;
  const participant1 = Keypair.generate();
  const participant2 = Keypair.generate();
  const participant3 = Keypair.generate();
  
  let tokenMint: PublicKey;
  let creatorTokenAccount: PublicKey;
  let participant1TokenAccount: PublicKey;
  let participant2TokenAccount: PublicKey;
  let participant3TokenAccount: PublicKey;
  
  const splitId = new anchor.BN(1);
  const fundAmount = new anchor.BN(1000 * 10 ** 6); // 1000 tokens with 6 decimals
  
  before(async () => {
    // Airdrop SOL to participants
    const airdropPromises = [participant1, participant2, participant3].map(async (kp) => {
      const sig = await provider.connection.requestAirdrop(
        kp.publicKey,
        LAMPORTS_PER_SOL
      );
      const latestBlockhash = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction({
        signature: sig,
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
      });
    });
    await Promise.all(airdropPromises);
    
    // Create token mint
    tokenMint = await createMint(
      provider.connection,
      creator.payer,
      creator.publicKey,
      null,
      6
    );
    
    // Create token accounts
    creatorTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      creator.payer,
      tokenMint,
      creator.publicKey
    );
    
    participant1TokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      creator.payer,
      tokenMint,
      participant1.publicKey
    );
    
    participant2TokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      creator.payer,
      tokenMint,
      participant2.publicKey
    );
    
    participant3TokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      creator.payer,
      tokenMint,
      participant3.publicKey
    );
    
    // Mint tokens to creator
    await mintTo(
      provider.connection,
      creator.payer,
      tokenMint,
      creatorTokenAccount,
      creator.publicKey,
      2000 * 10 ** 6
    );
  });

  it("Initialize user registry", async () => {
    const socialHandleHash = crypto.randomBytes(32);
    
    const [userRegistryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_registry"), creator.publicKey.toBuffer()],
      program.programId
    );
    
    const tx = await program.methods
      .initializeUser(Array.from(socialHandleHash))
      .accountsStrict({
        userRegistry: userRegistryPda,
        user: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log("Initialize user tx:", tx);
    
    const userRegistry = await program.account.userRegistry.fetch(userRegistryPda);
    assert.equal(userRegistry.walletPubkey.toString(), creator.publicKey.toString());
    assert.equal(userRegistry.verificationStatus, false);
    assert.equal(userRegistry.totalSplitsCreated, 0);
  });

  it("Create split arrangement", async () => {
    const participants = [
      participant1.publicKey,
      participant2.publicKey,
      participant3.publicKey,
    ];
    const percentages = [5000, 3000, 2000]; // 50%, 30%, 20%
    
    const [userRegistryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_registry"), creator.publicKey.toBuffer()],
      program.programId
    );
    
    const [splitArrangementPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("split"),
        creator.publicKey.toBuffer(),
        splitId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    
    const tx = await program.methods
      .createSplitArrangement(splitId, participants, percentages)
      .accountsStrict({
        splitArrangement: splitArrangementPda,
        userRegistry: userRegistryPda,
        creator: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log("Create split arrangement tx:", tx);
    
    const splitArrangement = await program.account.splitArrangement.fetch(
      splitArrangementPda
    );
    
    assert.equal(splitArrangement.creator.toString(), creator.publicKey.toString());
    assert.equal(splitArrangement.splitId.toString(), splitId.toString());
    assert.equal(splitArrangement.participants.length, 3);
    assert.equal(splitArrangement.percentages.length, 3);
    assert.equal(splitArrangement.status.created !== undefined, true);
  });

  it("Fund split arrangement", async () => {
    const [splitArrangementPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("split"),
        creator.publicKey.toBuffer(),
        splitId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    
    const escrowTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      splitArrangementPda,
      true
    );
    
    const tx = await program.methods
      .fundSplit(fundAmount)
      .accountsStrict({
        splitArrangement: splitArrangementPda,
        escrowTokenAccount: escrowTokenAccount,
        funderTokenAccount: creatorTokenAccount,
        tokenMint: tokenMint,
        funder: creator.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log("Fund split tx:", tx);
    
    const splitArrangement = await program.account.splitArrangement.fetch(
      splitArrangementPda
    );
    
    assert.equal(splitArrangement.status.funded !== undefined, true);
    assert.equal(splitArrangement.totalAmount.toString(), fundAmount.toString());
    assert.equal(splitArrangement.tokenMint.toString(), tokenMint.toString());
    
    const escrowAccount = await getAccount(provider.connection, escrowTokenAccount);
    assert.equal(escrowAccount.amount.toString(), fundAmount.toString());
  });

  it("Execute distribution", async () => {
    const [splitArrangementPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("split"),
        creator.publicKey.toBuffer(),
        splitId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    
    const escrowTokenAccount = await getAssociatedTokenAddress(
      tokenMint,
      splitArrangementPda,
      true
    );
    
    const participantAccounts = [
      participant1TokenAccount,
      participant2TokenAccount,
      participant3TokenAccount,
    ];
    
    const tx = await program.methods
      .executeDistribution()
      .accountsStrict({
        splitArrangement: splitArrangementPda,
        escrowTokenAccount: escrowTokenAccount,
        tokenMint: tokenMint,
        executor: creator.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts(
        participantAccounts.map((account) => ({
          pubkey: account,
          isWritable: true,
          isSigner: false,
        }))
      )
      .rpc();
    
    console.log("Execute distribution tx:", tx);
    
    const splitArrangement = await program.account.splitArrangement.fetch(
      splitArrangementPda
    );
    
    assert.equal(splitArrangement.status.completed !== undefined, true);
    assert.notEqual(splitArrangement.distributedAt, null);
    
    // Check participant balances
    const participant1Account = await getAccount(
      provider.connection,
      participant1TokenAccount
    );
    const participant2Account = await getAccount(
      provider.connection,
      participant2TokenAccount
    );
    const participant3Account = await getAccount(
      provider.connection,
      participant3TokenAccount
    );
    
    // 50% of 1000 = 500
    assert.equal(participant1Account.amount.toString(), (500 * 10 ** 6).toString());
    // 30% of 1000 = 300
    assert.equal(participant2Account.amount.toString(), (300 * 10 ** 6).toString());
    // 20% of 1000 = 200
    assert.equal(participant3Account.amount.toString(), (200 * 10 ** 6).toString());
    
    // Check escrow is empty
    const escrowAccount = await getAccount(provider.connection, escrowTokenAccount);
    assert.equal(escrowAccount.amount.toString(), "0");
  });
});