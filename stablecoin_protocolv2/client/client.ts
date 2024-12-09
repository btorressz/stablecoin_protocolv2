import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, Keypair } from "@solana/web3.js";

console.log("My address:", pg.wallet.publicKey.toString());

const balance = await pg.connection.getBalance(pg.wallet.publicKey);
console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

// Function to get SPL token balance
async function getTokenBalance(tokenAccount: PublicKey) {
  const balance = await pg.connection.getTokenAccountBalance(tokenAccount);
  console.log(`Token balance: ${balance.value.uiAmount} tokens`);
}

// Function to fetch user account data
async function fetchUserAccount(accountPubkey: PublicKey) {
  const userAccount = await pg.program.account.userAccount.fetch(accountPubkey);
  console.log("User Account Data:", userAccount);
}

// Function to mint stablecoins
async function mintStablecoin(amount: number) {
  const txHash = await pg.program.methods
    .mintStablecoin(new BN(amount))
    .accounts({
      userAccount: pg.wallet.publicKey,
      stablecoinMint: "<Stablecoin Mint Pubkey>",
      userStablecoinAccount: "<User's Token Account Pubkey>",
      tokenProgram: TOKEN_PROGRAM_ID,  
    })
    .rpc();
  console.log(`Mint transaction confirmed: ${txHash}`);
}

// Generate a new keypair
const newKeypair = Keypair.generate();
console.log("Generated new keypair with public key:", newKeypair.publicKey.toString());
