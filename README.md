# SolSplits

Revenue splitting protocol built on Solana for automated payment distribution.

## Status

**✅ COMPLETED**
- Anchor program fully implemented and tested
- Deployed to Devnet: `4pCvQPP2gqbJg56xehy8XmMo98u5DhK6yrGtLGznfmAg`
- All 4 core instructions working
- Comprehensive test suite passing
- Deployed by: `8UeRuQdqVCVwxERmajGkLon92Y8Ax3oJbDZQPGENzJkf` (Turbin3 wallet address)

**⏳ IN PROGRESS**
- Frontend application (Next.js + Bun + Privy + Gill)
  - Basic UI scaffolded
  - Wallet connection partially working
  - *Note: Frontend WIP*

## What it does

SolSplits lets you create payment splits between multiple wallets. Set up the percentages once, then any incoming funds get distributed automatically to all participants.

## Quick Start

### Prerequisites
- Rust 1.75+
- Solana CLI 1.18+
- Anchor 0.30+
- Node.js 18+

### Build
```bash
anchor build
```

### Test
```bash
anchor test
```

### Deploy
```bash
anchor deploy --provider.cluster devnet
```

## How it works

1. **Register users** - Each participant needs a user account
2. **Create split** - Define who gets what percentage (uses basis points: 10000 = 100%)
3. **Fund the split** - Send tokens to the escrow account
4. **Execute distribution** - Triggers automatic payout to all participants

## Program Structure

```
programs/anchor-solsplits/
├── src/
│   ├── instructions/      # Business logic
│   │   ├── initialize_user.rs
│   │   ├── create_split_arrangement.rs
│   │   ├── fund_split.rs
│   │   └── execute_distribution.rs
│   ├── state/             # Account schemas
│   │   ├── user_registry.rs
│   │   └── split_arrangement.rs
│   ├── errors.rs          # Custom errors
│   └── lib.rs             # Entry point
```

## Core Features

- **Multi-participant splits** - Support for 2-10 participants per arrangement
- **SPL token support** - Works with any SPL token, not just SOL
- **Escrow pattern** - Funds held securely until distribution
- **Percentage validation** - Ensures splits always total 100%
- **Status tracking** - Created → Funded → Completed lifecycle
- **User registry** - Social handle verification and reputation system
- **Basis points precision** - 10000 = 100% for accurate percentage splits

## Example Usage

```typescript
// Initialize a user
await program.methods
  .initializeUser(socialHandleHash)
  .accounts({
    user: userPda,
    payer: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

// Create a split (60/40)
await program.methods
  .createSplitArrangement(
    new BN(1),
    [participant1.publicKey, participant2.publicKey],
    [6000, 4000], // basis points
    tokenMint
  )
  .rpc();

// Fund it
await program.methods
  .fundSplit(new BN(1000000))
  .rpc();

// Distribute funds
await program.methods
  .executeDistribution()
  .rpc();
```

## Security Considerations

- Only split creators can execute distributions
- Participants must be registered users
- Percentages must sum to exactly 10000 (100%)
- Maximum 10 participants per split
- Funds locked in escrow until distribution

## Deployed Addresses

**Program ID**: `4pCvQPP2gqbJg56xehy8XmMo98u5DhK6yrGtLGznfmAg`

**Deployment Details**:
- Network: Devnet
- Authority: `8UeRuQdqVCVwxERmajGkLon92Y8Ax3oJbDZQPGENzJkf`
- Deploy Transaction: `5UvdE7cguzNf2eSV6WcsfoC9kaVMxkXFBkZjeUJPiVzNEnam2vj4UnTZCkibsur6wZMw8pVx72gyhVPcz11fwBFa`
- Deployed in Slot: 401403187
- Program Size: 313,928 bytes

## Development

Run tests locally:
```bash
# Start local validator
solana-test-validator

# Run test suite
anchor test --skip-local-validator
```

Check program logs:
```bash
solana logs --url devnet 4pCvQPP2gqbJg56xehy8XmMo98u5DhK6yrGtLGznfmAg
```

## Architecture Decisions

1. **Escrow Pattern**: Chose PDA-based escrow for secure fund holding without needing separate token accounts per split
2. **User Registry**: Implemented to enable social verification and reputation tracking for future features
3. **Basis Points**: Using 10000 as 100% for precise percentage calculations without floating point
4. **Status Enum**: Clear state machine (Created → Funded → Completed) prevents double-spending

## Future Enhancements

- [ ] Recurring splits (subscription model)
- [ ] Time-locked distributions
- [ ] Multi-token support in single split
- [ ] Governance for split modifications
- [ ] Integration with Solana Pay QR codes